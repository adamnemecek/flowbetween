use super::super::control::*;
use super::super::attribute_set::*;
use super::super::control_attribute::*;

// A control can contain a list of subcomponents
impl ControlAttr for Vec<Control> {
    fn matches_attribute_in_set(&self, attributes: &AttributeSet) -> bool {
        // Subcomponents require recursion to match, so we always say two controls have the same subcomponent set
        true
    }
}
