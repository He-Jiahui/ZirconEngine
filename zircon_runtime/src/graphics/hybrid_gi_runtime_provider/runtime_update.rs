use super::HybridGiRuntimeStats;
use crate::graphics::runtime_provider::{define_runtime_provider_update, RuntimeProviderUpdate};

define_runtime_provider_update! {
    pub struct HybridGiRuntimeUpdate {
        stats: HybridGiRuntimeStats => copy;
    }
}
