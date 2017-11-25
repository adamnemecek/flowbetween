use super::super::actions::*;
use super::super::attribute_set::*;
use super::super::control_attribute::*;

///
/// Represents the events that are triggered for particular control actions
/// 
#[derive(Clone, PartialEq)]
pub struct Action(pub ActionTrigger, String);

// Actions are stored in a vector as controls can have more than one
impl ControlAttr for Vec<Action> { 
    fn matches_attribute_in_set(&self, attributes: &AttributeSet) -> bool {
        Some(self) == attributes.get::<Self>()
    }
}
