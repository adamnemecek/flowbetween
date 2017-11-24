use super::super::control_attribute::*;
use super::super::super::control::*;

// A control can contain a list of subcomponents
impl ControlAttr for Vec<Control> { }
