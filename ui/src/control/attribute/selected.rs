use super::super::control::*;
use super::super::super::property::*;
use super::super::control_attribute::*;

///
/// Attribute representing whether or not this control is selected
/// 
#[derive(Clone, PartialEq)]
pub struct Selected(pub Property);

impl ControlAttr for Selected { 
    fn matches_attribute_in_control(&self, control: &Control) -> bool {
        Some(self) == control.get_attribute::<Self>()
    }
}
