use std::fmt;
use std::sync::Arc;

use super::SolariRuntimeProvider;

#[derive(Clone)]
pub struct SolariRuntimeProviderRegistration {
    provider_id: String,
    priority: i32,
    provider: Arc<dyn SolariRuntimeProvider>,
}

impl SolariRuntimeProviderRegistration {
    pub fn new(provider_id: impl Into<String>, provider: Arc<dyn SolariRuntimeProvider>) -> Self {
        Self {
            provider_id: provider_id.into(),
            priority: 0,
            provider,
        }
    }

    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }

    pub const fn priority(&self) -> i32 {
        self.priority
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn provider(&self) -> &dyn SolariRuntimeProvider {
        self.provider.as_ref()
    }
}

impl fmt::Debug for SolariRuntimeProviderRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SolariRuntimeProviderRegistration")
            .field("provider_id", &self.provider_id)
            .field("priority", &self.priority)
            .finish_non_exhaustive()
    }
}
