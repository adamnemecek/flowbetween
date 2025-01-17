use super::*;
use super::db_enum::*;
use super::flo_store::*;

use std::iter;

use self::DatabaseUpdate::*;

impl<TFile: FloFile+Send> AnimationDbCore<TFile> {
    ///
    /// Inserts a set of edits into the database
    ///
    pub fn insert_edits(&mut self, edits: &[AnimationEdit]) -> Result<()> {
        // Insert all of the edits in turn
        self.db.begin_queuing();
        for edit in edits {
            self.insert_edit_log(edit)?;
        }
        self.db.execute_queue()?;

        Ok(())
    }

    ///
    /// Inserts an element ID into the edit log
    ///
    fn insert_element_id(db: &mut TFile, element_id: &ElementId) -> Result<()> {
        use self::ElementId::*;

        match element_id {
            &Unassigned     => { },
            &Assigned(id)   => { db.update(vec![PushEditLogElementId(0, id)])?; },
        }

        Ok(())
    }

    ///
    /// Inserts an element ID into the edit log
    ///
    fn insert_element_id_list(db: &mut TFile, element_ids: &Vec<ElementId>) -> Result<()> {
        use self::ElementId::*;

        db.update(element_ids.iter()
            .enumerate()
            .map(|(index, element_id)| {
                match element_id {
                    &Unassigned     => { vec![] },
                    &Assigned(id)   => { vec![PushEditLogElementId(index, id)] },
                }
            })
            .flatten())?;

        Ok(())
    }

    ///
    /// Inserts a single AnimationEdit into the edit log
    ///
    fn insert_edit_log<'a>(&mut self, edit: &AnimationEdit) -> Result<()> {
        // Create the edit type and push the ID
        self.db.update(vec![PushEditType(EditLogType::from(edit))])?;

        // Insert the values for this edit and pop the ID
        self.insert_animation_edit(edit)?;

