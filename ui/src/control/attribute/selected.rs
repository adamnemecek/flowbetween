use super::super::super::property::*;
use super::super::control_attribute::*;

///
/// Attribute representing whether or not this control is selected
/// 
#[derive(Clone, PartialEq)]
pub struct Controller(pub Property);

impl ControlAttr for Controller { }
