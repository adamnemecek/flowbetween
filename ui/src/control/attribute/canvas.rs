use super::super::attribute_set::*;
use super::super::super::canvas::*;
use super::super::control_attribute::*;
use super::super::super::resource_manager::*;

// A control can have an associated canvas if it is a target for arbitrary rendering
impl ControlAttr for Resource<Canvas> { 
    fn matches_attribute_in_set(&self, attributes: &AttributeSet) -> bool {
        Some(self) == attributes.get::<Self>()
    }
}
