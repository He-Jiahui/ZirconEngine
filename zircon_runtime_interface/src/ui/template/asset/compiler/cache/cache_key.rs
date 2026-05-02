use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ui::template::{UiAssetFingerprint, UiInvalidationSnapshot};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiCompileCacheKey {
    pub root_document: UiAssetFingerprint,
    pub widget_imports: BTreeMap<String, UiAssetFingerprint>,
    pub style_imports: BTreeMap<String, UiAssetFingerprint>,
    pub descriptor_registry_revision: u64,
    pub component_contract_revision: UiAssetFingerprint,
    pub resource_dependencies_revision: UiAssetFingerprint,
}

impl UiCompileCacheKey {
    pub fn invalidation_snapshot(&self) -> UiInvalidationSnapshot {
        UiInvalidationSnapshot {
            document: self.root_document,
            widget_imports: self.widget_imports.clone(),
            style_imports: self.style_imports.clone(),
            descriptor_registry_revision: self.descriptor_registry_revision,
            component_contract_revision: self.component_contract_revision,
            resource_dependencies_revision: self.resource_dependencies_revision,
        }
    }
}
