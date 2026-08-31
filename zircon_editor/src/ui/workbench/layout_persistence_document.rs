use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use zircon_runtime_interface::serialization::{
    load_versioned, write_versioned_text, Format, LoadError, MigrateError, MigrationChain,
    MigrationStep, SchemaId, VersionedSchema, WriteError,
};

use super::layout::WorkbenchLayout;
use super::LayoutPresetPersistenceStore;

#[derive(Debug, Error)]
pub(crate) enum LayoutPersistenceDocumentError {
    #[error("workbench layout document encode failed: {0}")]
    Encode(#[from] WriteError),
    #[error("workbench layout document decode failed: {0}")]
    Decode(#[from] LoadError),
    #[error("workbench layout config value conversion failed: {0}")]
    ConfigValue(#[from] serde_json::Error),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DefaultLayoutDocument {
    workbench: WorkbenchLayout,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NamedLayoutPresetsDocument {
    presets: BTreeMap<String, WorkbenchLayout>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PageLayoutPresetsDocument {
    store: LayoutPresetPersistenceStore,
}

pub(crate) fn encode_default_layout_value(
    workbench: WorkbenchLayout,
) -> Result<Value, LayoutPersistenceDocumentError> {
    encode_config_value(&DefaultLayoutDocument { workbench })
}

pub(crate) fn decode_default_layout_value(
    value: Value,
) -> Result<WorkbenchLayout, LayoutPersistenceDocumentError> {
    Ok(decode_config_value::<DefaultLayoutDocument>(value)?.workbench)
}

pub(crate) fn encode_named_layout_presets_value(
    presets: BTreeMap<String, WorkbenchLayout>,
) -> Result<Value, LayoutPersistenceDocumentError> {
    encode_config_value(&NamedLayoutPresetsDocument { presets })
}

pub(crate) fn decode_named_layout_presets_value(
    value: Value,
) -> Result<BTreeMap<String, WorkbenchLayout>, LayoutPersistenceDocumentError> {
    Ok(decode_config_value::<NamedLayoutPresetsDocument>(value)?.presets)
}

pub(crate) fn encode_page_layout_presets_value(
    store: LayoutPresetPersistenceStore,
) -> Result<Value, LayoutPersistenceDocumentError> {
    encode_config_value(&PageLayoutPresetsDocument { store })
}

pub(crate) fn decode_page_layout_presets_value(
    value: Value,
) -> Result<LayoutPresetPersistenceStore, LayoutPersistenceDocumentError> {
    Ok(decode_config_value::<PageLayoutPresetsDocument>(value)?.store)
}

fn encode_config_value<T>(document: &T) -> Result<Value, LayoutPersistenceDocumentError>
where
    T: VersionedSchema + Serialize,
{
    let encoded = write_versioned_text(document)?;
    Ok(serde_json::from_str(&encoded)?)
}

fn decode_config_value<T>(value: Value) -> Result<T, LayoutPersistenceDocumentError>
where
    T: VersionedSchema + for<'de> Deserialize<'de> + 'static,
{
    let encoded = serde_json::to_vec(&value)?;
    Ok(load_versioned::<T>(&encoded, Format::Text)?.value)
}

impl VersionedSchema for DefaultLayoutDocument {
    const SCHEMA: SchemaId = SchemaId::new("zircon.editor.workbench.default-layout");
    const VERSION: u32 = 1;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<DefaultLayoutDocument> =
            MigrationChain::new(&[MigrationStep::new(0, reject_legacy_layout_document)]);
        &MIGRATIONS
    }
}

impl VersionedSchema for NamedLayoutPresetsDocument {
    const SCHEMA: SchemaId = SchemaId::new("zircon.editor.workbench.named-layout-presets");
    const VERSION: u32 = 1;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<NamedLayoutPresetsDocument> =
            MigrationChain::new(&[MigrationStep::new(0, reject_legacy_layout_document)]);
        &MIGRATIONS
    }
}

impl VersionedSchema for PageLayoutPresetsDocument {
    const SCHEMA: SchemaId = SchemaId::new("zircon.editor.workbench.page-layout-presets");
    const VERSION: u32 = 1;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<PageLayoutPresetsDocument> =
            MigrationChain::new(&[MigrationStep::new(0, reject_legacy_layout_document)]);
        &MIGRATIONS
    }
}

fn reject_legacy_layout_document(_value: Value) -> Result<Value, MigrateError> {
    Err(MigrateError::invalid_payload(
        "unversioned workbench layout documents are retired",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_layout_uses_the_current_version_shell_and_roundtrips() {
        let layout = WorkbenchLayout::default();

        let encoded = encode_default_layout_value(layout.clone()).unwrap();
        let header = &encoded["$zircon"]["header"];

        assert_eq!(header["schema_id"], DefaultLayoutDocument::SCHEMA.as_str());
        assert_eq!(header["schema_version"], DefaultLayoutDocument::VERSION);
        assert_eq!(decode_default_layout_value(encoded).unwrap(), layout);
    }

    #[test]
    fn raw_legacy_layout_is_rejected_instead_of_becoming_a_second_reader() {
        let legacy = serde_json::to_value(WorkbenchLayout::default()).unwrap();
        let legacy_named =
            serde_json::to_value(BTreeMap::<String, WorkbenchLayout>::new()).unwrap();
        let legacy_page = serde_json::to_value(LayoutPresetPersistenceStore::default()).unwrap();

        let error = decode_default_layout_value(legacy).unwrap_err();

        assert!(matches!(
            error,
            LayoutPersistenceDocumentError::Decode(LoadError::Migration(_))
        ));
        assert!(matches!(
            decode_named_layout_presets_value(legacy_named),
            Err(LayoutPersistenceDocumentError::Decode(
                LoadError::Migration(_)
            ))
        ));
        assert!(matches!(
            decode_page_layout_presets_value(legacy_page),
            Err(LayoutPersistenceDocumentError::Decode(
                LoadError::Migration(_)
            ))
        ));
    }

    #[test]
    fn layout_payload_kinds_have_distinct_schemas() {
        let default_layout = encode_default_layout_value(WorkbenchLayout::default()).unwrap();
        let mut named_presets = BTreeMap::new();
        named_presets.insert("authoring".to_string(), WorkbenchLayout::default());
        let named = encode_named_layout_presets_value(named_presets.clone()).unwrap();
        let page_store = LayoutPresetPersistenceStore::default();
        let page = encode_page_layout_presets_value(page_store.clone()).unwrap();

        let schema = |value: &Value| value["$zircon"]["header"]["schema_id"].clone();
        assert_ne!(schema(&default_layout), schema(&named));
        assert_ne!(schema(&default_layout), schema(&page));
        assert_ne!(schema(&named), schema(&page));
        assert!(decode_default_layout_value(named).is_err());
        assert_eq!(
            decode_named_layout_presets_value(
                encode_named_layout_presets_value(named_presets.clone()).unwrap()
            )
            .unwrap(),
            named_presets
        );
        assert_eq!(decode_page_layout_presets_value(page).unwrap(), page_store);
    }
}
