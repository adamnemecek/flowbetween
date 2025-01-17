use super::db_enum::*;
use super::flo_store::*;
use super::motion_path_type::*;
use super::super::error::*;
use super::super::result::Result;

use desync::*;
use flo_logging::*;
use flo_animation::*;
use flo_animation::brushes::*;

use itertools::*;
use std::sync::*;
use std::time::Duration;
use std::collections::HashMap;

///
/// The element IDs for the path properties for a particular layer
///
#[derive(Clone, Copy, Debug)]
pub struct PathPropertiesIds {
    /// The ID of the element defining the path brush
    pub brush_id: ElementId,

    /// The ID of the element defining the path brush properties
    pub properties_id: ElementId
}

///
/// A list of element IDs to attach automatically to a new element
///
#[derive(Clone, Debug)]
pub struct AttachProperties {
    /// The properties to attach of each type
    pub property_of_type: HashMap<VectorType, ElementId>
}

///
/// Core data structure used by the animation database
///
pub struct AnimationDbCore<TFile: FloFile+Send> {
    /// The logger for the core
    pub log: Arc<LogPublisher>,

    /// The database connection
    pub db: TFile,

    /// Pending work for generating cached data
    pub cache_work: Arc<Desync<()>>,

    /// If there has been a failure with the database, this is it. No future operations
    /// will work while there's an error that hasn't been cleared
    pub failure: Option<SqliteAnimationError>,

    /// Maps a layer ID to the properties that should be associated with the next path created
    pub path_properties_for_layer: HashMap<i64, PathPropertiesIds>,

    /// Maps a layer ID to the properties that should be associated with the next brush stroke created
    pub brush_properties_for_layer: HashMap<i64, AttachProperties>,

    /// Maps layers to the brush that's active
    pub active_brush_for_layer: HashMap<i64, (Duration, Arc<dyn Brush>)>,

    /// Maps the assigned layer IDs to their equivalent real IDs
    pub layer_id_for_assigned_id: HashMap<u64, i64>,

    /// The next element ID that will be assigned
    pub next_element_id: i64
}

impl<TFile: FloFile+Send> AnimationDbCore<TFile> {
    ///
    /// Assigns the next element ID and returns it
    ///
    pub fn next_element_id(&mut self) -> i64 {
        let result      = self.next_element_id;
        self.next_element_id += 1;
        result
    }

    ///
    /// Assigns an element ID to an animation edit
    ///
    fn assign_element_id(&mut self, edit: AnimationEdit) -> AnimationEdit {
        use self::AnimationEdit::*;
        use self::LayerEdit::*;
        use self::PaintEdit::*;

        match edit {
            Layer(layer_id, Paint(when, BrushProperties(ElementId::Unassigned, props))) =>
                Layer(layer_id, Paint(when, BrushProperties(ElementId::Assigned(self.next_element_id()), props))),

            Layer(layer_id, Paint(when, SelectBrush(ElementId::Unassigned, defn, drawing_style))) =>
                Layer(layer_id, Paint(when, SelectBrush(ElementId::Assigned(self.next_element_id()), defn, drawing_style))),

            Layer(layer_id, Paint(when, BrushStroke(ElementId::Unassigned, points))) =>
                Layer(layer_id, Paint(when, BrushStroke(ElementId::Assigned(self.next_element_id()), points))),

            Layer(layer_id, Path(when, PathEdit::CreatePath(ElementId::Unassigned, points))) =>
                Layer(layer_id, Path(when, PathEdit::CreatePath(ElementId::Assigned(self.next_element_id()), points))),

            Layer(layer_id, Path(when, PathEdit::SelectBrush(ElementId::Unassigned, definition, style))) =>
                Layer(layer_id, Path(when, PathEdit::SelectBrush(ElementId::Assigned(self.next_element_id()), definition, style))),

            Layer(layer_id, Path(when, PathEdit::BrushProperties(ElementId::Unassigned, properties))) =>
                Layer(layer_id, Path(when, PathEdit::BrushProperties(ElementId::Assigned(self.next_element_id()), properties))),

            other => other
        }
    }

    ///
    /// Assigns element IDs to a set of animation IDs
    ///
    pub fn assign_element_ids(&mut self, edits: Vec<AnimationEdit>) -> Vec<AnimationEdit> {
        edits.into_iter()
            .map(|edit| self.assign_element_id(edit))
            .collect()
    }

