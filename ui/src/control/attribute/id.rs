use super::super::control_attribute::*;

///
/// Represents the ID of a particular control
/// 
#[derive(Clone, PartialEq)]
pub struct Id(pub String);

impl ControlAttr for Id { }
