use super::super::attribute_set::*;
use super::super::control_attribute::*;

///
/// Represents the ID of a particular control
/// 
#[derive(Clone, PartialEq)]
pub struct Id(pub String);

impl ControlAttr for Id {
    fn matches_attribute_in_set(&self, attributes: &AttributeSet) -> bool {
        Some(self) == attributes.get::<Self>()
    }
}