    ///
    /// Retrieves the brush that is active on the specified layer at the specified time
    ///
    pub fn get_active_brush_for_layer(&mut self, layer_id: i64, when: Duration) -> Option<Arc<dyn Brush>> {
        // If the cached active brush is at the right time, then just use that
        if let Some((time, ref brush)) = self.active_brush_for_layer.get(&layer_id) {
            if time == &when {
                return Some(Arc::clone(&brush));
            }
        }

        // Get the brush properties for the layer
        if let Some(properties) = self.brush_properties_for_layer.get(&layer_id) {
            // Try to retrieve the brush definition ID from the properties
            if let Some(brush_definition_id) = properties.property_of_type.get(&VectorType::BrushDefinition) {
                // Turn these properties into a brush
                let brush_definition_id         = self.db.query_vector_element_id(brush_definition_id).unwrap().unwrap();
                let brush_definition            = self.db.query_vector_element(brush_definition_id).unwrap();
                let (brush_id, drawing_style)   = brush_definition.brush.unwrap();
                let brush_defn                  = Self::get_brush_definition(&mut self.db, brush_id).unwrap();
                let brush                       = create_brush_from_definition(&brush_defn, drawing_style.into());

                // Cache the brush for faster retrieval next time
                self.active_brush_for_layer.insert(layer_id, (when, Arc::clone(&brush)));

                // This is our result
                Some(brush)
            } else {
                // Brush properties have been set but not the brush definition
                None
            }
        } else {
            // No brush properties have been set yet for this layer
            None
        }
    }

    ///
    /// Creates a new vector element in an animation DB core, leaving the element ID, key frame ID and time pushed on the DB stack
    ///
    /// The element is created without its associated data.
    ///
    fn create_new_element(db: &mut TFile, layer_id: i64, when: Duration, element_id: ElementId, element_type: VectorElementType) -> Result<()> {
        if let ElementId::Assigned(assigned_id) = element_id {
            db.update(vec![
                DatabaseUpdate::PushLayerId(layer_id),
                DatabaseUpdate::PushNearestKeyFrame(when),
                DatabaseUpdate::PushVectorElementType(element_type),
                DatabaseUpdate::PushVectorElementTime(when),
                DatabaseUpdate::PushElementAssignId(assigned_id)
            ])?;
        } else {
            db.update(vec![
                DatabaseUpdate::PushLayerId(layer_id),
                DatabaseUpdate::PushNearestKeyFrame(when),
                DatabaseUpdate::PushVectorElementType(element_type),
                DatabaseUpdate::PushVectorElementTime(when)
            ])?;
        }

        Ok(())
    }

    ///
    /// Creates a new vector element in an animation DB core, leaving the element ID pushed on the DB stack
    ///
    /// The element is created without its associated data.
    ///
    fn create_unattached_element(db: &mut TFile, element_type: VectorElementType, id: ElementId) -> Result<()> {
        if let ElementId::Assigned(assigned_id) = id {
            db.update(vec![
                DatabaseUpdate::PushVectorElementType(element_type),
                DatabaseUpdate::PushElementAssignId(assigned_id)
            ])?;
        } else {
            db.update(vec![
                DatabaseUpdate::PushVectorElementType(element_type),
            ])?;
        }

        Ok(())
    }

    ///
    /// Writes a brush properties element to the database (popping the element ID)
    ///
    fn create_brush_properties(db: &mut TFile, properties: BrushProperties) -> Result<()> {
        Self::insert_brush_properties(db, &properties)?;

        // Create the element
        db.update(vec![
            DatabaseUpdate::PopVectorBrushPropertiesElement
        ])?;

        Ok(())
    }

    ///
    /// Writes a brush definition element to the database (popping the element ID)
    ///
    fn create_brush_definition(db: &mut TFile, definition: BrushDefinition, drawing_style: BrushDrawingStyle) -> Result<()> {
        // Create the brush definition
        Self::insert_brush(db, &definition)?;

        // Insert the properties for this element
        db.update(vec![
            DatabaseUpdate::PopVectorBrushElement(DrawingStyleType::from(&drawing_style))
        ])?;

        Ok(())
    }

    ///
    /// Writes a brush stroke to the database (popping the element ID)
    ///
    fn create_brush_stroke(&mut self, layer_id: i64, when: Duration, brush_stroke: Arc<Vec<RawPoint>>) -> Result<()> {
        // Convert the brush stroke to the brush points
        let active_brush = self.get_active_brush_for_layer(layer_id, when);
        if let Some(active_brush) = active_brush {
            let brush_stroke = active_brush.brush_points_for_raw_points(&*brush_stroke);

            // Store in the database
            self.db.update(vec![
                DatabaseUpdate::PopBrushPoints(Arc::new(brush_stroke))
            ])?;
        }

        Ok(())
    }

