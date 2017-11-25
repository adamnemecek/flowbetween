use super::super::bounds::*;
use super::super::control::*;
use super::super::control_attribute::*;

// A control can have a bounding box to specify its coordinates relative to other controls
impl ControlAttr for Bounds {
    fn matches_attribute_in_control(&self, control: &Control) -> bool {
        Some(self) == control.get_attribute::<Self>()
    }
}
