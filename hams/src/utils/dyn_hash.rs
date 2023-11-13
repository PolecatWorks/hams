use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use std::any::Any;

/// A type safe implementation of self for any object to allow access/reference to any other object enabling Eq to work across types
pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}
impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Dynhash provides a trait which is object safe to map to the underlying Has
pub trait DynHash {
    fn dyn_hash(&self, state: &mut dyn Hasher);
}
impl<H: Hash + ?Sized> DynHash for H {
    fn dyn_hash(&self, mut state: &mut dyn Hasher) {
        self.hash(&mut state);
    }
}

/// An object safe trait to describe Eq against types and Any other type
pub trait DynEq {
    fn dyn_eq(&self, other: &dyn Any) -> bool;
}
impl<T: Eq + Any> DynEq for T {
    fn dyn_eq(&self, other: &dyn Any) -> bool {
        if let Some(other) = other.downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    /// As a supertrait we can use SubTraits ... BUT them must be object-safe. Therefore there must be implementations behind them
    ///
    /// DynHash provides an implementation against the objects Hash
    /// DynEq provides an implementation against the objects Eq
    /// AnyEq provides an implementation to compare Eq across types
    trait MyOuter: DynHash + DynEq + AsAny {}
    /// Implement Hash for MyOuter to allow Box to derive the Hash of this trait
    impl Hash for dyn MyOuter {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.dyn_hash(state);
        }
    }
    /// Implement PartialEq for MyOuter to allow Eq to derive the PartialEq for this trait
    impl PartialEq for dyn MyOuter {
        fn eq(&self, other: &dyn MyOuter) -> bool {
            DynEq::dyn_eq(self, other.as_any())
        }
    }
    /// Implement Eq to allow Box to derive Eq
    impl Eq for dyn MyOuter {}

    #[derive(Hash, PartialEq)]
    struct Min0 {
        name: String,
    }
    impl Eq for Min0 {}

    #[derive(Hash, PartialEq)]
    struct Min1 {
        name: String,
    }
    impl Eq for Min1 {}

    impl MyOuter for Min0 {}

    impl MyOuter for Min1 {}

    #[test]
    fn minimal() {
        let mut my_hs: HashSet<Box<dyn MyOuter>> = HashSet::new();
        my_hs.insert(Box::new(Min0 {
            name: "Min00".to_owned(),
        }));
        my_hs.insert(Box::new(Min1 {
            name: "Min00".to_owned(),
        }));

        // assert!(false);
    }
}
