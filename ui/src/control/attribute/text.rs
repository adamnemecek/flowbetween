use super::super::attribute_set::*;
use super::super::control_attribute::*;

// It's possible to attach a string to a control to give it some text (eg, when it's a label)
impl ControlAttr for String { 
    fn matches_attribute_in_set(&self, attributes: &AttributeSet) -> bool {
        Some(self) == attributes.get::<Self>()
    }
}
