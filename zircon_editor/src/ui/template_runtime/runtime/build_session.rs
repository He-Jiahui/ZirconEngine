use crate::ui::template_runtime::builtin::{
    builtin_component_descriptors, builtin_template_bindings, builtin_template_documents,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
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
    write_diagnostic_log(
        "editor_template_compile",
        format!(
            "load_document path={} exists={}",
            path.display(),
            path.exists()
        ),
    );
    let document = template_service.load_document_file(path)?;
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
    Ok(template_service.compile_document_with_import_maps(
        &document,
        &widget_imports,
        &style_imports,
    )?)
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
    Ok(Some(template_service.load_document_file(source_path)?))
}

fn editor_runtime_asset_path(relative: &str) -> std::path::PathBuf {
    runtime_asset_path_with_dev_asset_root(relative, editor_dev_asset_root())
}

fn editor_dev_asset_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}
