use super::super::style::*;
use super::super::model::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

use desync::*;
use futures::*;
use futures::executor;
use futures::executor::Spawn;

use std::time::Duration;

///
/// Controller that provides controls for adding/deleting/editing layers (generally displayed above the main layer list)
/// 
pub struct TimelineLayerControlsController {
    /// The UI for this controller
    ui: BindRef<Control>,

    /// The animation editing stream where this will send updates
    edit: Desync<Spawn<Box<dyn Sink<SinkItem=Vec<AnimationEdit>, SinkError=()>+Send>>>,

    /// The animation that this will edit
    animation: Box<dyn Animation>
}

impl TimelineLayerControlsController {
    ///
    /// Creates a new timeline layer controls controller
    /// 
    pub fn new<Anim: 'static+Animation+EditableAnimation>(model: &FloModel<Anim>) -> TimelineLayerControlsController {
        let ui          = Self::ui();
        let edit        = executor::spawn(model.edit());
        let animation   = Box::new(model.clone());

        TimelineLayerControlsController {
            ui:         ui,
            edit:       Desync::new(edit),
            animation:  animation
        }
    }

    ///
    /// Creates the UI for the layer controls controller
    /// 
    fn ui() -> BindRef<Control> {
        // Create the UI
        let ui = computed(move || {
            Control::container()
                .with(Bounds::fill_all())
                .with(vec![
                    Control::container()
                        .with(Font::Size(13.0))
                        .with(Font::Weight(FontWeight::ExtraBold))
                        .with(ControlAttribute::Padding((4, 1), (4, 1)))
                        .with(vec![
                            Control::empty()
                                .with(Bounds::stretch_horiz(1.0)),
                            Control::label()
                                .with(Bounds::next_horiz(14.0))
                                .with(TextAlign::Center)
                                .with((ActionTrigger::Click, "AddNewLayer"))
                                .with("+"),
                            Control::label()
                                .with(Bounds::next_horiz(14.0))
                                .with(TextAlign::Center)
                                .with((ActionTrigger::Click, "RemoveLayer"))
                                .with("-"),
                        ])
                        .with(Bounds::stretch_vert(1.0)),
                    Control::empty()
                        .with(Appearance::Background(TIMESCALE_BORDER))
                        .with(Bounds::next_vert(1.0))
                ])
        });

        // Turn into a bindref
        BindRef::from(ui)
    }
}

impl Controller for TimelineLayerControlsController {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }

    fn action(&self, action_id: &str, _action_parameter: &ActionParameter) {
        match action_id {
            "AddNewLayer" => {
                // Pick a layer ID for the new layer
                let new_layer_id = self.animation.get_layer_ids().into_iter().max().unwrap_or(0) + 1;

                // Send to the animation
                self.edit.sync(|animation| {
                    animation.wait_send(vec![
                        AnimationEdit::AddNewLayer(new_layer_id),
                        AnimationEdit::Layer(new_layer_id, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
                    ]).unwrap();
                });
            },

            _ => { }
        }
    }
}