    ///
    /// Adds a new vector element to a vector layer
    ///
    fn paint_vector_layer(&mut self, layer_id: i64, when: Duration, new_element: PaintEdit) -> Result<()> {
        use self::PaintEdit::*;

        // Update the state of this object based on the element
        match new_element {
            SelectBrush(_id, ref brush_definition, drawing_style)   => {
                // Cache the brush so that follow up drawing instructions don't need to
                self.active_brush_for_layer.insert(layer_id, (when, create_brush_from_definition(brush_definition, drawing_style)));
            },

            _ => ()
        }

        // Record the details of the element itself
        match new_element {
            BrushStroke(id, brush_stroke)                       => {
                // New brush stroke element
                Self::create_new_element(&mut self.db, layer_id, when, id, VectorElementType::BrushStroke)?;

                // Attach the properties for this brush stroke
                let property_elements   = self.brush_properties_for_layer.get(&layer_id)
                    .map(|properties| properties.property_of_type.values().filter_map(|elem| elem.id()))
                    .map(|assigned_ids| assigned_ids.map(|assigned_id| DatabaseUpdate::PushElementIdForAssignedId(assigned_id)))
                    .map(|push_ids| push_ids.collect::<Vec<_>>())
                    .unwrap_or_else(|| vec![]);
                let num_properties      = property_elements.len();

                if num_properties > 0 {
                    // Push all of the assigned IDs for the properties, followed by attaching them to the brush element we're building
                    self.db.update(property_elements.into_iter()
                        .chain(vec![DatabaseUpdate::PushAttachElements(num_properties)]))?;
                }

                // Create the brush stroke (popping the element ID)
                self.create_brush_stroke(layer_id, when, brush_stroke)?;

                // Pop the frame ID and time (create_brush_stroke will have popped the element ID)
                self.db.update(vec![DatabaseUpdate::Pop, DatabaseUpdate::Pop])?;
            },

            SelectBrush(id, brush_definition, drawing_style)    => {
                // Create a new brush definition to use with the future brush strokes
                Self::create_unattached_element(&mut self.db, VectorElementType::BrushDefinition, id)?;
                Self::create_brush_definition(&mut self.db, brush_definition, drawing_style)?;

                // Attach to future brush strokes
                self.brush_properties_for_layer.entry(layer_id)
                    .or_insert_with(|| AttachProperties { property_of_type: HashMap::new() })
                    .property_of_type.insert(VectorType::BrushDefinition, id);
            },
            BrushProperties(id, brush_properties)               => {
                // Create a new brush properties to use with the future brush strokes
                Self::create_unattached_element(&mut self.db, VectorElementType::BrushProperties, id)?;
                Self::create_brush_properties(&mut self.db, brush_properties)?;

                // Attach to future brush strokes
                self.brush_properties_for_layer.entry(layer_id)
                    .or_insert_with(|| AttachProperties { property_of_type: HashMap::new() })
                    .property_of_type.insert(VectorType::BrushProperties, id);
            },
        }

        Ok(())
    }

