use super::VirtualGeometryRuntimeProvider;
use crate::graphics::runtime_provider::{
    RuntimeProviderRegistration, define_runtime_provider_registration,
};

define_runtime_provider_registration! {
    pub struct VirtualGeometryRuntimeProviderRegistration for VirtualGeometryRuntimeProvider;
}