        Ok(())
    }

    ///
    /// Inserts the values for an AnimationEdit into the edit log (db must have an edit ID pushed. This will be popped when this returns)
    ///
    fn insert_animation_edit<'a>(&mut self, edit: &AnimationEdit) -> Result<()> {
        use self::AnimationEdit::*;

        match edit {
            &Layer(layer_id, ref layer_edit)                => {
                self.db.update(vec![PushEditLogLayer(layer_id)])?;
                self.insert_layer_edit(layer_edit)?;
            },

            &SetSize(width, height)                         => {
                self.db.update(vec![PopEditLogSetSize(width as f32, height as f32)])?;
            },

            &AddNewLayer(layer_id)                          => {
                self.db.update(vec![PushEditLogLayer(layer_id), Pop])?;
            },

            &RemoveLayer(layer_id)                          => {
                self.db.update(vec![PushEditLogLayer(layer_id), Pop])?;
            },

            &Element(ref element_ids, ElementEdit::AddAttachment(element_id))    => {
                // The 'attached' element ID appears at the start of the list as we store it in the database, which isn't possible in insert_element_edit
                Self::insert_element_id_list(&mut self.db, &(iter::once(element_id).chain(element_ids.iter().cloned()).collect()))?;
                self.db.update(vec![Pop])?;
            },

            &Element(ref element_ids, ElementEdit::RemoveAttachment(element_id))    => {
                // The 'attached' element ID appears at the start of the list as we store it in the database, which isn't possible in insert_element_edit
                Self::insert_element_id_list(&mut self.db, &(iter::once(element_id).chain(element_ids.iter().cloned()).collect()))?;
                self.db.update(vec![Pop])?;
            },

            &Element(ref element_ids, ref element_edit)     => {
                Self::insert_element_id_list(&mut self.db, element_ids)?;
                self.insert_element_edit(element_edit)?;
            },

            &Motion(motion_id, ref motion_edit)             => {
                Self::insert_element_id(&mut self.db, &motion_id)?;
                self.insert_motion_edit(motion_edit)?;
            }
        };

        Ok(())
    }

    ///
    /// Inserts the parameters for an element edit into the edit log
    ///
    /// The stack contains the edit log ID when this is called
    ///
    fn insert_element_edit(&mut self, edit: &ElementEdit) -> Result<()> {
        use self::ElementEdit::*;

        match edit {
            AddAttachment(_element_id) | RemoveAttachment(_element_id) => {
                // Generally shouldn't be generated (see insert_animation_edit above for where this is actually implemented)
                self.db.update(vec![Pop])?;
            },

            SetControlPoints(points) => {
                self.db.update(vec![
                    PushPath(points.clone()),
                    PushEditLogPath,
                    Pop
                ])?;
            },

            SetPath(components) => {
                self.db.update(vec![
                    PushPathComponents(Arc::clone(components)),
                    PushEditLogPath,
                    Pop
                ])?;
            },

            Order(ElementOrdering::Before(element_id))  => {
                match element_id {
                    ElementId::Assigned(element_id)     => { self.db.update(vec![PushEditLogInt(0, *element_id as i64), Pop])?; }
                    ElementId::Unassigned               => { self.db.update(vec![Pop])?; }
                }
            }
            Order(_ordering)                            => { self.db.update(vec![Pop])?; }
            Delete                                      => { self.db.update(vec![Pop])?; }
            DetachFromFrame                             => { self.db.update(vec![Pop])?; }
        }

        Ok(())
    }

    ///
    /// Inserts the parameters for a motion edit into the edit log
    ///
    fn insert_motion_edit(&mut self, edit: &MotionEdit) -> Result<()> {
        use self::MotionEdit::*;

        match edit {
            Create                  => {
                self.db.update(vec![Pop])?;
            },

            Delete                  => {
                self.db.update(vec![Pop])?;
            },

            SetType(motion_type)    => {
                self.db.update(vec![PushEditLogMotionType(*motion_type), Pop])?;
            },

            SetOrigin(x, y)         => {
                self.db.update(vec![PushEditLogMotionOrigin(*x, *y), Pop])?;
            },

            SetPath(curve)          => {
                // Create the points in the curve
                self.db.update(curve.points
                    .iter()
                    .flat_map(|control_point| vec![&control_point.point, &control_point.past, &control_point.future])
                    .map(|&TimePoint(ref x, ref y, ref millis)| PushTimePoint(*x, *y, *millis)))?;

                // Turn into an edit log path
                self.db.update(vec![PushEditLogMotionPath(curve.points.len()*3), Pop])?;
            },
        }

        Ok(())
    }

    ///
    /// Inserts the values for a LayerEdit into the edit log (db must have an edit ID pushed. This will be popped when this returns)
    ///
    fn insert_layer_edit(&mut self, edit: &LayerEdit) -> Result<()> {
        use self::LayerEdit::*;

        match edit {
            Paint(when, paint_edit)         => {
                self.db.update(vec![PushEditLogWhen(*when)])?;
                self.insert_paint_edit(paint_edit)?;
            }

            Path(when, path_edit)           => {
                self.db.update(vec![PushEditLogWhen(*when)])?;
                self.insert_path_edit(path_edit)?;
            }

            AddKeyFrame(when)              => {
                self.db.update(vec![PushEditLogWhen(*when), Pop])?;
            }

            RemoveKeyFrame(when)           => {
                self.db.update(vec![PushEditLogWhen(*when), Pop])?;
            }

            SetName(new_name)              => {
                self.db.update(vec![PopEditLogString(0, new_name.clone())])?;
            },

            SetOrdering(at_index)           => {
                self.db.update(vec![PushEditLogInt(0, *at_index as i64), Pop])?;
            }
        }

        Ok(())
    }

    ///
    /// Inserts the values for a PaintEdit into the edit log (db must have an edit ID + a when value pushed. This will be popped when this returns)
    ///
    fn insert_paint_edit<'a>(&mut self, edit: &PaintEdit) -> Result<()> {
        use self::PaintEdit::*;

        match edit {
            &SelectBrush(ref id, ref definition, ref drawing_style) => {
                Self::insert_element_id(&mut self.db, id)?;
                Self::insert_brush(&mut self.db, definition)?;
                self.db.update(vec![PopEditLogBrush(DrawingStyleType::from(drawing_style))])?;
            },

            &BrushProperties(ref id, ref properties)                => {
                Self::insert_element_id(&mut self.db, id)?;
                Self::insert_brush_properties(&mut self.db, properties)?;
                self.db.update(vec![PopEditLogBrushProperties])?;
            },

            &BrushStroke(ref id, ref points)                        => {
                Self::insert_element_id(&mut self.db, id)?;
                self.db.update(vec![PushRawPoints(Arc::clone(points)), Pop])?;
            }
        }

        Ok(())
    }

    ///
    /// Inserts the values for a PathEdit into the edit log (db must have an edit ID + a when value pushed pushed)
    ///
    fn insert_path_edit<'a>(&mut self, edit: &PathEdit) -> Result<()> {
        use self::PathEdit::*;

        match edit {
            SelectBrush(id, definition, drawing_style)  => {
                Self::insert_element_id(&mut self.db, id)?;
                Self::insert_brush(&mut self.db, definition)?;
                self.db.update(vec![PopEditLogBrush(DrawingStyleType::from(drawing_style))])?;
            },

            BrushProperties(id, properties)             => {
                Self::insert_element_id(&mut self.db, id)?;
                Self::insert_brush_properties(&mut self.db, properties)?;
                self.db.update(vec![PopEditLogBrushProperties])?;
            },

            CreatePath(id, points)                      => {
                Self::insert_element_id(&mut self.db, id)?;
                self.db.update(vec![
                    PushPathComponents(Arc::clone(points)),
                    PushEditLogPath,
                    Pop
                ])?;
            }
        }

        Ok(())
    }
}
