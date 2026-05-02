use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::UiAssetFingerprint;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiAssetChange {
    Document,
    WidgetImport,
    StyleImport,
    DescriptorRegistry,
    ComponentContract,
    ResourceDependency,
}

// Snapshot fields mirror the compiler cache key, but remain owned by the
// invalidation module so graph classification is usable outside caching.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiInvalidationSnapshot {
    pub document: UiAssetFingerprint,
    pub widget_imports: BTreeMap<String, UiAssetFingerprint>,
    pub style_imports: BTreeMap<String, UiAssetFingerprint>,
    pub descriptor_registry_revision: u64,
    pub component_contract_revision: UiAssetFingerprint,
    pub resource_dependencies_revision: UiAssetFingerprint,
}
