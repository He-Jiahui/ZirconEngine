use std::fmt::Debug;

use crate::core::framework::render::{SolariProviderAvailability, SolariRuntimeStatus};

pub trait SolariRuntimeProvider: Debug + Send + Sync {
    fn runtime_status(&self) -> SolariRuntimeStatus {
        SolariRuntimeStatus::Ready
    }

    fn runtime_status_message(&self) -> Option<&str> {
        None
    }

    fn availability(&self, provider_id: &str) -> SolariProviderAvailability {
        match self.runtime_status() {
            SolariRuntimeStatus::Ready => SolariProviderAvailability::ready(provider_id),
            SolariRuntimeStatus::Unavailable => SolariProviderAvailability::unavailable(
                provider_id,
                self.runtime_status_message()
                    .unwrap_or("Solari provider is unavailable"),
            ),
            _ => SolariProviderAvailability::unavailable(
                provider_id,
                self.runtime_status_message()
                    .unwrap_or("Solari provider did not report a ready runtime status"),
            ),
        }
    }
}
