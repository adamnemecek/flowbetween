use super::super::control::*;
use super::super::control_attribute::*;

// A control can contain a list of subcomponents
impl ControlAttr for Vec<Control> {
    fn matches_attribute_in_control(&self, control: &Control) -> bool {
        true
    }
}
