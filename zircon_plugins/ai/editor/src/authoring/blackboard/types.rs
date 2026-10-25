use thiserror::Error;
use zircon_plugin_ai_runtime::behavior_tree::{BehaviorNodeSemantics, FrozenBehaviorNodeCatalog};
use zircon_runtime::asset::{AssetUri, AssetUuid};
use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeParameter, AiBlackboardSchemaDescriptor, AiManagerError,
};
use zircon_runtime::plugin::PluginModuleId;

use super::super::{AiBehaviorTreeAsset, AiBehaviorTreeEdit, AiBehaviorTreeEditError};

/// Host-supplied identity of the original bytes, not the candidate's canonical TOML.
/// Construction does not prove catalog registration or a successful source read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiBlackboardSourceVersion {
    pub asset_uuid: AssetUuid,
    pub uri: AssetUri,
    pub revision: u64,
    pub source_sha256: [u8; 32],
}

/// An explicit persisted association supplied by its owner. The editor does not infer
/// it from matching names, a live tick handle, or a node's Blackboard key text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiBlackboardTreeAssociation {
    pub tree: AiBlackboardSourceVersion,
    pub tree_id: String,
    pub schema: AiBlackboardSourceVersion,
    pub schema_id: String,
    pub owner: AiBlackboardSourceVersion,
    pub field_path: String,
}

#[derive(Clone, Copy, Debug)]
pub struct AiBlackboardSchemaInput<'a> {
    pub source: &'a AiBlackboardSourceVersion,
    pub descriptor: &'a AiBlackboardSchemaDescriptor,
    pub writable: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct AiBlackboardTreeInput<'a> {
    pub source: &'a AiBlackboardSourceVersion,
    pub asset: &'a AiBehaviorTreeAsset,
    pub association: Option<&'a AiBlackboardTreeAssociation>,
    pub writable: bool,
}

/// Registry identity includes its lifecycle: raw module IDs alone are not durable.
#[derive(Clone, Copy, Debug)]
pub struct AiBlackboardProviderSnapshot<'a> {
    pub catalog: &'a FrozenBehaviorNodeCatalog,
    pub registry_identity: &'a str,
    pub revision: u64,
    pub builtin_provider: PluginModuleId,
}

/// A dependency enumeration from the host/catalog, not a Boolean validity assertion.
/// Unknown enumeration is represented by `None` at scan time. This value cannot
/// establish global completeness, writer admission, or cross-document atomicity.
#[derive(Clone, Copy, Debug)]
pub struct AiBlackboardDependencies<'a> {
    pub catalog_identity: &'a str,
    pub revision: u64,
    pub tree_assets: &'a [AssetUuid],
    pub unresolved: &'a [AssetUri],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiBlackboardKeyUsage {
    ConditionAndObserverSlot,
    ObserverSlot,
    EffectWrite,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AiBlackboardKeyReference {
    pub tree: AiBlackboardSourceVersion,
    pub node_id: String,
    pub implementation: String,
    pub provider: PluginModuleId,
    pub original: AiBehaviorNodeParameter,
    pub usage: AiBlackboardKeyUsage,
    pub(super) tree_index: usize,
    pub(super) node_index: usize,
    pub(super) parameter_index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AiBlackboardReferenceBlocker {
    MissingDependencySet,
    MissingCatalogIdentity,
    UnresolvedDependency {
        uri: AssetUri,
    },
    MissingTree {
        tree: AssetUuid,
    },
    UnexpectedTree {
        tree: AssetUuid,
    },
    DuplicateTree {
        tree: AssetUuid,
    },
    MissingAssociation {
        tree: AssetUuid,
    },
    StaleAssociation {
        tree: AssetUuid,
    },
    UnknownProvider {
        tree: AssetUuid,
        node_id: String,
    },
    UnboundKey {
        tree: AssetUuid,
        node_id: String,
        key: String,
    },
    OpaqueNode {
        tree: AssetUuid,
        node_id: String,
        semantics: BehaviorNodeSemantics,
    },
}

#[derive(Debug)]
pub struct AiBlackboardReferencePlan<'a> {
    pub(super) schema: AiBlackboardSchemaInput<'a>,
    pub(super) trees: &'a [AiBlackboardTreeInput<'a>],
    pub(super) providers: AiBlackboardProviderSnapshot<'a>,
    pub(super) dependencies: Option<AiBlackboardDependencies<'a>>,
    pub(super) references: Vec<AiBlackboardKeyReference>,
    pub(super) blockers: Vec<AiBlackboardReferenceBlocker>,
}

#[derive(Debug, Error)]
pub enum AiBlackboardRenameError {
    #[error("Blackboard references are unresolved: {0:?}")]
    Blocked(Vec<AiBlackboardReferenceBlocker>),
    #[error("Blackboard source is read-only: {asset}")]
    ReadOnly { asset: AssetUuid },
    #[error("Blackboard key does not exist: {0}")]
    MissingKey(String),
    #[error("Blackboard key already exists: {0}")]
    DuplicateKey(String),
    #[error("Blackboard schema validation failed: {0:?}")]
    InvalidSchema(AiManagerError),
    #[error("behavior-tree candidate for {asset} is invalid: {error}")]
    InvalidTree {
        asset: AssetUuid,
        error: AiBehaviorTreeEditError,
    },
}

/// A business rewrite and inverse, not a document edit, undo entry or save receipt.
#[derive(Debug)]
pub struct AiBlackboardTreeRewrite {
    pub source: AiBlackboardSourceVersion,
    pub candidate: AiBehaviorTreeAsset,
    pub forward: Vec<AiBehaviorTreeEdit>,
    pub inverse: Vec<AiBehaviorTreeEdit>,
}

/// The host must admit allocations before building this value, then recheck the
/// entire closure and publish through its own cross-document transaction. No
/// individual tree may be saved early. The runtime schema remains untouched.
#[derive(Debug)]
pub struct AiBlackboardRenameCandidate {
    pub schema_source: AiBlackboardSourceVersion,
    pub schema_before: AiBlackboardSchemaDescriptor,
    pub schema_after: AiBlackboardSchemaDescriptor,
    pub trees: Vec<AiBlackboardTreeRewrite>,
    pub source_preconditions: Vec<AiBlackboardSourceVersion>,
    pub association_preconditions: Vec<AiBlackboardTreeAssociation>,
    pub provider_registry: String,
    pub provider_revision: u64,
    pub builtin_provider: PluginModuleId,
    pub dependency_catalog: String,
    pub dependency_revision: u64,
    pub dependency_tree_assets: Vec<AssetUuid>,
}
