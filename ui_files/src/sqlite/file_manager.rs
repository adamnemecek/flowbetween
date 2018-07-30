use super::file_list::*;
use super::super::file_update::*;
use super::super::file_manager::*;

use dirs;
use uuid::*;
use desync::*;
use rusqlite::*;
use futures::*;
use futures::executor;
use futures::sync::mpsc;

use std::fs;
use std::mem;
use std::sync::*;
use std::path::{Path, PathBuf};

const FILES_DB: &str = "files.db";
const DATA_DIR: &str = "data";

lazy_static! {
    // Prevents multiple threads from trying to create the database all at the same time
    static ref CREATING_DATABASE: Mutex<()> = Mutex::new(());
}

struct SqliteFileManagerCore {
    /// The database containing the list of files
    file_list: FileList,

    /// The senders for updates to this file manager
    updates: Vec<mpsc::Sender<FileUpdate>>
}

///
/// A file manager that uses Sqlite 
/// 
pub struct SqliteFileManager {
    /// Where we store our files
    root_path: PathBuf,

    /// The core of this file manager
    core: Desync<SqliteFileManagerCore>
}

impl SqliteFileManagerCore {
    ///
    /// Sends an update to everything that's listening for them
    /// 
    pub fn send_update(&mut self, update: FileUpdate) {
        // Swap out the updates (we'll push back the updates that are still alive when we're done)
        let mut updates = vec![];
        mem::swap(&mut updates, &mut self.updates);

        // Try to send to all of the update sinks
        for mut update_sink in updates.into_iter() {
            // Send the update to this sink
            let send_result = update_sink.try_send(update.clone());

            match send_result {
                // Send to this sink again if the update is successful
                Ok(_) => self.updates.push(update_sink),

                // Handle various error cases
                Err(try_err) => {
                    if try_err.is_full() {
                        // Wait using a normal executor if we've generated too many events
                        let mut sink_spawn  = executor::spawn(update_sink);
                        let send_result     = sink_spawn.wait_send(update.clone());

                        // If sent OK, keep sending to this sink
                        if send_result.is_ok() {
                            self.updates.push(sink_spawn.into_inner());
                        }
                    } else {
                        // Otherwise, don't update this sink next time (assume it's disconnected)
                    }
                }
            }
            
        }
    }
}

impl SqliteFileManager {
    ///
    /// Creates a new Sqlite file manager (in a sub-path of the main files directory)
    /// 
    /// Separate sub-paths can be used to allow for multi-user scenarios: in single-user
    /// scenarios we usually set this to `"default"`.
    /// 
    pub fn new(application_path: &str, sub_path: &str) -> SqliteFileManager {
        let _creating = CREATING_DATABASE.lock().unwrap();

        // This will be the 'root' data directory for the user
        let mut root_path = dirs::data_local_dir()
            .or_else(|| dirs::data_dir())
            .unwrap();

        // Append the path components
        root_path.push(application_path);
        root_path.push(sub_path);

        // Create the data directory if it does not exist
        fs::create_dir_all(root_path.as_path()).unwrap();

        // Create the subdirectories too
        let mut data_dir = root_path.clone();
        data_dir.push(DATA_DIR);
        fs::create_dir_all(data_dir.as_path()).unwrap();

        // Check for the file list database file
        let mut database_file = root_path.clone();
        database_file.push(FILES_DB);

        // Connect to the Sqlite database
        let database_file_exists    = database_file.is_file();
        let database_connection     = Connection::open(database_file.as_path()).unwrap();
        let file_list               = FileList::new(database_connection);

        if !database_file_exists {
            file_list.initialize().unwrap();
        }

        // Put together the file manager
        SqliteFileManager {
            root_path:  root_path,
            core:       Desync::new(SqliteFileManagerCore {
                file_list:  file_list,
                updates:    vec![]
            })
        }
    }

    ///
    /// Finds the path to request from the file list for a particular file path
    /// 
    fn file_list_path(&self, path: &Path) -> Option<PathBuf> {
        // Construct a path representing where we store our data
        let mut data_path = self.root_path.clone();
        data_path.push(DATA_DIR);

        if path.components().count() == 1 && path.is_relative() {

            // A single relative component is left intact
            path.components().last()
                .map(|component| component.as_os_str().to_string_lossy())
                .map(|component| {
                    let mut buf = PathBuf::new();
                    buf.push(component.to_string());
                    buf
                })

        } else if path.starts_with(data_path) {

            // If the path is in the data path, then use the last component
            // TODO: this isn't quite right if the path is in a subdirectory
            path.components().last()
                .map(|component| component.as_os_str().to_string_lossy())
                .map(|component| {
                    let mut buf = PathBuf::new();
                    buf.push(component.to_string());
                    buf
                })

        } else {
            None
        }
    }
}

