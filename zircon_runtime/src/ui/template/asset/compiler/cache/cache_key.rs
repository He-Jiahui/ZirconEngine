use crate::ui::template::{
    component_contract_fingerprint, document_import_fingerprints, fingerprint_document,
    resource_dependencies_fingerprint,
};
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiAssetError, UiCompileCacheKey};

use super::super::UiDocumentCompiler;

pub fn compile_cache_key_from_compiler(
    compiler: &UiDocumentCompiler,
    document: &UiAssetDocument,
) -> Result<UiCompileCacheKey, UiAssetError> {
    Ok(UiCompileCacheKey {
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
