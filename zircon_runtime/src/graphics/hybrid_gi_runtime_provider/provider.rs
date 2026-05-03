use std::fmt::Debug;

use super::HybridGiRuntimeState;

pub trait HybridGiRuntimeProvider: Debug + Send + Sync {
    fn create_state(&self) -> Box<dyn HybridGiRuntimeState>;
}
