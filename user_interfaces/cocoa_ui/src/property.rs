use flo_ui::*;

use objc::*;
use objc::rc::*;
use objc::runtime::*;
use cocoa::base::*;

use std::ops::Deref;

const UTF8_ENCODING: usize = 4;

///
/// Wrapper for the FloProperty bridging class
///
pub struct FloProperty {
    object: StrongPtr
}

impl From<PropertyValue> for FloProperty {
    fn from(val: PropertyValue) -> FloProperty {
        FloProperty::from(&val)
    }
}

impl From<&PropertyValue> for FloProperty {
    fn from(val: &PropertyValue) -> FloProperty {
        unsafe {
            // Initialise the FloProperty class (provided from the objective-C side)
            let obj: *mut Object = msg_send!(class!(FloProperty), alloc);

            // Initialise based on the property type
            use self::PropertyValue::*;
            let obj: *mut Object = match val {
                Nothing     => msg_send!(obj, init),
                Bool(val)   => msg_send!(obj, initWithBool: *val),
                Int(val)    => msg_send!(obj, initWithInt: (*val) as i64),
                Float(val)  => msg_send!(obj, initWithFloat: (*val) as f64),

                String(val) => {
                    let ns_string: *mut Object = msg_send!(class!(NSString), alloc);
                    let ns_string = msg_send!(ns_string, initWithBytes: val.as_ptr() length: val.len() encoding: UTF8_ENCODING as id);
                    let ns_string = StrongPtr::new(ns_string);

                    msg_send!(obj, initWithString: ns_string)
                }
            };

            // Create the final structure
            FloProperty {
                object: StrongPtr::new(obj)
            }
        }
    }
}

impl Deref for FloProperty {
    type Target = Object;

    fn deref(&self) -> &Object {
        unsafe { &**self.object }
    }
}