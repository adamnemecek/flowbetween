use ui::*;

use std::sync::*;

///
/// The timeline allows the user to pick a point in time and create layers in the animation
///
pub struct TimelineController {
    view_model: Arc<NullViewModel>,
    ui:         Binding<Control>
}

impl TimelineController {
    pub fn new() -> TimelineController {
        let ui = bind(Control::empty());

        TimelineController {
            view_model: Arc::new(NullViewModel::new()),
            ui:         ui
        }
    }
}

impl Controller for TimelineController {
    fn ui(&self) -> Box<Bound<Control>> {
        Box::new(self.ui.clone())
    }

    fn get_viewmodel(&self) -> Arc<ViewModel> {
        self.view_model.clone()
    }
}
