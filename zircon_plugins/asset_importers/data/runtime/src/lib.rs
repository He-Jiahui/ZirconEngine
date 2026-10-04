use serde_json::{Map, Value};
use std::collections::HashSet;
use std::ops::Range;
use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetUri, DataAsset,
    DataAssetFormat, ImportedAsset,
};
use zircon_runtime::core::resource::ResourceScheme;

const PERSISTED_ASSET_URI_PREFIXES: [&str; 4] =
    ["res://", "lib://", "package://", "builtin://"];

mod capability;
mod plugin;

pub use capability::{
    DATA_ASSET_IMPORTER_DECLARATION, IMPORTER_FAMILY, JSON_IMPORTER_CAPABILITY, MODULE_NAME,
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST, PLUGIN_ID, RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME,
    TOML_IMPORTER_CAPABILITY, XML_IMPORTER_CAPABILITY, YAML_IMPORTER_CAPABILITY,
};
pub use plugin::{
    asset_importer_descriptors, dist_module_manifest, module_descriptor, package_manifest,
    plugin_registration, runtime_capabilities, runtime_module_manifest, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, supported_platforms, supported_targets,
    DataAssetImporterRuntimePlugin, ASSET_IMPORTER_DATA_DIST_CRATE_NAME,
    ASSET_IMPORTER_DATA_DIST_RUNTIME_ENTRY,
};

pub fn import_toml_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let text = context.source_text()?;
    let value: toml::Value = toml::from_str(&text)
        .map_err(|error| AssetImportError::Parse(format!("parse toml data: {error}")))?;
    data_outcome(
        context,
        DataAssetFormat::Toml,
        text,
        serde_json::to_value(value)?,
    )
}

pub fn import_json_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let text = context.source_text()?;
    let canonical_json: Value = serde_json::from_str(&text)
        .map_err(|error| AssetImportError::Parse(format!("parse json data: {error}")))?;
    data_outcome(context, DataAssetFormat::Json, text, canonical_json)
}

pub fn import_yaml_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let text = context.source_text()?;
    let canonical_json: Value = serde_yaml::from_str(&text)
        .map_err(|error| AssetImportError::Parse(format!("parse yaml data: {error}")))?;
    data_outcome(context, DataAssetFormat::Yaml, text, canonical_json)
}

pub fn import_xml_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let text = context.source_text()?;
    let document = roxmltree::Document::parse(&text)
        .map_err(|error| AssetImportError::Parse(format!("parse xml data: {error}")))?;
    let canonical_json = xml_element_to_json(document.root_element(), &text);
    data_outcome(context, DataAssetFormat::Xml, text, canonical_json)
}

fn data_outcome(
    context: &AssetImportContext,
    format: DataAssetFormat,
    text: String,
    canonical_json: Value,
) -> Result<AssetImportOutcome, AssetImportError> {
    let dependencies = collect_data_asset_dependencies(&canonical_json, &context.uri);
    let mut outcome = AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(DataAsset {
            uri: context.uri.clone(),
            format,
            text,
            canonical_json,
        }),
    );

    for dependency in dependencies {
        outcome = outcome.with_dependency(dependency);
    }

    Ok(outcome)
}

fn collect_data_asset_dependencies(value: &Value, owner: &AssetUri) -> Vec<AssetUri> {
    let mut dependencies = Vec::new();
    let mut seen = HashSet::new();
    collect_data_asset_dependencies_into(value, owner, &mut seen, &mut dependencies);
    dependencies
}

