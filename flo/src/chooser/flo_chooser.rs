use super::consts::*;
use super::super::model::*;
use super::super::editor::*;

use flo_animation::*;
use flo_ui_files::*;
use flo_ui_files::ui::*;
use flo_ui_files::sqlite::*;

use std::sync::*;

///
/// The default file chooser for FlowBetween
///
pub struct FloChooser<Anim: 'static+EditableAnimation+FileAnimation> {
    /// The file manager managed by this chooser
    file_manager: Arc<SqliteFileManager>,

    /// The shared open file store for this animation
    file_store: Arc<OpenFileStore<FloSharedModel<Anim>>>
}

impl<Anim: 'static+EditableAnimation+FileAnimation> FloChooser<Anim> {
    ///
    /// Creates a new chooser
    ///
    pub fn new() -> FloChooser<Anim> {
        // Create the file manager (we use a single default user by default)
        let file_manager = Arc::new(SqliteFileManager::new(APP_NAME, DEFAULT_USER_FOLDER));

        // Create the file store
        let file_store = Arc::new(OpenFileStore::new());

        // Put everything together
        FloChooser {
            file_manager:   file_manager,
            file_store:     file_store
        }
    }
}

impl<Anim: 'static+EditableAnimation+FileAnimation> FileChooser for FloChooser<Anim> {
    /// The controller that edits/displays open files
    type Controller = EditorController<Anim>;

    /// The file manager that finds paths where files can be located
    type FileManager = SqliteFileManager;

    ///
    /// Retrieves the file manager for this file chooser
    ///
    fn get_file_manager(&self) -> Arc<Self::FileManager> {
        Arc::clone(&self.file_manager)
    }

    ///
    /// Retrieves the shared file store for this chooser
    ///
    fn get_file_store(&self) -> Arc<OpenFileStore<FloSharedModel<Anim>>> {
        Arc::clone(&self.file_store)
    }
}