impl FileManager for SqliteFileManager {
    ///
    /// Returns a list of all the files that can be opened by this manager
    /// 
    fn get_all_files(&self) -> Vec<PathBuf> {
        // Retrieve from the file list and append the folder we're using
        self.core.sync(|core| core.file_list.list_paths())
            .into_iter()
            .map(|last_component| {
                let mut full_path = self.root_path.clone();
                full_path.push(DATA_DIR);
                full_path.push(last_component);
                full_path
            })
            .collect()
    }

    ///
    /// Returns the display name for a particular path
    /// 
    fn display_name_for_path(&self, path: &Path) -> Option<String> {
        let path = self.file_list_path(path);

        if let Some(path) = path {
            self.core.sync(|core| core.file_list.display_name_for_path(path.as_path()))
        } else {
            None
        }
    }

    ///
    /// Reserves a path for a new file (this path is valid and won't be re-used by future calls but
    /// no files will exist here yet)
    /// 
    fn create_new_path(&self) -> PathBuf {
        // Generate a filename
        let filename        = Uuid::new_v4().simple().to_string();
        let mut full_path   = self.root_path.clone();

        full_path.push(DATA_DIR);
        full_path.push(&filename);

        let update          = FileUpdate::NewFile(full_path.clone());

        // Add to the database
        let mut filename_buf = PathBuf::new();
        filename_buf.push(filename);
        self.core.async(move |core| {
            core.file_list.add_path(filename_buf.as_path());
            core.send_update(update);
        });

        // Result is the full path
        full_path
    }

    ///
    /// Updates or creates the display name associated with a particular path (which must be
    /// returned via get_all_files: setting the name for a non-existent path will just
    /// result)
    ///
    fn set_display_name_for_path(&self, full_path: &Path, display_name: String) {
        let path = self.file_list_path(full_path);

        if let Some(path) = path {
            let update = FileUpdate::SetDisplayName(PathBuf::from(full_path), display_name.clone());

            self.core.async(move |core| {
                core.file_list.set_display_name_for_path(path.as_path(), &display_name);
                core.send_update(update);
            });
        }
    }

    ///
    /// Returns a stream of updates indicating changes made to the file manager
    /// 
    fn update_stream(&self) -> Box<dyn Stream<Item=FileUpdate, Error=()>+Send> {
        // Create the channel for sending updates
        let (sender, receiver) = mpsc::channel(10);

        // Store the sender in the core
        self.core.async(move |core| core.updates.push(sender));

        // Result is the receiver
        Box::new(receiver)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_new_path() {
        let test_files  = SqliteFileManager::new("app.flowbetween.test", "default");
        let new_path    = test_files.create_new_path();

        assert!(new_path.components().count() > 3);
    }

    #[test]
    fn display_name_is_initially_none() {
        let test_files  = SqliteFileManager::new("app.flowbetween.test", "default");
        let new_path    = test_files.create_new_path();

        assert!(test_files.display_name_for_path(new_path.as_path()) == None);
    }

    #[test]
    fn set_alternative_display_name() {
        let test_files  = SqliteFileManager::new("app.flowbetween.test", "default");
        let new_path    = test_files.create_new_path();

        test_files.set_display_name_for_path(new_path.as_path(), "Test display name".to_string());
        assert!(test_files.display_name_for_path(new_path.as_path()) == Some("Test display name".to_string()));
    }

    #[test]
    fn will_send_updates_to_stream() {
        let test_files          = SqliteFileManager::new("app.flowbetween.test", "default");
        let mut update_stream   = executor::spawn(test_files.update_stream());

        let new_path            = test_files.create_new_path();
        test_files.set_display_name_for_path(new_path.as_path(), "Another display name".to_string());

        assert!(update_stream.wait_stream() == Some(Ok(FileUpdate::NewFile(new_path.clone()))));
        assert!(update_stream.wait_stream() == Some(Ok(FileUpdate::SetDisplayName(new_path.clone(), "Another display name".to_string()))));
    }
}