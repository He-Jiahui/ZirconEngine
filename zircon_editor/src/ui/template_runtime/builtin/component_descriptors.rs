use crate::ui::template::{
    parse_editor_component_catalog_manifest, EditorComponentCatalogManifestError,
    EditorComponentDescriptor,
};

pub(crate) const BUILTIN_COMPONENT_CATALOG_MANIFEST_ID: &str =
    "res://ui/editor/components/catalog.toml";
const BUILTIN_COMPONENT_CATALOG_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/components/catalog.toml"
));

pub(crate) fn builtin_component_descriptors(
) -> Result<Vec<EditorComponentDescriptor>, EditorComponentCatalogManifestError> {
    parse_editor_component_catalog_manifest(BUILTIN_COMPONENT_CATALOG_MANIFEST)
}

#[cfg(test)]
fn builtin_component_descriptors_for_tests() -> Vec<EditorComponentDescriptor> {
    builtin_component_descriptors().expect("builtin component catalog asset should remain valid")
}

#[cfg(test)]
fn primitive_root_prop_default(
    document_id: &str,
    property_name: &str,
) -> crate::ui::template::EditorPropLiteral {
    let relative_path = document_id
        .strip_prefix("res://")
        .expect("builtin primitive documents should use res:// identifiers");
    let source_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(relative_path);
    let source = std::fs::read_to_string(&source_path)
        .unwrap_or_else(|error| panic!("could not read {}: {error}", source_path.display()));
    let document: toml::Value = toml::from_str(&source)
        .unwrap_or_else(|error| panic!("could not parse {}: {error}", source_path.display()));
    let property = document
        .get("nodes")
        .and_then(toml::Value::as_table)
        .and_then(|nodes| nodes.get("root"))
        .and_then(toml::Value::as_table)
        .and_then(|root| root.get("props"))
        .and_then(toml::Value::as_table)
        .and_then(|props| props.get(property_name))
        .unwrap_or_else(|| {
            panic!(
                "{} should expose root prop `{property_name}`",
                source_path.display()
            )
        });
    match property {
        toml::Value::String(value) => crate::ui::template::EditorPropLiteral::Text(value.clone()),
        toml::Value::Boolean(value) => crate::ui::template::EditorPropLiteral::Boolean(*value),
        toml::Value::Integer(value) => crate::ui::template::EditorPropLiteral::Integer(*value),
        toml::Value::Float(value) => crate::ui::template::EditorPropLiteral::Float(*value),
        toml::Value::Array(values) => crate::ui::template::EditorPropLiteral::TextList(
            values
                .iter()
                .map(|value| {
                    value.as_str().unwrap_or_else(|| {
                        panic!(
                            "{} root prop `{property_name}` must contain only text values",
                            source_path.display()
                        )
                    })
                })
                .map(str::to_string)
                .collect(),
        ),
        _ => panic!(
            "{} root prop `{property_name}` must be a supported literal default",
            source_path.display()
        ),
    }
}

#[cfg(test)]
#[path = "component_descriptors/tests/dialog_contract_tests.rs"]
mod dialog_contract_tests;

#[cfg(test)]
#[path = "component_descriptors/tests/feedback_container_contract_tests.rs"]
mod feedback_container_contract_tests;

#[cfg(test)]
#[path = "component_descriptors/tests/feedback_state_contract_tests.rs"]
mod feedback_state_contract_tests;

#[cfg(test)]
#[path = "component_descriptors/tests/tooltip_contract_tests.rs"]
mod tooltip_contract_tests;

#[cfg(test)]
#[path = "tests/component_descriptors.rs"]
mod tests;
