use std::any::type_name;
use std::fmt::Debug;
use std::hash::Hash;

/// Marker contract for runtime-wide finite-state machine values.
pub trait StateSpec: 'static + Send + Sync + Clone + PartialEq + Eq + Hash + Debug {
    fn state_name() -> &'static str {
        type_name::<Self>()
    }
}

impl<T> StateSpec for T where T: 'static + Send + Sync + Clone + PartialEq + Eq + Hash + Debug {}
