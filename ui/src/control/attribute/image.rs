use super::super::control::*;
use super::super::super::image::*;
use super::super::control_attribute::*;
use super::super::super::resource_manager::*;

// Can attach an image resource to a control
impl ControlAttr for Resource<Image> {
    fn matches_attribute_in_control(&self, control: &Control) -> bool {
        Some(self) == control.get_attribute::<Self>()
    }
}