    ///
    /// Adds a vector path to a vector layer
    ///
    fn path_vector_layer(&mut self, layer_id: i64, when: Duration, new_element: PathEdit) -> Result<()> {
        use self::PathEdit::*;

        // Update the state of this object based on the element
        match new_element {
            CreatePath(element_id, components)                          => {
                // Get the current path properties
                let path_properties = self.path_properties_for_layer.entry(layer_id)
                    .or_insert_with(|| PathPropertiesIds { brush_id: ElementId::Unassigned, properties_id: ElementId::Unassigned });

                match (path_properties.brush_id, path_properties.properties_id) {
                    (ElementId::Assigned(brush_id), ElementId::Assigned(properties_id)) => {
                        // Need the stack to be path_id, brush_properties_id, brush_id, element_id to create a path element
                        Self::create_new_element(&mut self.db, layer_id, when, element_id, VectorElementType::Path)?;
                        self.db.update(vec![
                            DatabaseUpdate::PushElementIdForAssignedId(brush_id),
                            DatabaseUpdate::PushElementIdForAssignedId(properties_id),
                            DatabaseUpdate::PushPathComponents(components),
                            DatabaseUpdate::PopVectorPathElement
                        ])?;
                    },

                    (ElementId::Assigned(_brush_id), ElementId::Unassigned) => {
                        // TODO: proper logging
                        println!("Can't create path: properties ID not defined")
                    },

                    _ => {
                        // TODO: proper logging
                        println!("Can't create path: brush ID not defined")
                    }
                }
            },

            SelectBrush(element_id, brush_definition, drawing_style)    => {
                // Create a new brush definition to use with the path and store it
                Self::create_unattached_element(&mut self.db, VectorElementType::BrushDefinition, element_id)?;
                Self::create_brush_definition(&mut self.db, brush_definition, drawing_style)?;

                self.path_properties_for_layer.entry(layer_id)
                    .or_insert_with(|| PathPropertiesIds { brush_id: ElementId::Unassigned, properties_id: ElementId::Unassigned })
                    .brush_id = element_id;
            },

            BrushProperties(element_id, brush_properties)               => {
                // Create some new brush properties to use with the path and store them
                Self::create_unattached_element(&mut self.db, VectorElementType::BrushProperties, element_id)?;
                Self::create_brush_properties(&mut self.db, brush_properties)?;

                self.path_properties_for_layer.entry(layer_id)
                    .or_insert_with(|| PathPropertiesIds { brush_id: ElementId::Unassigned, properties_id: ElementId::Unassigned })
                    .properties_id = element_id;
            }
        }

        Ok(())
    }

    ///
    /// Performs an editing action on a motion
    ///
    fn edit_motion(&mut self, motion_id: ElementId, edit: MotionEdit) -> Result<()> {
        use self::MotionEdit::*;

        if let ElementId::Assigned(motion_id) = motion_id {
            // Motion IDs must have an assigned ID
            let motion_id = motion_id as i64;

            match edit {
                Create => {
                    self.db.update(vec![
                        DatabaseUpdate::PushVectorElementType(VectorElementType::Motion),
                        DatabaseUpdate::PushElementAssignId(motion_id),
                        DatabaseUpdate::Pop,
                        DatabaseUpdate::CreateMotion(motion_id),
                    ])?;
                },

                Delete => {
                    self.db.update(vec![
                        DatabaseUpdate::DeleteMotion(motion_id)
                    ])?;
                },

                SetType(motion_type) => {
                    self.db.update(vec![
                        DatabaseUpdate::SetMotionType(motion_id, motion_type)
                    ])?;
                },

                SetOrigin(x, y) => {
                    self.db.update(vec![
                        DatabaseUpdate::SetMotionOrigin(motion_id, x, y)
                    ])?;
                },

                SetPath(time_path) => {
                    // Create the points in the curve
                    self.db.update(time_path.points
                        .iter()
                        .flat_map(|control_point| vec![&control_point.point, &control_point.past, &control_point.future])
                        .map(|&TimePoint(ref x, ref y, ref millis)| DatabaseUpdate::PushTimePoint(*x, *y, *millis)))?;

                    // Turn into a motion path
                    self.db.update(vec![DatabaseUpdate::SetMotionPath(motion_id, MotionPathType::Position, time_path.points.len()*3)])?;
                },
            }
        }

        Ok(())
    }

