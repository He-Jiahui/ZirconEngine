use std::sync::Arc;

use zircon_runtime::asset::project::{ProjectActivationIdentity, ProjectCatalogInputGeneration};
use zircon_runtime::asset::registry::AssetRegistryEntry;
use zircon_runtime::asset::AssetUuid;
use zircon_runtime_interface::project::AssetRef;

use crate::core::asset::AssetSourceAuthority;
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetCatalogRecord, EditorAssetDetailsGeneration,
};

/// One in-process publication, not an asset/document revision or a serializable token.
#[derive(Clone, Debug)]
pub struct EditorAssetReadScope {
    activation: ProjectActivationIdentity,
    input: Arc<ProjectCatalogInputGeneration>,
    catalog: Arc<EditorAssetCatalogGeneration>,
}

impl EditorAssetReadScope {
    pub(crate) fn new(
        activation: ProjectActivationIdentity,
        input: Arc<ProjectCatalogInputGeneration>,
        catalog: Arc<EditorAssetCatalogGeneration>,
    ) -> Self {
        Self {
            activation,
            input,
            catalog,
        }
    }

    pub fn activation(&self) -> &ProjectActivationIdentity {
        &self.activation
    }

    pub fn input(&self) -> &Arc<ProjectCatalogInputGeneration> {
        &self.input
    }

    pub fn catalog(&self) -> &Arc<EditorAssetCatalogGeneration> {
        &self.catalog
    }

    /// Preview-only publications also invalidate an expectation of this exact snapshot.
    pub fn same_publication(&self, other: &Self) -> bool {
        self.activation == other.activation
            && Arc::ptr_eq(&self.input, &other.input)
            && Arc::ptr_eq(&self.catalog, &other.catalog)
    }
}

/// Owned immutable observations. No manager lock or callback is retained by this value.
#[derive(Clone, Debug)]
pub struct EditorAssetReadSnapshot {
    pub requested: AssetRef,
    /// Present only after the project/index/input publication has matched.
    pub scope: Option<EditorAssetReadScope>,
    pub observation: EditorAssetReadObservation,
}

impl EditorAssetReadSnapshot {
    pub(crate) fn unknown(requested: &AssetRef, reason: EditorAssetReadUnknown) -> Self {
        Self {
            requested: requested.clone(),
            scope: None,
            observation: EditorAssetReadObservation::Unknown(reason),
        }
    }
}

#[derive(Clone, Debug)]
pub enum EditorAssetReadObservation {
    Known(Box<EditorAssetReadFacts>),
    Missing(EditorAssetReadMissing),
    Unknown(EditorAssetReadUnknown),
    Stale(EditorAssetReadStale),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorAssetReadUnknown {
    UnsupportedManager,
    NoActiveProject,
    IndexUnavailable,
    InputUnavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorAssetReadStale {
    ProjectRetired,
    ActivationMismatch,
    InputMismatch,
    RegistryMismatch,
    ExpectedPublicationMismatch,
    CatalogProjectionMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorAssetReadMissing {
    GuidNotRegistered,
    /// The GUID exists, but its registered label differs from the requested sub.
    GuidSubMismatch {
        registered_sub: Option<String>,
    },
}

#[derive(Clone, Debug)]
pub struct EditorAssetReadFacts {
    /// Runtime kind and opaque metadata digest; the digest is not a bytes SHA256.
    /// This entry does not establish an editor semantic type or a ready toolkit.
    pub registry_entry: AssetRegistryEntry,
    /// Both projections come from scope.catalog(); absent projection means unknown.
    pub asset: Option<Arc<EditorAssetCatalogRecord>>,
    pub details: Option<Arc<EditorAssetDetailsGeneration>>,
    pub dependencies: EditorAssetDependencyObservation,
    /// Derived only from the caller's actual type policy and the exact locator.
    /// None means no policy was supplied. This proves neither physical IO access,
    /// document locking, semantic validation nor save/transaction admission.
    pub source_policy_access: Option<AssetSourceAuthority>,
}

/// Registry edges, including referencers omitted by the UI catalog projection.
#[derive(Clone, Debug)]
pub struct EditorAssetDependencyObservation {
    pub outgoing: Vec<EditorAssetRegistryReference>,
    pub incoming: Vec<EditorAssetRegistryReference>,
    pub asset_edge_coverage: EditorAssetDependencyCoverage,
    pub semantic_coverage: EditorAssetDependencyCoverage,
}

/// Neither current extractors nor the registry declare complete semantic coverage.
/// An empty observed edge list therefore never establishes zero references.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorAssetDependencyCoverage {
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorAssetRegistryReference {
    Known(AssetRegistryEntry),
    /// An observed edge points at a UUID absent from this captured registry.
    Missing(AssetUuid),
}
