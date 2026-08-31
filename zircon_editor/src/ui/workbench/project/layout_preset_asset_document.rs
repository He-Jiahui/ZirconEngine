use serde::{Deserialize, Serialize};
use serde_json::Value;
use zircon_runtime_interface::serialization::{
    load_versioned, write_versioned_text, Format, LoadError, MigrateError, MigrationChain,
    MigrationStep, SchemaId, VersionedSchema, WriteError,
};

use crate::ui::workbench::layout::WorkbenchLayout;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::ui::workbench::project) struct LayoutPresetAssetDocument {
    pub(in crate::ui::workbench::project) workbench: WorkbenchLayout,
}

#[derive(Serialize)]
struct LayoutPresetAssetDocumentRef<'layout> {
    workbench: &'layout WorkbenchLayout,
}

pub(super) fn encode_layout_preset_asset_document(
    workbench: &WorkbenchLayout,
) -> Result<String, WriteError> {
    write_versioned_text(&LayoutPresetAssetDocumentRef { workbench })
}

pub(super) fn decode_layout_preset_asset_document(
    source: &[u8],
) -> Result<WorkbenchLayout, LoadError> {
    Ok(
        load_versioned::<LayoutPresetAssetDocument>(source, Format::Text)?
            .value
            .workbench,
    )
}

impl VersionedSchema for LayoutPresetAssetDocument {
    const SCHEMA: SchemaId = SchemaId::new("zircon.editor.workbench.project-layout-preset");
    const VERSION: u32 = 1;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<LayoutPresetAssetDocument> =
            MigrationChain::new(&[MigrationStep::new(0, reject_legacy_layout_preset_asset)]);
        &MIGRATIONS
    }
}

impl<'layout> VersionedSchema for LayoutPresetAssetDocumentRef<'layout> {
    const SCHEMA: SchemaId = LayoutPresetAssetDocument::SCHEMA;
    const VERSION: u32 = LayoutPresetAssetDocument::VERSION;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<LayoutPresetAssetDocumentRef<'static>> =
            MigrationChain::new(&[MigrationStep::new(0, reject_legacy_layout_preset_asset)]);
        &MIGRATIONS
    }
}

fn reject_legacy_layout_preset_asset(_value: Value) -> Result<Value, MigrateError> {
    Err(MigrateError::invalid_payload(
        "unversioned project layout preset assets are retired",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_layout_preset_uses_the_current_version_shell_and_roundtrips() {
        let layout = WorkbenchLayout::default();

        let encoded = encode_layout_preset_asset_document(&layout).unwrap();

        assert!(encoded.contains(LayoutPresetAssetDocument::SCHEMA.as_str()));
        assert_eq!(
            decode_layout_preset_asset_document(encoded.as_bytes()).unwrap(),
            layout
        );
    }

    #[test]
    fn unversioned_project_layout_preset_is_rejected() {
        let legacy = serde_json::to_vec(&LayoutPresetAssetDocument {
            workbench: WorkbenchLayout::default(),
        })
        .unwrap();

        assert!(matches!(
            decode_layout_preset_asset_document(&legacy),
            Err(LoadError::MissingTextEnvelope { .. })
        ));
    }
}
