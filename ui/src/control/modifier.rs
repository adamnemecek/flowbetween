use super::control::*;
use super::control_attribute::*;

///
/// Modifier that can be applied to a control
///
pub trait ControlModifier {
    /// Applies this modifier to a control
    fn modify(self, control: &mut Control);
}

impl<A: ControlModifier, B: ControlModifier> ControlModifier for (A, B) {
    fn modify(self, control: &mut Control) {
        self.0.modify(control);
        self.1.modify(control);
    }
}

impl<A: ControlModifier, B: ControlModifier, C: ControlModifier> ControlModifier for (A, B, C) {
    fn modify(self, control: &mut Control) {
        self.0.modify(control);
        self.1.modify(control);
        self.2.modify(control);
    }
}

impl<A: ControlModifier, B: ControlModifier, C: ControlModifier, D: ControlModifier> ControlModifier for (A, B, C, D) {
    fn modify(self, control: &mut Control) {
        self.0.modify(control);
        self.1.modify(control);
        self.2.modify(control);
        self.3.modify(control);
    }
}

impl<A: ControlModifier, B: ControlModifier, C: ControlModifier, D: ControlModifier, E: ControlModifier> ControlModifier for (A, B, C, D, E) {
    fn modify(self, control: &mut Control) {
        self.0.modify(control);
        self.1.modify(control);
        self.2.modify(control);
        self.3.modify(control);
        self.4.modify(control);
    }
}

impl<Attribute: ControlAttr> ControlModifier for Attribute {
    fn modify(self, control: &mut Control) {
        control.set_attribute(self)
    }
}
