use super::VirtualGeometryRuntimeStats;
use crate::graphics::runtime_provider::{RuntimeProviderUpdate, define_runtime_provider_update};

define_runtime_provider_update! {
    pub struct VirtualGeometryRuntimeUpdate {
        stats: VirtualGeometryRuntimeStats => ref;
    }
}
