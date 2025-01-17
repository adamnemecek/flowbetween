use super::event::*;
use super::update::*;
use super::htmlcontrol::*;
use super::canvas_update::*;

use ui::*;
use ui::session::*;
use canvas::*;
use binding::*;
use futures::*;
use futures::stream;
use itertools::join;
use percent_encoding::*;

use std::sync::*;

///
/// Converts a core user interface into a HTTP user interface
///
pub struct HttpUserInterface<CoreUi> {
    /// The core UI is the non-platform specific implementation of the user interface
    core_ui: Arc<CoreUi>,

    /// A binding ref for the UI tree (we need this for converting controller paths)
    ui_tree: BindRef<Control>,

    /// The base path of the instance (where URIs are generated relative to)
    base_path: String
}

impl<CoreUi: CoreUserInterface> HttpUserInterface<CoreUi> {
    ///
    /// Creates a new HTTP UI that will translate requests for the specified core UI
    ///
    pub fn new(ui: Arc<CoreUi>, base_path: String) -> HttpUserInterface<CoreUi> {
        let ui_tree = ui.ui_tree();

        HttpUserInterface {
            core_ui:    ui,
            ui_tree:    ui_tree,
            base_path:  base_path
        }
    }

    ///
    /// Retrieves the underlying non-platform specific UI object
    ///
    pub fn core(&self) -> Arc<CoreUi> {
        Arc::clone(&self.core_ui)
    }

    ///
    /// Converts an event from the HTTP side of things into a UI event
    ///
    fn http_event_to_core_event(http_event: Event) -> UiEvent {
        use Event::*;

        match http_event {
            NewSession      => UiEvent::Tick,
            UiRefresh       => UiEvent::Tick,
            Tick            => UiEvent::Tick,
            SuspendUpdates  => UiEvent::SuspendUpdates,
            ResumeUpdates   => UiEvent::ResumeUpdates,

            Action(controller_path, action_name, action_parameter) => UiEvent::Action(controller_path, action_name, action_parameter)
        }
    }

    ///
    /// Generates a new UI update (transforms a set of updates into the 'new HTML' update)
    ///
    /// The start update should come with an 'UpdateHtml' update that replaces the entire
    /// tree. We turn this into a NewUserInterfaceHtml update by combining it with the
    /// contents of the view model updates.
    ///
    fn new_ui_update(old_updates: Vec<Update>) -> Vec<Update> {
        let mut new_updates = vec![];

        // Collect all the viewmodel updates into one place
        let viewmodel_updates: Vec<_> = old_updates.iter()
            .flat_map(|update| match update {
                &Update::UpdateViewModel(ref view_model) => view_model.clone(),
                _ => vec![]
            }.into_iter())
            .collect();

        // Convert the updates in the old update
        for update in old_updates {
            match update {
                Update::UpdateHtml(html_diff) => {
                    if html_diff.len() == 1 && html_diff[0].address.len() == 0 {
                        // This should be converted into a new HTML event
                        new_updates.push(Update::NewUserInterfaceHtml(html_diff[0].new_html.clone(), html_diff[0].ui_tree.clone(), viewmodel_updates.clone()));
                    } else {
                        // Just treat this as a standard update
                        new_updates.push(Update::UpdateHtml(html_diff));
                    }
                },

                // Viewmodel updates should all wind up rolled into the new HTML update if we generate it
                // (Nothing will get generated normally)
                Update::UpdateViewModel(_) => (),

                // Everything else is left as-is
                update => new_updates.push(update)
            }
        }

        new_updates
    }

    ///
    /// Converts a canvas diff into a canvas update
    ///
    /// Mainly this means encoding the content of the update
    ///
    fn map_canvas_diff(canvas_diff: CanvasDiff) -> CanvasUpdate {
        // Encode the updates from the diff
        let mut encoded_updates = String::new();
        canvas_diff.updates.encode_canvas(&mut encoded_updates);

        // Create the HTTP version of the controller path
        let controller_path = join(canvas_diff.controller.iter()
            .map(|component| utf8_percent_encode(&*component, DEFAULT_ENCODE_SET)),
            "/");

        // Canvas name also needs to be encoded
        let canvas_name     = utf8_percent_encode(&canvas_diff.canvas_name, DEFAULT_ENCODE_SET).to_string();

        // Can now generate an update
        CanvasUpdate::new(controller_path, canvas_name, encoded_updates)
    }

    ///
    /// Maps a core UI diff into a HTML diff
    ///
    fn map_core_ui_diff(ui_diff: UiDiff, ui_tree: &Control, base_path: &str) -> HtmlDiff {
        // Fetch the properties of this difference
        let address         = ui_diff.address;
        let new_ui          = ui_diff.new_ui;
        let controller_path = html_controller_path_for_address(ui_tree, &address);
        let html            = new_ui.to_html_subcomponent(base_path, &controller_path);

        // Turn into a HTML diff
        HtmlDiff::new(address, &new_ui, html.to_string())
    }

