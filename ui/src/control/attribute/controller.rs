use super::super::attribute_set::*;
use super::super::control_attribute::*;

///
/// Represents the controller attached to a particular control
/// 
#[derive(Clone, PartialEq)]
pub struct Controller(pub String);

impl ControlAttr for Controller { 
    fn matches_attribute_in_set(&self, attributes: &AttributeSet) -> bool {
        Some(self) == attributes.get::<Self>()
    }
}
