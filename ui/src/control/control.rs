use super::types::*;
use super::modifier::*;
use super::attribute_set::*;
use super::control_attribute::*;
use super::attribute;

use super::super::canvas;
use super::super::diff::*;
use super::super::resource_manager::*;

use ControlType::*;

///
/// Represents a control
///
#[derive(Clone, PartialEq)]
pub struct Control {
    /// Attributes for this control
    attributes: AttributeSet,

    /// Type of this control
    control_type: ControlType
}

impl Control {
    /// Creates a new control of a particular type
    pub fn new(control_type: ControlType) -> Control {
        Control { attributes: AttributeSet::new(), control_type: control_type }
    }

    /// Creates a new container control
    pub fn container() -> Control {
        Self::new(Container)
    }

    /// Creates a new button control
    pub fn button() -> Control {
        Self::new(Button)
    }

    /// Creates a new label control
    pub fn label() -> Control {
        Self::new(Label)
    }

    /// Create a new empty control
    pub fn empty() -> Control {
        Self::new(Empty)
    }

    /// Creates a new canvas control
    pub fn canvas() -> Control {
        Self::new(ControlType::Canvas)
    }

    /// Sets (or updates) an attribute within this control
    pub fn set_attribute<T: 'static+ControlAttr+Clone>(&mut self, attribute: T) {
        self.attributes.set(attribute);
    }

    /// Creates a control with some attributes added to it
    pub fn with<T: ControlModifier>(self, modifier: T) -> Control {
        modifier.modify(&mut self);
        self
    }

    ///
    /// Creates a control with an added controller
    ///
    pub fn with_controller(&self, controller: &str) -> Control {
        self.with(attribute::Controller(String::from(controller)))
    }

    /// The type of this control
    pub fn control_type(&self) -> ControlType {
        self.control_type
    }

    ///
    /// True if any of the attributes of this control exactly match the specified attribute
    /// (using the rules of is_different_flat, so no recursion when there are subcomponents)
    ///
    pub fn has_attribute_flat<Attribute: ControlAttr>(&self, attr: &Attribute) -> bool {
        unimplemented!()
        /*
        self.attributes.iter()
            .any(|test_attr| !test_attr.is_different_flat(attr))
        */
    }

    ///
    /// If this control has a controller attribute, finds it
    ///
    pub fn controller<'a>(&'a self) -> Option<&'a str> {
        if let Some(&attribute::Controller(ref controller)) = self.attributes.get() {
            Some(&*controller)
        } else {
            None
        }
    }

    ///
    /// If this control has a controller attribute, finds it
    ///
    pub fn subcomponents<'a>(&'a self) -> Option<&'a Vec<Control>> {
        if let Some(subcomponents) = self.attributes.get::<Vec<Control>>() {
            Some(subcomponents)
        } else {
            None
        }
    }

    ///
    /// If this control has a canvas attribute, finds it
    ///
    pub fn canvas_resource<'a>(&'a self) -> Option<&Resource<canvas::Canvas>> {
        if let Some(canvas) = self.attributes.get::<Resource<canvas::Canvas>>() {
            Some(canvas)
        } else {
            None
        }
    }

    ///
    /// Finds the names of all of the controllers referenced by this control and its subcontrols
    ///
    pub fn all_controllers(&self) -> Vec<String> {
        let mut result = vec![];

        fn all_controllers(ctrl: &Control, result: &mut Vec<String>) {
            // Push the controller to the result if there is one
            if let Some(controller_name) = ctrl.controller() {
                result.push(String::from(controller_name));
            }

            // Go through the subcomponents as well
            if let Some(subcomponents) = ctrl.subcomponents() {
                for subcomponent in subcomponents.iter() {
                    all_controllers(subcomponent, result);
                }
            }
        }

        all_controllers(self, &mut result);

        // Remove duplicate controllers
        result.sort();
        result.dedup();

        result
    }

    ///
    /// Visits the control tree and performs a mapping function on each item
    ///
    pub fn map<TFn: Fn(&Control) -> Control>(&self, map_fn: &TFn) -> Control {
        // Map this control
        let mut new_control = map_fn(self);

        // Map the subcomponents of this control
        if let Some(subcomponents) = self.attributes.get::<Vec<Control>>() {
            // This control has subcomponents: map them and add them to the new control
            let new_subcomponents = vec![];

            for component in subcomponents.iter() {
                new_subcomponents.push(component.map(map_fn));
            }

            new_control.with(new_subcomponents)
        } else {
            // No subcomponents: just return the mapped control
            new_control
        }
    }
}

impl DiffableTree for Control {
    fn child_nodes<'a>(&'a self) -> Vec<&'a Self> {
        unimplemented!()
        /*
        self.attributes
            .iter()
            .map(|attr| attr.subcomponents().map(|component| component.iter()))
            .filter(|maybe_components| maybe_components.is_some())
            .flat_map(|components| components.unwrap())
            .collect()
            */
    }

    fn is_different(&self, compare_to: &Self) -> bool {
        unimplemented!()
        /*
        self.control_type() != compare_to.control_type()
            || self.attributes.iter().any(|attr| !compare_to.has_attribute_flat(attr))
        */
    }
}
