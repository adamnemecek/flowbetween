use super::super::control::*;
use super::super::super::canvas::*;
use super::super::control_attribute::*;
use super::super::super::resource_manager::*;

// A control can have an associated canvas if it is a target for arbitrary rendering
impl ControlAttr for Resource<Canvas> { 
    fn matches_attribute_in_control(&self, control: &Control) -> bool {
        return Some(self) == control.get_attribute::<Self>();
    }
}
