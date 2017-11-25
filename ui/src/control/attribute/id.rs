use super::super::control::*;
use super::super::control_attribute::*;

///
/// Represents the ID of a particular control
/// 
#[derive(Clone, PartialEq)]
pub struct Id(pub String);

impl ControlAttr for Id {
    fn matches_attribute_in_control(&self, control: &Control) -> bool {
        Some(self) == control.get_attribute::<Self>()
    }
}
