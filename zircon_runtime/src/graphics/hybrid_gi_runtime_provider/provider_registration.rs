use std::fmt;
use std::sync::Arc;

use super::HybridGiRuntimeProvider;

#[derive(Clone)]
pub struct HybridGiRuntimeProviderRegistration {
    provider_id: String,
    provider: Arc<dyn HybridGiRuntimeProvider>,
}

impl HybridGiRuntimeProviderRegistration {
    pub fn new(provider_id: impl Into<String>, provider: Arc<dyn HybridGiRuntimeProvider>) -> Self {
        Self {
            provider_id: provider_id.into(),
            provider,
        }
    }

    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }

    pub fn provider(&self) -> &dyn HybridGiRuntimeProvider {
        self.provider.as_ref()
    }
}

impl fmt::Debug for HybridGiRuntimeProviderRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HybridGiRuntimeProviderRegistration")
            .field("provider_id", &self.provider_id)
            .finish_non_exhaustive()
    }
}
