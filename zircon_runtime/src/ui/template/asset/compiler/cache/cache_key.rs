use crate::ui::template::{
    component_contract_fingerprint, declared_imports_fingerprint, document_import_fingerprints,
    fingerprint_document, resource_dependencies_fingerprint,
};
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiAssetError, UiCompileCacheKey};

use super::super::UiDocumentCompiler;

/// 同时刻画源文档、已注册导入、描述符和跨资产契约，用于内存复用及包产物失效判断。
/// 注册集合的变化也是失效输入；即使未被当前布局使用的注册项变化，也会造成保守失效。
pub fn compile_cache_key_from_compiler(
    compiler: &UiDocumentCompiler,
    document: &UiAssetDocument,
) -> Result<UiCompileCacheKey, UiAssetError> {
    Ok(UiCompileCacheKey {
        root_document: fingerprint_document(document)?,
        widget_imports: document_import_fingerprints(&compiler.widget_imports)?,
        style_imports: document_import_fingerprints(&compiler.style_imports)?,
        declared_widget_imports_revision: declared_imports_fingerprint(&document.imports.widgets)?,
        declared_style_imports_revision: declared_imports_fingerprint(&document.imports.styles)?,
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
