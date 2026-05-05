use crate::ui::template_runtime::builtin::{
    builtin_component_descriptors, builtin_template_bindings, builtin_template_documents,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::UNIX_EPOCH;
use zircon_runtime::asset::runtime_asset_path_with_dev_asset_root;
use zircon_runtime::diagnostic_log::write_diagnostic_log;
use zircon_runtime::ui::template::UiCompiledDocument;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiComponentDefinition,
};

use crate::ui::template::EditorTemplateRuntimeService;

use super::runtime_host::{EditorUiHostRuntime, EditorUiHostRuntimeError};

pub(super) fn load_builtin_host_templates(
    runtime: &mut EditorUiHostRuntime,
) -> Result<(), EditorUiHostRuntimeError> {
    if runtime.builtin_host_templates_loaded {
        return Ok(());
    }

    for descriptor in builtin_component_descriptors() {
        runtime.register_component(descriptor)?;
    }

    register_builtin_template_documents(runtime)?;

    for (binding_id, binding) in builtin_template_bindings() {
        runtime.register_binding(binding_id, binding)?;
    }

    runtime.builtin_host_templates_loaded = true;
    Ok(())
}

fn register_builtin_template_documents(
    runtime: &mut EditorUiHostRuntime,
) -> Result<(), EditorUiHostRuntimeError> {
    for (document_id, path) in builtin_template_documents() {
        write_diagnostic_log(
            "editor_builtin_templates",
            format!(
                "register_document id={} path={} exists={}",
                document_id,
                path.display(),
                path.exists()
            ),
        );
        runtime.register_document_file(document_id, path)?;
    }

    Ok(())
}

pub(super) fn compile_template_document_file(
    template_service: &EditorTemplateRuntimeService,
    path: &Path,
) -> Result<UiCompiledDocument, EditorUiHostRuntimeError> {
    let cache_key = BuiltinTemplateCompileCacheKey::from_path(path);
    if let Some(compiled) = builtin_template_compile_cache()
        .lock()
        .expect("builtin template compile cache mutex should not be poisoned")
        .get(&cache_key)
        .cloned()
    {
        write_diagnostic_log(
            "editor_template_compile_cache",
            format!(
                "hit path={} modified_unix_ns={}",
                cache_key.path.display(),
                cache_key.modified_unix_ns.unwrap_or(0)
            ),
        );
        return Ok(compiled);
    }

    write_diagnostic_log(
        "editor_template_compile",
        format!(
            "load_document path={} exists={}",
            path.display(),
            path.exists()
        ),
    );
    let document = load_builtin_template_document_file(template_service, path)
        .map_err(EditorUiHostRuntimeError::from)?;
    let mut widget_imports = BTreeMap::new();
    let mut style_imports = BTreeMap::new();
    let mut seen_imports = BTreeSet::new();
    register_document_imports(
        template_service,
        &mut widget_imports,
        &mut style_imports,
        &document,
        &mut seen_imports,
    )?;
    let compiled = template_service.compile_document_with_import_maps(
        &document,
        &widget_imports,
        &style_imports,
    )?;
    builtin_template_compile_cache()
        .lock()
        .expect("builtin template compile cache mutex should not be poisoned")
        .insert(cache_key, compiled.clone());
    Ok(compiled)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct BuiltinTemplateCompileCacheKey {
    path: PathBuf,
    modified_unix_ns: Option<u128>,
    len: Option<u64>,
}

impl BuiltinTemplateCompileCacheKey {
    fn from_path(path: &Path) -> Self {
        let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let metadata = std::fs::metadata(&path).ok();
        let modified_unix_ns = metadata
            .as_ref()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_nanos());
        let len = metadata.as_ref().map(std::fs::Metadata::len);
        Self {
            path,
            modified_unix_ns,
            len,
        }
    }
}

fn builtin_template_compile_cache(
) -> &'static Mutex<BTreeMap<BuiltinTemplateCompileCacheKey, UiCompiledDocument>> {
    static CACHE: OnceLock<Mutex<BTreeMap<BuiltinTemplateCompileCacheKey, UiCompiledDocument>>> =
        OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn load_builtin_template_document_file(
    template_service: &EditorTemplateRuntimeService,
    path: &Path,
) -> Result<UiAssetDocument, UiAssetError> {
    let cache_key = BuiltinTemplateCompileCacheKey::from_path(path);
    if let Some(document) = builtin_template_document_cache()
        .lock()
        .expect("builtin template document cache mutex should not be poisoned")
        .get(&cache_key)
        .cloned()
    {
        write_diagnostic_log(
            "editor_template_document_cache",
            format!(
                "hit path={} modified_unix_ns={}",
                cache_key.path.display(),
                cache_key.modified_unix_ns.unwrap_or(0)
            ),
        );
        return Ok(document);
    }

    let document = template_service.load_document_file(path)?;
    builtin_template_document_cache()
        .lock()
        .expect("builtin template document cache mutex should not be poisoned")
        .insert(cache_key, document.clone());
    Ok(document)
}

