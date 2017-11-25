use super::super::bounds::*;
use super::super::attribute_set::*;
use super::super::control_attribute::*;

// A control can have a bounding box to specify its coordinates relative to other controls
impl ControlAttr for Bounds {
    fn matches_attribute_in_set(&self, attributes: &AttributeSet) -> bool {
        Some(self) == attributes.get::<Self>()
    }
}
