use super::super::control::*;
use super::super::control_attribute::*;

///
/// Represents the controller attached to a particular control
/// 
#[derive(Clone, PartialEq)]
pub struct Controller(pub String);

impl ControlAttr for Controller { 
    fn matches_attribute_in_control(&self, control: &Control) -> bool {
        Some(self) == control.get_attribute::<Self>()
    }
}