    ///
    /// Maps a single core update to a HTTP update
    ///
    fn map_core_update(core_update: UiUpdate, base_path: &str, ui_tree: &Control) -> Vec<Update> {
        use self::UiUpdate::*;

        match core_update {
            Start => vec![],

            UpdateUi(core_diffs) => {
                // Map the UI differences
                vec![Update::UpdateHtml(core_diffs.into_iter()
                    .map(|core_ui_diff| Self::map_core_ui_diff(core_ui_diff, &ui_tree, &base_path))
                    .collect()
                )]
            },

            UpdateCanvas(canvas_diffs) => vec![Update::UpdateCanvas(canvas_diffs.into_iter().map(|diff| Self::map_canvas_diff(diff)).collect())],

            UpdateViewModel(view_model_diffs) => vec![Update::UpdateViewModel(view_model_diffs)]
        }
    }

    ///
    /// Converts updates from the core into HTTP updates
    ///
    fn core_updates_to_http_updates(core_update: Vec<UiUpdate>, base_path: &str, ui_tree: &Control) -> Vec<Update> {
        use self::UiUpdate::*;

        let is_start    = core_update.len() > 0 && core_update[0] == Start;
        let base_update = core_update.into_iter()
                .flat_map(|core_update| Self::map_core_update(core_update, base_path, ui_tree).into_iter())
                .collect();

        if is_start {
            // Generate the new UI HTML update
            Self::new_ui_update(base_update)
        } else {
            // Convert each update individually
            base_update
        }
    }

    ///
    /// Retrieves the controller used for this UI
    ///
    pub fn controller(&self) -> Arc<CoreUi::CoreController> {
        self.core_ui.controller()
    }
}

pub type HttpEventSink      = Box<dyn Sink<SinkItem=Vec<Event>, SinkError=()>+Send>;
pub type HttpUpdateStream   = Box<dyn Stream<Item=Vec<Update>, Error=()>+Send>;

impl<CoreUi: CoreUserInterface> UserInterface<Vec<Event>, Vec<Update>, ()> for HttpUserInterface<CoreUi> {
    type EventSink      = HttpEventSink;
    type UpdateStream   = HttpUpdateStream;

    fn get_input_sink(&self) -> Self::EventSink {
        // Get the core event sink
        let core_sink   = self.core_ui.get_input_sink();

        // Create a sink that turns HTTP events into core events
        let mapped_sink = core_sink.with_flat_map(|http_events: Vec<_>| {
            let core_events = http_events.into_iter()
                .map(|evt| Self::http_event_to_core_event(evt))
                .collect();
            stream::once(Ok(core_events))
        });

        // This new sink is our result
        Box::new(mapped_sink)
    }

    fn get_updates(&self) -> Self::UpdateStream {
        // Fetch the updates from the core
        let core_updates = self.core_ui.get_updates();

        // Fetch the extra components we need to map events from this object
        let ui_tree     = BindRef::clone(&self.ui_tree);
        let base_path   = self.base_path.clone();

        // Turn into HTTP updates
        let mapped_updates = core_updates.map(move |core_updates| {
            let ui_tree = ui_tree.get();

            Self::core_updates_to_http_updates(core_updates, &base_path, &ui_tree)
        });

        // These are the results
        Box::new(mapped_updates)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use serde_json::*;
    use futures::sync::oneshot;

    use std::time::Duration;
    use std::thread::*;

    struct TestController {
        ui: Binding<Control>,
    }

    impl Controller for TestController {
        fn ui(&self) -> BindRef<Control> {
            BindRef::new(&self.ui)
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    enum TestItem {
        Updates(Vec<Update>),
        Timeout
    }

    /// Creates a timeout future
    fn timeout(ms: u64) -> oneshot::Receiver<()> {
        let (timeout_send, timeout_recv) = oneshot::channel::<()>();

        spawn(move || {
            sleep(Duration::from_millis(ms));
            timeout_send.send(()).ok();
        });

        timeout_recv
    }

    #[test]
    fn generates_initial_update() {
        let controller      = TestController { ui: bind(Control::empty()) };
        let core_session    = Arc::new(UiSession::new(controller));
        let http_session    = HttpUserInterface::new(core_session, "test/session".to_string());

        let http_stream     = http_session.get_updates();

        let next_or_timeout     = http_stream.map(|updates| TestItem::Updates(updates)).select(timeout(2000).into_stream().map(|_| TestItem::Timeout).map_err(|_| ()));
        let mut next_or_timeout = executor::spawn(next_or_timeout);

        // First update should be munged into a NewUserInterfaceHtml update
        let first_update = next_or_timeout.wait_stream().unwrap();
        assert!(first_update != Ok(TestItem::Timeout));
        assert!(first_update == Ok(TestItem::Updates(vec![
            Update::NewUserInterfaceHtml("<flo-empty></flo-empty>".to_string(), json![{ "attributes": Vec::<String>::new(), "control_type": "Empty" }], vec![])
        ])));
    }
}
