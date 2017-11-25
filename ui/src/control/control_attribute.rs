use super::attribute_set::*;

///
/// Trait implemented by things that can be attached to controls as attributes 
///
pub trait ControlAttr : Send {
    ///
    /// Returns true if this attribute matches the one in the specified control 
    ///
    fn matches_attribute_in_set(&self, attributes: &AttributeSet) -> bool;
}
