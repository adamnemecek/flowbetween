use super::super::control_attribute::*;

///
/// Represents the controller attached to a particular control
/// 
#[derive(Clone, PartialEq)]
pub struct Controller(pub String);

impl ControlAttr for Controller { }