fn builtin_template_document_cache(
) -> &'static Mutex<BTreeMap<BuiltinTemplateCompileCacheKey, UiAssetDocument>> {
    static CACHE: OnceLock<Mutex<BTreeMap<BuiltinTemplateCompileCacheKey, UiAssetDocument>>> =
        OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn register_document_imports(
    template_service: &EditorTemplateRuntimeService,
    widget_imports: &mut BTreeMap<String, UiAssetDocument>,
    style_imports: &mut BTreeMap<String, UiAssetDocument>,
    document: &UiAssetDocument,
    seen_imports: &mut BTreeSet<String>,
) -> Result<(), UiAssetError> {
    for reference in &document.imports.widgets {
        if !seen_imports.insert(reference.clone()) {
            continue;
        }
        let Some(imported) = resolve_builtin_import(template_service, reference)? else {
            continue;
        };
        widget_imports.insert(reference.clone(), imported.clone());
        if !reference.contains('#') {
            for component_name in imported.components.keys() {
                widget_imports.insert(format!("{reference}#{component_name}"), imported.clone());
            }
            for alias in root_component_aliases(&imported) {
                widget_imports.insert(
                    format!("{reference}#{alias}"),
                    document_with_root_component_alias(imported.clone(), alias),
                );
            }
        }
        register_document_imports(
            template_service,
            widget_imports,
            style_imports,
            &imported,
            seen_imports,
        )?;
    }

    for reference in &document.imports.styles {
        if !seen_imports.insert(reference.clone()) {
            continue;
        }
        let Some(imported) = resolve_builtin_import(template_service, reference)? else {
            continue;
        };
        style_imports.insert(reference.clone(), imported.clone());
    }

    Ok(())
}

fn root_component_aliases(document: &UiAssetDocument) -> Vec<String> {
    let Some(root) = &document.root else {
        return Vec::new();
    };
    [root.control_id.as_ref(), Some(&root.node_id)]
        .into_iter()
        .flatten()
        .filter(|alias| !alias.is_empty() && !document.components.contains_key(alias.as_str()))
        .cloned()
        .collect()
}

fn document_with_root_component_alias(
    mut document: UiAssetDocument,
    alias: String,
) -> UiAssetDocument {
    let Some(root) = document.root.clone() else {
        return document;
    };
    document.components.insert(
        alias,
        UiComponentDefinition {
            root,
            ..Default::default()
        },
    );
    document
}

fn resolve_builtin_import(
    template_service: &EditorTemplateRuntimeService,
    reference: &str,
) -> Result<Option<UiAssetDocument>, UiAssetError> {
    let Some(path) = reference
        .strip_prefix("res://")
        .and_then(|value| value.split('#').next())
    else {
        return Ok(None);
    };
    let source_path = editor_runtime_asset_path(path);
    write_diagnostic_log(
        "editor_template_import",
        format!(
            "reference={} resolved_path={} exists={}",
            reference,
            source_path.display(),
            source_path.exists()
        ),
    );
    Ok(Some(load_builtin_template_document_file(
        template_service,
        &source_path,
    )?))
}

fn editor_runtime_asset_path(relative: &str) -> std::path::PathBuf {
    runtime_asset_path_with_dev_asset_root(relative, editor_dev_asset_root())
}

fn editor_dev_asset_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_template_compile_cache_is_reused_across_runtime_instances() {
        let mut first = EditorUiHostRuntime::default();
        first
            .load_builtin_host_templates()
            .expect("first runtime should load builtin templates");
        let compiled_after_first = builtin_template_compile_cache()
            .lock()
            .expect("cache mutex should not be poisoned")
            .len();
        let documents_after_first = builtin_template_document_cache()
            .lock()
            .expect("document cache mutex should not be poisoned")
            .len();

        let mut second = EditorUiHostRuntime::default();
        second
            .load_builtin_host_templates()
            .expect("second runtime should reuse builtin template cache");

        assert!(compiled_after_first > 0);
        assert!(documents_after_first > 0);
        assert_eq!(
            builtin_template_compile_cache()
                .lock()
                .expect("cache mutex should not be poisoned")
                .len(),
            compiled_after_first,
            "second runtime should not compile additional builtin documents"
        );
        assert_eq!(
            builtin_template_document_cache()
                .lock()
                .expect("document cache mutex should not be poisoned")
                .len(),
            documents_after_first,
            "second runtime should not reload additional builtin documents"
        );
    }
}
