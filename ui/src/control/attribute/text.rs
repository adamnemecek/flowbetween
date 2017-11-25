use super::super::control::*;
use super::super::control_attribute::*;

// It's possible to attach a string to a control to give it some text (eg, when it's a label)
impl ControlAttr for String { 
    fn matches_attribute_in_control(&self, control: &Control) -> bool {
        Some(self) == control.get_attribute::<Self>()
    }
}
