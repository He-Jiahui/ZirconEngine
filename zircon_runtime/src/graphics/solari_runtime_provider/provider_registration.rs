use super::SolariRuntimeProvider;
use crate::graphics::runtime_provider::{
    RuntimeProviderRegistration, define_runtime_provider_registration,
};

define_runtime_provider_registration! {
    pub struct SolariRuntimeProviderRegistration for SolariRuntimeProvider;
}
