use crate::ui::template_runtime::builtin::{
    builtin_component_descriptors, builtin_template_bindings, builtin_template_documents,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use zircon_runtime::ui::template::{
    UiAssetDocument, UiAssetError, UiAssetLoader, UiComponentDefinition, UiDocumentCompiler,
};

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
    let documents = load_builtin_template_documents()?;

    for (document_id, document) in &documents {
        let mut compiler = UiDocumentCompiler::default();
        let mut seen_imports = BTreeSet::new();
        register_document_imports(&mut compiler, document, &documents, &mut seen_imports)?;
        let compiled = compiler.compile(document)?;
        runtime
            .template_registry
            .register_compiled_document(*document_id, compiled)?;
    }

    Ok(())
}

fn load_builtin_template_documents(
) -> Result<BTreeMap<&'static str, UiAssetDocument>, EditorUiHostRuntimeError> {
    let mut documents = BTreeMap::new();
    for (document_id, path) in builtin_template_documents() {
        let source =
            std::fs::read_to_string(path).map_err(|error| UiAssetError::Io(error.to_string()))?;
        let document = UiAssetLoader::load_toml_str(&source)?;
        documents.insert(document_id, document);
    }
    Ok(documents)
}

fn register_document_imports(
    compiler: &mut UiDocumentCompiler,
    document: &UiAssetDocument,
    documents: &BTreeMap<&'static str, UiAssetDocument>,
    seen_imports: &mut BTreeSet<String>,
) -> Result<(), UiAssetError> {
    for reference in &document.imports.widgets {
        if !seen_imports.insert(reference.clone()) {
            continue;
        }
        let Some(imported) = resolve_builtin_import(reference, documents)? else {
            continue;
        };
        compiler.register_widget_import(reference, imported.clone())?;
        if !reference.contains('#') {
            for component_name in imported.components.keys() {
                compiler.register_widget_import(
                    format!("{reference}#{component_name}"),
                    imported.clone(),
                )?;
            }
            for alias in root_component_aliases(&imported) {
                compiler.register_widget_import(
                    format!("{reference}#{alias}"),
                    document_with_root_component_alias(imported.clone(), alias),
                )?;
            }
        }
        register_document_imports(compiler, &imported, documents, seen_imports)?;
    }

    for reference in &document.imports.styles {
        if !seen_imports.insert(reference.clone()) {
            continue;
        }
        let Some(imported) = resolve_builtin_import(reference, documents)? else {
            continue;
        };
        compiler.register_style_import(reference, imported.clone())?;
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
    reference: &str,
    documents: &BTreeMap<&'static str, UiAssetDocument>,
) -> Result<Option<UiAssetDocument>, UiAssetError> {
    let Some(path) = reference
        .strip_prefix("res://")
        .and_then(|value| value.split('#').next())
    else {
        return Ok(None);
    };
    let normalized = Path::new("assets").join(PathBuf::from(path));
    if let Some((document_id, _)) = builtin_template_documents()
        .into_iter()
        .find(|(_, candidate_path)| candidate_path.ends_with(&normalized))
    {
        return Ok(documents.get(document_id).cloned());
    }

    let source_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(normalized);
    let source = std::fs::read_to_string(source_path)
        .map_err(|error| UiAssetError::Io(error.to_string()))?;
    Ok(Some(UiAssetLoader::load_toml_str(&source)?))
}
