use super::*;

use animation::*;

use futures::*;
use std::ops::Range;
use std::time::Duration;

impl AnimationMotion for SqliteAnimation {
    fn get_motion_ids(&self, when: Range<Duration>) -> Box<Stream<Item=ElementId, Error=()>> {
        unimplemented!()
    }

    fn get_motion(&self, motion_id: ElementId) -> Option<Motion> {
        self.db.get_motion(motion_id)
    }

    fn get_motions_for_element(&self, element_id: ElementId) -> Vec<ElementId> {
        self.db.get_motions_for_element(element_id)
    }
}
