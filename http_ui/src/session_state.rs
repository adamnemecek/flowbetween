use ui::*;

use uuid::*;
use std::sync::*;

///
/// The core of the session state
///
struct SessionStateCore {
    /// The tree attached to this state
    ui_tree: Box<Bound<Control>>,

    /// The previous state of the tree
    previous_tree: Binding<Option<Control>>,

    /// Lifetime of the watcher that
    ui_tree_watcher_lifetime: Box<Releasable>,

    /// Binding that specifies whether or not the tree has changed
    tree_has_changed: Binding<bool>
}

///
/// The session state object represents the stored state of a particular session
///
pub struct SessionState {
    /// A string identifying this session
    session_id: String,

    /// The core of the state
    core: Mutex<SessionStateCore>
}

impl SessionState {
    ///
    /// Creates a new session state
    ///
    pub fn new() -> SessionState {
        let session_id                      = Uuid::new_v4().simple().to_string();
        let mut tree: Box<Bound<Control>>   = Box::new(bind(Control::container()));
        let has_changed                     = bind(false);
        let watcher_lifetime                = Self::watch_tree(&mut tree, has_changed.clone());

        let core = SessionStateCore {
            ui_tree:                    tree,
            previous_tree:              bind(None),
            ui_tree_watcher_lifetime:   watcher_lifetime,
            tree_has_changed:           has_changed
        };

        SessionState { 
            session_id: session_id,
            core:       Mutex::new(core)
        }
    }

    ///
    /// Sets has_changed to true when the ui_tree changes
    ///
    fn watch_tree(ui_tree: &mut Box<Bound<Control>>, mut has_changed: Binding<bool>) -> Box<Releasable> {
        ui_tree.when_changed(notify(move || has_changed.set(true)))
    }

    ///
    /// Retrieves the ID of this session
    ///
    pub fn id(&self) -> String {
        self.session_id.clone()
    }

    ///
    /// Replaces the UI tree in this session
    ///
    pub fn set_ui_tree<TBinding: 'static+Bound<Control>>(&self, new_tree: TBinding) {
        let mut core = self.core.lock().unwrap();

        // Stop watching the old tree
        core.ui_tree_watcher_lifetime.done();

        // Store the new UI tree in this object
        core.ui_tree = Box::new(new_tree);

        // Whenever the new tree is changed, set the has_changed binding
        let mut has_changed = core.tree_has_changed.clone();
        has_changed.set(true);
        core.ui_tree_watcher_lifetime = Self::watch_tree(&mut core.ui_tree, has_changed);
    }

    ///
    /// Retrieves the current state of the UI for this session
    ///
    pub fn entire_ui_tree(&self) -> Control {
        let core = self.core.lock().unwrap();

        core.ui_tree.get()
    }
}

impl Drop for SessionState {
    fn drop(&mut self) {
        let mut core = self.core.lock().unwrap();

        core.ui_tree_watcher_lifetime.done();
    }
}