    ///
    /// Edits the element with the specified ID
    ///
    fn edit_element(&mut self, element_id: ElementId, element_edit: ElementEdit) -> Result<()> {
        if let ElementId::Assigned(assigned_id) = element_id {
            // Get the type of the element so we can use the appropriate editing method
            let element_type = self.db.query_vector_element_type_from_assigned_id(assigned_id)?;

            if let Some(element_type) = element_type {
                // Action depends on the element type
                match (element_type, element_edit) {
                    (VectorElementType::BrushStroke, ElementEdit::SetControlPoints(points)) => {
                        // The first point doesn't have 'real' control points, so we duplicate them here
                        let prefix = vec![points[0], points[0]];
                        let points = prefix.into_iter().chain(points.into_iter());

                        // Convert to tuples. Ordering is cp1, cp2, pos.
                        let points = points.tuples()
                            .collect();

                        // Perform the update
                        self.db.update(vec![
                            DatabaseUpdate::PushElementIdForAssignedId(assigned_id),
                            DatabaseUpdate::UpdateBrushPointCoords(Arc::new(points))
                        ])?;
                    },

                    (VectorElementType::Path, ElementEdit::SetControlPoints(points)) => {
                        // Paths don't format their points so we can just update them immediately
                        self.db.update(vec![
                            DatabaseUpdate::PushElementIdForAssignedId(assigned_id),
                            DatabaseUpdate::PushPathIdForElementId,
                            DatabaseUpdate::UpdatePathPointCoords(Arc::new(points)),
                            DatabaseUpdate::Pop
                        ])?;
                    },

                    (VectorElementType::Path, ElementEdit::SetPath(components)) => {
                        // Count the number of points in this path before the update
                        let element_id          = self.db.query_vector_element_id(&ElementId::Assigned(assigned_id))?
                            .ok_or(SqliteAnimationError::MissingElementId(ElementId::Assigned(assigned_id)))?;
                        let path_entry          = self.db.query_path_element(element_id)?
                            .ok_or(SqliteAnimationError::UnexpectedElementType(ElementId::Assigned(assigned_id)))?;
                        let existing_components = self.db.query_path_components(path_entry.path_id)?;
                        let number_of_points    = existing_components.into_iter().map(|component| component.num_points()).sum::<usize>();

                        // Remove and replace all of the points
                        self.db.update(vec![
                            DatabaseUpdate::PushElementIdForAssignedId(assigned_id),
                            DatabaseUpdate::PushPathIdForElementId,
                            DatabaseUpdate::Duplicate,
                            DatabaseUpdate::PopRemovePathPoints(0..number_of_points),
                            DatabaseUpdate::PopInsertPathComponents(0, components),
                            DatabaseUpdate::Pop
                        ])?;
                    },

                    (_any_type, ElementEdit::Order(ordering)) => {
                        let update_order = match ordering {
                            ElementOrdering::InFront    => vec![DatabaseUpdate::PopVectorElementMove(DbElementMove::Up)],
                            ElementOrdering::Behind     => vec![DatabaseUpdate::PopVectorElementMove(DbElementMove::Down)],
                            ElementOrdering::ToTop      => vec![DatabaseUpdate::PopVectorElementMove(DbElementMove::ToTop)],
                            ElementOrdering::ToBottom   => vec![DatabaseUpdate::PopVectorElementMove(DbElementMove::ToBottom)],
                            ElementOrdering::Before(_)  => unimplemented!()
                        }.into_iter();

                        // Need to push the element ID and the keyframe ID
                        self.db.update(vec![
                            DatabaseUpdate::PushElementIdForAssignedId(assigned_id),
                            DatabaseUpdate::PushKeyFrameIdForElementId
                        ].into_iter()
                        .chain(update_order))?;
                    },

                    (_any_type, ElementEdit::AddAttachment(attach_element_id)) => {
                        if let ElementId::Assigned(attach_element_id) = attach_element_id {
                            self.db.update(vec![
                                DatabaseUpdate::PushElementIdForAssignedId(assigned_id),
                                DatabaseUpdate::PushElementIdForAssignedId(attach_element_id),
                                DatabaseUpdate::PushAttachElements(1),
                                DatabaseUpdate::Pop
                            ])?;
                        }
                    },

                    (_any_type, ElementEdit::RemoveAttachment(detach_element_id)) => {
                        if let ElementId::Assigned(detach_element_id) = detach_element_id {
                            self.db.update(vec![
                                DatabaseUpdate::PushElementIdForAssignedId(assigned_id),
                                DatabaseUpdate::PushElementIdForAssignedId(detach_element_id),
                                DatabaseUpdate::PushDetachElements(1),
                                DatabaseUpdate::Pop
                            ])?;
                        }
                    },

                    (_any_type, ElementEdit::Delete) => {
                        self.db.update(vec![
                            DatabaseUpdate::PushElementIdForAssignedId(assigned_id),
                            DatabaseUpdate::PopDeleteVectorElement
                        ])?;
                    },

                    (_any_type, ElementEdit::DetachFromFrame) => {
                        self.db.update(vec![
                            DatabaseUpdate::PushElementIdForAssignedId(assigned_id),
                            DatabaseUpdate::PopDetachVectorElementFromFrame
                        ])?;
                    },

                    // Other types have no action
                    _ => ()
                }
            } else {
                // No action if this element has no type
            }
        }

        Ok(())
    }

