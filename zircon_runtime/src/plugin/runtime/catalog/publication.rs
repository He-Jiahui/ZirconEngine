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

    /// 仅当当前快照仍是候选所基于的同一 Arc 时替换，拒绝过期并发发布。
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
#[path = "tests/publication.rs"]
mod tests;
