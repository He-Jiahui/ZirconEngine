//! Plugin discovery inputs and errors owned by the catalog publication boundary.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::super::admission::EditorPluginCatalogAdmissionError;
use super::super::catalog::EditorPluginCatalog;
use super::super::phases::EditorPluginLoadingPhase;
use super::super::sdk::lifecycle::EditorPluginLifecycleStage;

/// The discovery authority for an editor plugin package.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EditorPluginSource {
    Builtin,
    Project,
    PackageManifest,
}

/// One upstream discovery result consumed when a manager publishes its first generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditorPluginDiscovery {
    package_id: String,
    source: EditorPluginSource,
    loading_phase: EditorPluginLoadingPhase,
}

impl EditorPluginDiscovery {
    pub fn builtin(package_id: impl Into<String>) -> Self {
        Self::new(
            package_id,
            EditorPluginSource::Builtin,
            EditorPluginLoadingPhase::Default,
        )
    }

    pub fn project(package_id: impl Into<String>) -> Self {
        Self::new(
            package_id,
            EditorPluginSource::Project,
            EditorPluginLoadingPhase::PreWorkbench,
        )
    }

    pub fn package_manifest(package_id: impl Into<String>) -> Self {
        Self::new(
            package_id,
            EditorPluginSource::PackageManifest,
            EditorPluginLoadingPhase::Default,
        )
    }

    pub fn new(
        package_id: impl Into<String>,
        source: EditorPluginSource,
        loading_phase: EditorPluginLoadingPhase,
    ) -> Self {
        Self {
            package_id: package_id.into(),
            source,
            loading_phase,
        }
    }

    pub fn package_id(&self) -> &str {
        &self.package_id
    }

    pub fn source(&self) -> EditorPluginSource {
        self.source
    }

    pub fn loading_phase(&self) -> EditorPluginLoadingPhase {
        self.loading_phase
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EditorPluginDiscoveryError {
    BuiltinInitialization,
    MutationInProgress,
    PhaseRetractionRequiresDisable {
        package_id: String,
    },
    DisabledLifecycleRetryRequired {
        package_id: String,
    },
    LifecycleCleanupFailed {
        package_id: String,
        stage: EditorPluginLifecycleStage,
    },
    CatalogAdmission(EditorPluginCatalogAdmissionError),
    DuplicateDiscovery {
        package_id: String,
    },
    UnknownPackage {
        package_id: String,
    },
}

impl fmt::Display for EditorPluginDiscoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BuiltinInitialization => {
                formatter.write_str("builtin editor plugin manager could not be initialized")
            }
            Self::MutationInProgress => {
                formatter.write_str("editor plugin manager is dispatching a lifecycle mutation")
            }
            Self::PhaseRetractionRequiresDisable { package_id } => write!(
                formatter,
                "editor plugin `{package_id}` must be disabled before its active loading phase may be retracted"
            ),
            Self::DisabledLifecycleRetryRequired { package_id } => write!(
                formatter,
                "editor plugin `{package_id}` must retry its failed disabled lifecycle callback before replacement"
            ),
            Self::LifecycleCleanupFailed { package_id, stage } => write!(
                formatter,
                "editor plugin `{package_id}` failed lifecycle cleanup stage {stage:?} before replacement"
            ),
            Self::CatalogAdmission(error) => error.fmt(formatter),
            Self::DuplicateDiscovery { package_id } => {
                write!(
                    formatter,
                    "editor plugin `{package_id}` was discovered more than once"
                )
            }
            Self::UnknownPackage { package_id } => {
                write!(
                    formatter,
                    "editor plugin `{package_id}` is not in the catalog"
                )
            }
        }
    }
}

impl std::error::Error for EditorPluginDiscoveryError {}

impl From<EditorPluginCatalogAdmissionError> for EditorPluginDiscoveryError {
    fn from(value: EditorPluginCatalogAdmissionError) -> Self {
        Self::CatalogAdmission(value)
    }
}

/// Validates that one discovery row exists at most once for each catalog package.
pub(super) fn discovery_index(
    catalog: &EditorPluginCatalog,
    discoveries: impl IntoIterator<Item = EditorPluginDiscovery>,
) -> Result<BTreeMap<String, EditorPluginDiscovery>, EditorPluginDiscoveryError> {
    let package_ids = catalog
        .package_manifests()
        .iter()
        .map(|package| package.id.clone())
        .collect::<BTreeSet<_>>();
    let mut result = BTreeMap::new();
    for discovery in discoveries {
        if !package_ids.contains(discovery.package_id()) {
            return Err(EditorPluginDiscoveryError::UnknownPackage {
                package_id: discovery.package_id().to_string(),
            });
        }
        if result
            .insert(discovery.package_id().to_string(), discovery.clone())
            .is_some()
        {
            return Err(EditorPluginDiscoveryError::DuplicateDiscovery {
                package_id: discovery.package_id().to_string(),
            });
        }
    }
    Ok(result)
}
