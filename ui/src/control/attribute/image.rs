use super::super::super::image::*;
use super::super::attribute_set::*;
use super::super::control_attribute::*;
use super::super::super::resource_manager::*;

// Can attach an image resource to a control
impl ControlAttr for Resource<Image> {
    fn matches_attribute_in_set(&self, attributes: &AttributeSet) -> bool {
        Some(self) == attributes.get::<Self>()
    }
}
