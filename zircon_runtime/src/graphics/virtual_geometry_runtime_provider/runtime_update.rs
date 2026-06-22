use super::VirtualGeometryRuntimeStats;
use crate::graphics::runtime_provider::{define_runtime_provider_update, RuntimeProviderUpdate};

define_runtime_provider_update! {
    pub struct VirtualGeometryRuntimeUpdate {
        stats: VirtualGeometryRuntimeStats => ref;
    }
}
