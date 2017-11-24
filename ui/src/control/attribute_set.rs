use super::control_attribute::*;

use typemap::*;
use std::marker::PhantomData;

/// Key type used for referencing into our attribute set
struct Attribute<T> {
    phantom: PhantomData<T>
}

/// Specifies that the value type used in our attribute set is a ControlAttr object
impl<T: 'static+ControlAttr> Key for Attribute<T> {
    type Value = T;
}

///
/// A collection of attributes in a control 
///
#[derive(Clone)]
pub struct AttributeSet {
    /// The attributes that are in this set
    attributes: TypeMap<CloneAny+Send>
}

impl AttributeSet {
    ///
    /// Creates a new attribute set
    ///
    pub fn new() -> AttributeSet {
        AttributeSet { attributes: TypeMap::custom() }
    }

    ///
    /// Attempts to retrieve the attribute for this control of the specified type
    /// 
    pub fn get<T: 'static+Clone+ControlAttr>(&self) -> Option<&T> {
        self.attributes.get::<Attribute<T>>()
    }

    ///
    /// Adds an attribute to this attribute set
    /// 
    pub fn set<T: 'static+Clone+ControlAttr>(&mut self, val: T) {
        self.attributes.insert::<Attribute<T>>(val);
    }
}

impl PartialEq for AttributeSet {
    fn eq(&self, rhs: &AttributeSet) -> bool {
        // TODO: implement me
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone, PartialEq)]
    struct TestAttribute(String);
    impl ControlAttr for TestAttribute {}

    #[test]
    fn missing_attributes_are_none() {
        let attributes = AttributeSet::new();

        assert!(attributes.get::<TestAttribute>() == None);
    }

    #[test]
    fn can_get_and_set_attribute() {
        let mut attributes = AttributeSet::new();
        let test_attribute = TestAttribute("Hello".to_string());

        attributes.set(test_attribute);
        assert!(attributes.get() == Some(&TestAttribute("Hello".to_string())));
    }
}