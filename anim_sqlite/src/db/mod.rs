use super::error::*;
use super::result::Result;

use flo_logging::*;
use flo_animation::*;

use desync::*;
use futures::*;
use rusqlite::*;

use std::mem;
use std::sync::*;
use std::ops::Range;
use std::collections::HashMap;

#[cfg(test)] mod tests;
#[cfg(test)] mod tests_smoke;

mod flo_store;
mod flo_query;
mod flo_sqlite;
mod db_enum;
mod edit_sink;
mod edit_stream;
mod insert_editlog;
mod animation;
mod animation_core;
mod color;
mod brush;
mod motion;
mod time_path;
mod vector_layer;
mod motion_path_type;
mod layer_cache;
pub mod vector_frame;

pub use self::animation::*;
pub use self::insert_editlog::*;
pub use self::vector_layer::*;
use self::animation_core::*;
use self::flo_sqlite::*;
use self::flo_store::*;
use self::flo_query::*;
use self::edit_stream::*;
use self::edit_sink::*;

///
/// Database used to store an animation
///
pub struct AnimationDb {
    /// The core contains details of the database
    core: Arc<Desync<AnimationDbCore<FloSqlite>>>,
}

impl AnimationDb {
    ///
    /// Creates a new animation database with an in-memory database
    ///
    pub fn new() -> AnimationDb {
        Self::new_from_connection(Connection::open_in_memory().unwrap())
    }

    ///
    /// Creates a new animation database using the specified SQLite connection
    ///
    pub fn new_from_connection(connection: Connection) -> AnimationDb {
        FloSqlite::setup(&connection).unwrap();

        let core    = Arc::new(Desync::new(AnimationDbCore::new(connection)));

        let db      = AnimationDb {
            core:   core
        };

        db
    }

    ///
    /// Creates an animation database that uses an existing database already set up in a SQLite connection
    ///
    pub fn from_connection(connection: Connection) -> AnimationDb {
        let core    = Arc::new(Desync::new(AnimationDbCore::new(connection)));

        let db = AnimationDb {
            core:   core,
        };

        db
    }

    ///
    /// If there has been an error, retrieves what it is and clears the condition
    ///
    pub fn retrieve_and_clear_error(&self) -> Option<SqliteAnimationError> {
        // We have to clear the error as rusqlite::Error does not implement clone or copy
        self.core.sync(|core| {
            core.retrieve_and_clear_error()
        })
    }

    ///
    /// Retrieves the number of edits in the animation
    ///
    pub fn get_num_edits(&self) -> Result<usize> {
        self.core.sync(|core| core.db.query_edit_log_length()).map(|length| length as usize)
    }

    ///
    /// Creates a stream for reading the specified range of elements from this animation
    ///
    pub fn read_edit_log(&self, range: Range<usize>) -> Box<dyn Stream<Item=AnimationEdit, Error=()>> {
        let edit_stream = EditStream::new(&self.core, range);

        Box::new(edit_stream)
    }

    ///
    /// Creates a sink for writing to the animation
    ///
    pub fn create_edit_sink(&self) -> Box<dyn Sink<SinkItem=Vec<AnimationEdit>, SinkError=()>+Send> {
        Box::new(EditSink::new(&self.core))
    }
}

impl AnimationDbCore<FloSqlite> {
    ///
    /// Creates a new database core with a sqlite connection
    ///
    fn new(connection: Connection) -> AnimationDbCore<FloSqlite> {
        // Query the database to warm up our cached values
        let mut db = FloSqlite::new(connection);

        // We begin assigning element IDs at the current length of the edit log
        let initial_element_id = db.query_edit_log_length().unwrap() as i64;

        // Generate the core
        let core = AnimationDbCore {
            log:                        Arc::new(LogPublisher::new(module_path!())),
            db:                         db,
            failure:                    None,
            cache_work:                 Arc::new(Desync::new(())),
            active_brush_for_layer:     HashMap::new(),
            layer_id_for_assigned_id:   HashMap::new(),
            path_properties_for_layer:  HashMap::new(),
            brush_properties_for_layer: HashMap::new(),
            next_element_id:            initial_element_id
        };

        core
    }
}

impl<TFile: FloFile+Send> AnimationDbCore<TFile> {
    ///
    /// If there has been an error, retrieves what it is and clears the condition
    ///
    fn retrieve_and_clear_error(&mut self) -> Option<SqliteAnimationError> {
        // We have to clear the error as rusqlite::Error does not implement clone or copy
        let mut failure = None;
        mem::swap(&mut self.failure, &mut failure);

        failure
    }
}
