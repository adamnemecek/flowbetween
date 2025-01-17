use super::super::menu::*;
use super::super::tools::*;
use super::super::model::*;

use flo_ui::*;
use flo_canvas::*;
use flo_binding::*;
use flo_animation::*;
use flo_animation::raycast::*;
use flo_curves::bezier::path::algorithms::*;

use futures::*;
use itertools::*;

use std::iter;
use std::sync::*;

///
/// Model for the flood-fill tool
///
pub struct FloodFillModel {
    /// The opacity of the next flood fill that will be added
    pub opacity: Binding<f32>,

    /// The color of the next flood fill that will be added
    pub color: Binding<Color>
}

///
/// Data passed through to the flood-fill tool
///
#[derive(Clone, PartialEq, Debug)]
pub struct FloodFillData {
    /// The properties to use when drawing flood-fills
    pub brush_properties: BrushProperties
}

///
/// A tool for flood-filling areas of the canvas
///
pub struct FloodFill {

}

impl FloodFillModel {
    ///
    /// Creates the default flood-fill model
    ///
    pub fn new() -> FloodFillModel {
        FloodFillModel {
            opacity:    bind(1.0),
            color:      bind(Color::Rgba(0.0, 0.6, 0.35, 1.0))
        }
    }
}

impl FloodFill {
    ///
    /// Creates a new flood-fill tool
    ///
    pub fn new() -> FloodFill {
        FloodFill {
        }
    }

    ///
    /// Generates the actions for a flood fill operation
    ///
    pub fn flood_fill<Anim: 'static+Animation>(&self, model: Arc<FloModel<Anim>>, center_point: (f32, f32), data: &FloodFillData) -> impl Iterator<Item=ToolAction<FloodFillData>> {
        // Turn the x, y coordinates into a pathpoint
        let (x, y)          = center_point;
        let center_point    = PathPoint::new(x, y);

        // Get the current frame information
        let when            = model.timeline().current_time.get();
        let layer           = model.timeline().selected_layer.get();
        let frame           = model.frame().frame.get();

        if let (Some(frame), Some(layer)) = (frame, layer) {
            // Generate a ray-casting function from it
            let ray_casting_fn  = vector_frame_raycast(&frame);

            // Attempt to generate a path element by flood-filling
            let fill_path       = flood_fill_convex(center_point, &FillOptions::default(), ray_casting_fn);

            if let Some(fill_path) = fill_path {
                // Create a new path element for this fill path
                let fill_path: Path = fill_path;

                let brush_defn      = BrushDefinition::Ink(InkDefinition::default());
                let mut brush_props = data.brush_properties.clone();

                // Generate the editing actions to create this fill path
                let actions         = vec![
                    PathEdit::SelectBrush(ElementId::Unassigned, brush_defn, BrushDrawingStyle::Draw),
                    PathEdit::BrushProperties(ElementId::Unassigned, brush_props),
                    PathEdit::CreatePath(ElementId::Unassigned, Arc::new(fill_path.elements().collect()))
                ];
                let actions = actions.into_iter()
                    .map(move |action| LayerEdit::Path(when, action))
                    .map(move |action| AnimationEdit::Layer(layer, action))
                    .map(|action| ToolAction::Edit(action))
                    .chain(vec![
                        ToolAction::BrushPreview(BrushPreviewAction::Layer(layer)),
                        ToolAction::BrushPreview(BrushPreviewAction::UnsetProperties),
                        ToolAction::InvalidateFrame
                    ]);

                Either::Left(actions)
            } else {
                Either::Right(iter::empty())
            }
        } else {
            Either::Right(iter::empty())
        }
    }
}

impl<Anim: 'static+Animation> Tool<Anim> for FloodFill {
    type ToolData   = FloodFillData;
    type Model      = FloodFillModel;

    fn tool_name(&self) -> String { "Flood Fill".to_string() }

    fn image_name(&self) -> String { "floodfill".to_string() }

    fn create_model(&self, _flo_model: Arc<FloModel<Anim>>) -> FloodFillModel {
        FloodFillModel::new()
    }

    fn create_menu_controller(&self, _flo_model: Arc<FloModel<Anim>>, tool_model: &FloodFillModel) -> Option<Arc<dyn Controller>> {
        let color   = tool_model.color.clone();
        let opacity = tool_model.opacity.clone();

        Some(Arc::new(FloodFillMenuController::new(color, opacity)))
    }

    fn actions_for_model(&self, _flo_model: Arc<FloModel<Anim>>, tool_model: &FloodFillModel) -> Box<dyn Stream<Item=ToolAction<FloodFillData>, Error=()>+Send> {
        // Compute brush properties from the model
        let color               = tool_model.color.clone();
        let opacity             = tool_model.opacity.clone();
        let brush_properties    = computed(move || {
            BrushProperties {
                size:       1.0,
                opacity:    opacity.get(),
                color:      color.get()
            }
        });

        // Compute the data from that
        let fill_data           = computed(move || {
            FloodFillData {
                brush_properties: brush_properties.get()
            }
        });

        // Turn the computed values into a stream and update the brush whenever the values change
        Box::new(follow(fill_data).map(|fill_data| ToolAction::Data(fill_data)))
    }

    fn actions_for_input<'a>(&'a self, flo_model: Arc<FloModel<Anim>>, data: Option<Arc<FloodFillData>>, input: Box<dyn 'a+Iterator<Item=ToolInput<FloodFillData>>>) -> Box<dyn Iterator<Item=ToolAction<FloodFillData>>> {
        Box::new(
            input.flat_map(move |action| {
                let actions : Box<dyn Iterator<Item=ToolAction<FloodFillData>>> =
                    match action {
                        ToolInput::Paint(painting) => {
                            match painting.action {
                                PaintAction::Finish => {
                                    // Perform the flood-fill action when the painting finishes
                                    Box::new(self.flood_fill(Arc::clone(&flo_model), painting.location, &*(data.clone().unwrap())))
                                },

                                _ => {
                                    // Nothing to do for other paint actions
                                    Box::new(vec![].into_iter())
                                }
                            }
                        },

                        _ => {
                            // No action for other kinds of input input
                            Box::new(vec![].into_iter())
                        }
                    };

                actions
            })
            .collect::<Vec<_>>()
            .into_iter()
        )
    }
}
