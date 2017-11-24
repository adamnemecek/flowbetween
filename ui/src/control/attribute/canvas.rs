use super::super::super::canvas::*;
use super::super::super::resource_manager::*;
use super::super::control_attribute::*;

// A control can have an associated canvas if it is a target for arbitrary rendering
impl ControlAttr for Resource<Canvas> { }
