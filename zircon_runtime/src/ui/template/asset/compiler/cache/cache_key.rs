use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ui::template::{
    component_contract_fingerprint, document_import_fingerprints, fingerprint_document,
    resource_dependencies_fingerprint,
};
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiAssetFingerprint, UiInvalidationSnapshot,
};

use super::super::UiDocumentCompiler;

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
    pub fn from_compiler(
        compiler: &UiDocumentCompiler,
        document: &UiAssetDocument,
    ) -> Result<Self, UiAssetError> {
        Ok(Self {
            root_document: fingerprint_document(document)?,
            widget_imports: document_import_fingerprints(&compiler.widget_imports)?,
            style_imports: document_import_fingerprints(&compiler.style_imports)?,
            descriptor_registry_revision: compiler.component_registry_revision(),
            component_contract_revision: component_contract_fingerprint(
                document,
                &compiler.widget_imports,
            )?,
            resource_dependencies_revision: resource_dependencies_fingerprint(
                document,
                &compiler.widget_imports,
                &compiler.style_imports,
            )?,
        })
    }

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