fn collect_data_asset_dependencies_into(
    value: &Value,
    owner: &AssetUri,
    seen: &mut HashSet<AssetUri>,
    dependencies: &mut Vec<AssetUri>,
) {
    match value {
        Value::String(value) => {
            if !PERSISTED_ASSET_URI_PREFIXES
                .iter()
                .any(|prefix| value.starts_with(prefix))
            {
                return;
            }

            let Ok(uri) = AssetUri::parse(value) else {
                return;
            };

            // Only exact canonical, persisted resource locators are dependencies. This is a
            // conservative URI projection, not schema validation: ordinary paths/IDs, memory
            // previews, and normalizable-but-noncanonical text stay data while malformed or
            // future schemes fail closed.
            if !uri.matches_display(value)
                || &uri == owner
                || !matches!(
                    uri.scheme(),
                    ResourceScheme::Res
                        | ResourceScheme::Library
                        | ResourceScheme::Package
                        | ResourceScheme::Builtin
                )
            {
                return;
            }

            if seen.insert(uri.clone()) {
                dependencies.push(uri);
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_data_asset_dependencies_into(value, owner, seen, dependencies);
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                collect_data_asset_dependencies_into(value, owner, seen, dependencies);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

// XML is not intrinsically JSON-shaped, so the importer emits a stable neutral tree DTO.
// The existing fields remain for consumers already reading the neutral tree; ordered metadata
// keeps namespace, attribute-order, mixed-content, and source-span semantics available without
// changing the public DataAsset shape.
fn xml_element_to_json(node: roxmltree::Node<'_, '_>, source: &str) -> Value {
    let preserve_metadata = xml_node_needs_lossless_metadata(node);
    let mut object = Map::new();
    object.insert(
        "name".to_string(),
        Value::String(node.tag_name().name().to_string()),
    );
    if let Some(namespace) = node.tag_name().namespace() {
        object.insert(
            "namespace".to_string(),
            Value::String(namespace.to_string()),
        );
    }

    if preserve_metadata {
        object.insert("source_span".to_string(), source_span(node.range()));

        let namespaces = node
            .namespaces()
            .map(|namespace| {
                let mut entry = Map::new();
                entry.insert(
                    "prefix".to_string(),
                    namespace
                        .name()
                        .map(str::to_owned)
                        .map(Value::String)
                        .unwrap_or(Value::Null),
                );
                entry.insert(
                    "uri".to_string(),
                    Value::String(namespace.uri().to_string()),
                );
                Value::Object(entry)
            })
            .collect::<Vec<_>>();
        if !namespaces.is_empty() {
            object.insert("namespaces".to_string(), Value::Array(namespaces));
        }
    }

    let attributes = node
        .attributes()
        .map(|attribute| {
            (
                attribute.name().to_string(),
                Value::String(attribute.value().to_string()),
            )
        })
        .collect::<Map<_, _>>();
    if !attributes.is_empty() {
        object.insert("attributes".to_string(), Value::Object(attributes));
    }

    if preserve_metadata {
        let attributes_ordered = node
            .attributes()
            .map(|attribute| {
                let mut entry = Map::new();
                let qname = source
                    .get(attribute.range_qname())
                    .unwrap_or(attribute.name())
                    .to_string();
                entry.insert("qname".to_string(), Value::String(qname));
                entry.insert(
                    "name".to_string(),
                    Value::String(attribute.name().to_string()),
                );
                entry.insert(
                    "namespace".to_string(),
                    attribute
                        .namespace()
                        .map(str::to_owned)
                        .map(Value::String)
                        .unwrap_or(Value::Null),
                );
                entry.insert(
                    "value".to_string(),
                    Value::String(attribute.value().to_string()),
                );
                entry.insert("source_span".to_string(), source_span(attribute.range()));
                Value::Object(entry)
            })
            .collect::<Vec<_>>();
        if !attributes_ordered.is_empty() {
            object.insert(
                "attributes_ordered".to_string(),
                Value::Array(attributes_ordered),
            );
        }
    }

    let mut text = None;
    let mut children = Vec::new();
    let mut content = Vec::new();
    for child in node.children() {
        if let Some(child_text) = child.text().map(str::trim).filter(|text| !text.is_empty()) {
            append_xml_text(&mut text, child_text);
        }
        if child.is_element() {
            let index = children.len();
            children.push(xml_element_to_json(child, source));
            if preserve_metadata {
                let mut entry = Map::new();
                entry.insert("kind".to_string(), Value::String("element".to_string()));
                entry.insert(
                    "index".to_string(),
                    Value::Number(serde_json::Number::from(index)),
                );
                entry.insert("source_span".to_string(), source_span(child.range()));
                content.push(Value::Object(entry));
            }
        } else if preserve_metadata && child.is_text() {
            let mut entry = Map::new();
            entry.insert("kind".to_string(), Value::String("text".to_string()));
            entry.insert(
                "value".to_string(),
                Value::String(child.text().unwrap_or_default().to_string()),
            );
            entry.insert("source_span".to_string(), source_span(child.range()));
            content.push(Value::Object(entry));
        } else if preserve_metadata && child.is_comment() {
            let mut entry = Map::new();
            entry.insert("kind".to_string(), Value::String("comment".to_string()));
            entry.insert(
                "value".to_string(),
                Value::String(child.text().unwrap_or_default().to_string()),
            );
            entry.insert("source_span".to_string(), source_span(child.range()));
            content.push(Value::Object(entry));
        } else if preserve_metadata {
            if let Some(pi) = child.pi() {
                let mut entry = Map::new();
                entry.insert(
                    "kind".to_string(),
                    Value::String("processing_instruction".to_string()),
                );
                entry.insert("target".to_string(), Value::String(pi.target.to_string()));
                entry.insert(
                    "value".to_string(),
                    pi.value
                        .map(str::to_owned)
                        .map(Value::String)
                        .unwrap_or(Value::Null),
                );
                entry.insert("source_span".to_string(), source_span(child.range()));
                content.push(Value::Object(entry));
            }
        }
    }
    if let Some(text) = text {
        object.insert("text".to_string(), text);
    }
    if !children.is_empty() {
        object.insert("children".to_string(), Value::Array(children));
    }
    if !content.is_empty() {
        object.insert("content".to_string(), Value::Array(content));
    }

    Value::Object(object)
}

fn xml_node_needs_lossless_metadata(node: roxmltree::Node<'_, '_>) -> bool {
    if node.namespaces().len() > 0 {
        return true;
    }

    let attributes = node.attributes();
    if attributes.len() > 1
        || attributes
            .clone()
            .any(|attribute| attribute.namespace().is_some())
    {
        return true;
    }

    let mut has_text = false;
    let mut has_element = false;
    for child in node.children() {
        if child.is_text() {
            has_text = true;
            if child.text().is_some_and(|text| text.trim() != text) {
                return true;
            }
        } else if child.is_element() {
            has_element = true;
        } else {
            return true;
        }
    }

    has_text && has_element
}

fn source_span(range: Range<usize>) -> Value {
    // roxmltree ranges are UTF-8 byte offsets; DataAsset::text retains the exact source.
    Value::Array(vec![
        Value::Number(serde_json::Number::from(range.start)),
        Value::Number(serde_json::Number::from(range.end)),
    ])
}

fn append_xml_text(slot: &mut Option<Value>, text: &str) {
    let next = Value::String(text.to_string());
    let current = slot.take();
    *slot = Some(match current {
        None => next,
        Some(Value::Array(mut values)) => {
            values.push(next);
            Value::Array(values)
        }
        Some(first) => Value::Array(vec![first, next]),
    });
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
