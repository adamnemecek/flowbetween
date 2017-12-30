use super::traits::*;

use std::sync::*;

///
/// A `BindRef` references another binding without needing to know precisely
/// what kind of binding it is. It is read-only, so mostly useful for passing
/// a binding around, particularly for computed bindings. Create one with
/// `BindRef::from(binding)`.
/// 
/// Cloning a `BindRef` will create another reference to the same binding.
/// 
#[derive(Clone)]
pub struct BindRef<Target> {
    reference: Arc<Bound<Target>>
}

impl<Value> Bound<Value> for BindRef<Value> {
    #[inline]
    fn get(&self) -> Value {
        self.reference.get()
    }

    #[inline]
    fn clone_box(&self) -> Box<Bound<Value>> {
        self.reference.clone_box()
    }
}

impl<Value> Changeable for BindRef<Value> {
    #[inline]
    fn when_changed(&self, what: Arc<Notifiable>) -> Box<Releasable> {
        self.reference.when_changed(what)
    }
}

impl<Value> BindRef<Value> {
    ///
    /// Creates a new BindRef from an existing binding
    /// 
    pub fn from<Binding: 'static+Bound<Value>>(binding: Binding) -> BindRef<Value> {
        BindRef {
            reference: Arc::new(binding)
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::*;

    #[test]
    fn bindref_matches_core_value() {
        let mut bind    = bind(1);
        let bind_ref    = BindRef::from(bind.clone());

        assert!(bind_ref.get() == 1);

        bind.set(2);

        assert!(bind_ref.get() == 2);
    }
}