    ///
    /// Sends an editing operation to many elements at once
    ///
    fn edit_many_elements<ElementIter: IntoIterator<Item=ElementId>>(&mut self, element_ids: ElementIter, element_edit: ElementEdit) -> Result<()> {
        // TODO: some operations (ordering, for instance) can be performed more efficiently by performing the edits on all of the elements as a single operation
        for element_id in element_ids {
            self.edit_element(element_id, element_edit.clone())?;
        }

        Ok(())
    }

    ///
    /// Performs a layer edit to a vector layer
    ///
    pub fn edit_vector_layer(&mut self, layer_id: i64, edit: LayerEdit) -> Result<()> {
        use self::LayerEdit::*;

        // Note that we can't access the core at this point (the database implies that the core is already in use)

        match edit {
            AddKeyFrame(when) => {
                self.db.update(vec![
                    DatabaseUpdate::PushLayerId(layer_id),
                    DatabaseUpdate::PopAddKeyFrame(when)
                ])?;

                self.db.update(vec![
                    DatabaseUpdate::PushLayerId(layer_id),
                    DatabaseUpdate::PopDeleteLayerCache(when, CacheType::OnionSkinLayer)
                ])?;
            },

            RemoveKeyFrame(when) => {
                self.db.update(vec![
                    DatabaseUpdate::PushLayerId(layer_id),
                    DatabaseUpdate::PopRemoveKeyFrame(when)
                ])?;

                self.db.update(vec![
                    DatabaseUpdate::PushLayerId(layer_id),
                    DatabaseUpdate::PopDeleteLayerCache(when, CacheType::OnionSkinLayer)
                ])?;
            },

            Paint(when, edit) => {
                self.paint_vector_layer(layer_id, when, edit)?;
                self.db.update(vec![
                    DatabaseUpdate::PushLayerId(layer_id),
                    DatabaseUpdate::PopDeleteLayerCache(when, CacheType::OnionSkinLayer)
                ])?;
            },

            Path(when, edit) => {
                self.path_vector_layer(layer_id, when, edit)?;
                self.db.update(vec![
                    DatabaseUpdate::PushLayerId(layer_id),
                    DatabaseUpdate::PopDeleteLayerCache(when, CacheType::OnionSkinLayer)
                ])?;
            }

            SetName(new_name) => {
                self.db.update(vec![
                    DatabaseUpdate::PushLayerId(layer_id),
                    DatabaseUpdate::PopLayerName(new_name.clone())
                ])?;
            },

            SetOrdering(at_index) => {
                unimplemented!("Layer ordering not implemented yet")
            }
        }

        Ok(())
    }

    ///
    /// Performs an edit on this core
    ///
    pub fn perform_edit(&mut self, edit: AnimationEdit) -> Result<()> {
        use self::AnimationEdit::*;

        let result = self.log.clone().with(|| {
            match edit {
                SetSize(width, height) => {
                    self.db.update(vec![
                        DatabaseUpdate::UpdateCanvasSize(width, height)
                    ])?;
                },

                AddNewLayer(new_layer_id) => {
                    // Create a layer with the new ID
                    self.db.update(vec![
                        DatabaseUpdate::PushLayerType(LayerType::Vector),
                        DatabaseUpdate::PushAssignLayer(new_layer_id),
                        DatabaseUpdate::Pop
                    ])?;
                },

                RemoveLayer(old_layer_id) => {
                    // Delete this layer
                    self.db.update(vec![
                        DatabaseUpdate::PushLayerForAssignedId(old_layer_id),
                        DatabaseUpdate::PopDeleteLayer
                    ])?;
                },

                Layer(assigned_layer_id, layer_edit) => {
                    // Look up the real layer ID (which is often different to the assigned ID)
                    let layer_id = {
                        let db                          = &mut self.db;
                        let layer_id_for_assigned_id    = &mut self.layer_id_for_assigned_id;
                        let layer_id                    = *layer_id_for_assigned_id.entry(assigned_layer_id)
                            .or_insert_with(|| db.query_layer_id_for_assigned_id(assigned_layer_id).map(|(id, _name)| id).unwrap_or(-1));

                        layer_id
                    };

                    // Edit this layer
                    self.edit_vector_layer(layer_id, layer_edit)?;
                },

                Element(element_ids, element_edit) => {
                    self.edit_many_elements(element_ids, element_edit)?;
                },

                Motion(motion_id, motion_edit) => {
                    self.edit_motion(motion_id, motion_edit)?;
                }
            }

            Ok(())
        });

        if let Err(ref error) = result {
            self.log.log((Level::Error, format!("Could not perform edit due to error {:?}", error)));
        }

        result
    }
}
