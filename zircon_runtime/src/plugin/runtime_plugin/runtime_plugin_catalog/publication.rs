use std::fmt;
use std::sync::Arc;

use arc_swap::ArcSwap;

use super::RuntimePluginCatalogPreparedGeneration;
use super::{PluginCatalogGeneration, RuntimePluginCatalog, RuntimePluginCatalogSnapshot};

/// Owns the single lock-free publication point for runtime plugin catalog snapshots.
pub struct RuntimePluginCatalogAuthority {
    current: ArcSwap<RuntimePluginCatalogSnapshot>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePluginCatalogPublicationError {
    Conflict {
        expected: PluginCatalogGeneration,
        observed: PluginCatalogGeneration,
    },
}

impl RuntimePluginCatalogAuthority {
    pub fn from_catalog(initial: RuntimePluginCatalog) -> Self {
        Self::from_snapshot(Arc::new(RuntimePluginCatalogSnapshot::from_catalog(
            initial,
        )))
    }

    fn from_snapshot(initial: Arc<RuntimePluginCatalogSnapshot>) -> Self {
        Self {
            current: ArcSwap::from(initial),
        }
    }

    pub fn snapshot(&self) -> Arc<RuntimePluginCatalogSnapshot> {
        self.current.load_full()
    }

    pub fn publish(
        &self,
        prepared: RuntimePluginCatalogPreparedGeneration,
    ) -> Result<Arc<RuntimePluginCatalogSnapshot>, RuntimePluginCatalogPublicationError> {
        let (expected, candidate) = prepared.into_publication_parts();
        let expected_generation = expected.generation();

        let observed = self
            .current
            .compare_and_swap(&expected, Arc::clone(&candidate));
        if Arc::ptr_eq(&observed, &expected) {
            Ok(candidate)
        } else {
            Err(RuntimePluginCatalogPublicationError::Conflict {
                expected: expected_generation,
                observed: observed.generation(),
            })
        }
    }
}

impl fmt::Display for RuntimePluginCatalogPublicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Conflict { expected, observed } => write!(
                formatter,
                "runtime plugin catalog publication conflict: expected generation {expected}, observed {observed}"
            ),
        }
    }
}

impl std::error::Error for RuntimePluginCatalogPublicationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::{PluginPackageManifest, RuntimePluginRegistrationReport};

    #[test]
    fn publication_replaces_one_complete_successor_snapshot() {
        let authority =
            RuntimePluginCatalogAuthority::from_catalog(RuntimePluginCatalog::from_descriptors([]));
        let initial = authority.snapshot();
        let candidate = successor_generation(&initial, "first");
        let candidate_snapshot = Arc::clone(candidate.snapshot());

        let published = authority
            .publish(candidate)
            .expect("the exact successor should publish");

        assert!(Arc::ptr_eq(&published, &candidate_snapshot));
        assert!(Arc::ptr_eq(&authority.snapshot(), &candidate_snapshot));
        assert_eq!(initial.generation().get(), 1);
        assert_eq!(published.generation().get(), 2);
    }

    #[test]
    fn stale_publisher_cannot_replace_the_current_snapshot() {
        let authority =
            RuntimePluginCatalogAuthority::from_catalog(RuntimePluginCatalog::from_descriptors([]));
        let initial = authority.snapshot();
        let winner = successor_generation(&initial, "winner");
        let winner_snapshot = Arc::clone(winner.snapshot());
        let stale_candidate = successor_generation(&initial, "stale");
        authority
            .publish(winner)
            .expect("first successor should publish");

        let error = authority
            .publish(stale_candidate)
            .expect_err("stale expected handle must lose the compare-exchange");

        assert_eq!(
            error,
            RuntimePluginCatalogPublicationError::Conflict {
                expected: initial.generation(),
                observed: winner_snapshot.generation(),
            }
        );
        assert!(Arc::ptr_eq(&authority.snapshot(), &winner_snapshot));
    }

    fn successor_generation(
        base: &Arc<RuntimePluginCatalogSnapshot>,
        package_id: &str,
    ) -> RuntimePluginCatalogPreparedGeneration {
        let mut candidate = base.stage_update();
        candidate.append_registration(
            RuntimePluginRegistrationReport::from_native_package_manifest(
                PluginPackageManifest::new(package_id, package_id),
            ),
        );
        candidate.prepare().expect("valid candidate should prepare")
    }
}
