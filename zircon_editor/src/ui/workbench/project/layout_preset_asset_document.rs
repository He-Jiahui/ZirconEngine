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

/// 借用布局写项目资源版本壳；与全局配置preset协议分开。
pub(super) fn encode_layout_preset_asset_document(
    workbench: &WorkbenchLayout,
) -> Result<String, WriteError> {
    write_versioned_text(&LayoutPresetAssetDocumentRef { workbench })
}

/// 读取当前项目资源协议；解码成功不替代恢复时的实例与宿主校验。
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
#[path = "tests/layout_preset_asset_document.rs"]
mod tests;
