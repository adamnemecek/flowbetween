use super::super::traits::*;
use super::super::deref_map::*;

use std::time::Duration;
use std::ops::Deref;
use std::sync::*;

///
/// A keyframe in a vector layer
/// 
pub struct VectorKeyFrame {
    core: Mutex<VectorKeyFrameCore>
}

impl VectorKeyFrame {
    ///
    /// Creates a new vector key frame
    /// 
    pub fn new(start_time: Duration) -> VectorKeyFrame {
        VectorKeyFrame {
            core: Mutex::new(VectorKeyFrameCore::new(start_time))
        }
    }

    ///
    /// The start time of this key frame
    /// 
    pub fn start_time(&self) -> Duration {
        self.core.lock().unwrap().start_time()
    }

    ///
    /// Adds a new element to the front of the vector
    /// 
    pub fn add_element(&self, when: Duration, new_element: Box<VectorElement>) {
        self.core.lock().unwrap().add_element(when, new_element);
    }

    ///
    /// Retrieves the elements in this keyframe
    /// 
    pub fn elements<'a>(&'a self) -> Box<'a+Deref<Target=Vec<(Duration, Box<VectorElement>)>>> {
        let core            = self.core.lock().unwrap();

        let elements = DerefMap::map(core, |core| &core.elements);

        Box::new(elements)
    }

    ///
    /// Retrieves the properties that will be applied to the next element added to this keyframe
    /// 
    #[inline]
    pub fn active_properties(&self) -> VectorProperties {
        self.core.lock().unwrap().active_properties().clone()
    }
}

///
/// Data storage for a vector keyframe
/// 
struct VectorKeyFrameCore {
    /// When this frame starts
    start_time: Duration,

    /// The elements in this key frame (ordered from back to front)
    elements: Vec<(Duration, Box<VectorElement>)>,

    /// The properties that will apply to the next element added to this core
    active_properties: VectorProperties
}

impl VectorKeyFrameCore {
    ///
    /// Creates a new vector key frame
    /// 
    pub fn new(start_time: Duration) -> VectorKeyFrameCore {
        VectorKeyFrameCore {
            start_time:         start_time,
            elements:           vec![],
            active_properties:  VectorProperties::default()
        }
    }

    ///
    /// The start time of this key frame
    /// 
    #[inline]
    pub fn start_time(&self) -> Duration {
        self.start_time
    }

    ///
    /// Retrieves the properties that will be applied to the next element added to this keyframe
    /// 
    #[inline]
    pub fn active_properties<'a>(&'a self) -> &'a VectorProperties {
        &self.active_properties
    }

    ///
    /// Adds a new element to the front of the vector
    /// 
    #[inline]
    pub fn add_element(&mut self, when: Duration, new_element: Box<VectorElement>) {
        new_element.update_properties(&mut self.active_properties);
        self.elements.push((when, new_element));
    }
}
