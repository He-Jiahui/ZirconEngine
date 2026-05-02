use std::collections::BTreeMap;

use serde::Serialize;

use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiAssetFingerprint, UiResourceRef,
};

use super::super::resource_ref::{
    collect_document_resource_dependencies, unique_resource_references,
};

pub fn document_import_fingerprints(
    imports: &BTreeMap<String, UiAssetDocument>,
) -> Result<BTreeMap<String, UiAssetFingerprint>, UiAssetError> {
    imports
        .iter()
        .map(|(reference, document)| Ok((reference.clone(), fingerprint_document(document)?)))
        .collect()
}

pub fn fingerprint_document(
    document: &UiAssetDocument,
) -> Result<UiAssetFingerprint, UiAssetError> {
    fingerprint_serializable(document)
}

pub fn component_contract_fingerprint(
    document: &UiAssetDocument,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
) -> Result<UiAssetFingerprint, UiAssetError> {
    let mut source = String::new();
    append_contracts(&mut source, "root", document)?;
    for (reference, import) in widget_imports {
        append_contracts(&mut source, reference, import)?;
    }
    Ok(UiAssetFingerprint::from_bytes(source.as_bytes()))
}

pub fn resource_dependencies_fingerprint(
    document: &UiAssetDocument,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
    style_imports: &BTreeMap<String, UiAssetDocument>,
) -> Result<UiAssetFingerprint, UiAssetError> {
    let report = collect_document_resource_dependencies(document, widget_imports, style_imports)?;
    let input = UiResourceDependencyFingerprintInput {
        references: unique_resource_references(&report.dependencies)
            .into_iter()
            .collect(),
    };
    fingerprint_serializable(&input)
}

#[derive(Serialize)]
struct UiResourceDependencyFingerprintInput {
    references: Vec<UiResourceRef>,
}

fn append_contracts(
    source: &mut String,
    owner: &str,
    document: &UiAssetDocument,
) -> Result<(), UiAssetError> {
    source.push_str(owner);
    source.push('\n');
    for (component_name, component) in &document.components {
        source.push_str(component_name);
        source.push('\n');
        source.push_str(&serialize_for_fingerprint(&component.contract)?);
        source.push('\n');
    }
    Ok(())
}

fn fingerprint_serializable<T>(value: &T) -> Result<UiAssetFingerprint, UiAssetError>
where
    T: Serialize,
{
    serialize_for_fingerprint(value)
        .map(|serialized| UiAssetFingerprint::from_bytes(serialized.as_bytes()))
}

fn serialize_for_fingerprint<T>(value: &T) -> Result<String, UiAssetError>
where
    T: Serialize,
{
    toml::to_string(value).map_err(|error| UiAssetError::InvalidDocument {
        asset_id: "ui-asset-fingerprint".to_string(),
        detail: format!("failed to serialize deterministic fingerprint input: {error}"),
    })
}
