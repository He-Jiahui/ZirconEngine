use std::fmt;
use std::sync::Arc;

use super::VirtualGeometryRuntimeProvider;

#[derive(Clone)]
pub struct VirtualGeometryRuntimeProviderRegistration {
    provider_id: String,
    priority: i32,
    provider: Arc<dyn VirtualGeometryRuntimeProvider>,
}

impl VirtualGeometryRuntimeProviderRegistration {
    pub fn new(
        provider_id: impl Into<String>,
        provider: Arc<dyn VirtualGeometryRuntimeProvider>,
    ) -> Self {
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

    pub fn provider(&self) -> &dyn VirtualGeometryRuntimeProvider {
        self.provider.as_ref()
    }
}

impl fmt::Debug for VirtualGeometryRuntimeProviderRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VirtualGeometryRuntimeProviderRegistration")
            .field("provider_id", &self.provider_id)
            .field("priority", &self.priority)
            .finish_non_exhaustive()
    }
}
