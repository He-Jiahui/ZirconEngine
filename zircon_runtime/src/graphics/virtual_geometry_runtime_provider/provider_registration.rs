use super::VirtualGeometryRuntimeProvider;
use crate::graphics::runtime_provider::{
    define_runtime_provider_registration, RuntimeProviderRegistration,
};

define_runtime_provider_registration! {
    pub struct VirtualGeometryRuntimeProviderRegistration for VirtualGeometryRuntimeProvider;
}
