//! 配置存储与版本化布局文档之间的唯一编解码边界；默认布局、命名预设和页面用户预设使用独立模式身份。
//! 旧无版本文档被明确拒绝，宿主按各存储用途决定恢复内建布局或空预设。
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

// 写入全局默认布局的版本化文档；布局引用是否可恢复仍由宿主布局恢复链判断。
pub(crate) fn encode_default_layout_value(
    workbench: WorkbenchLayout,
) -> Result<Value, LayoutPersistenceDocumentError> {
    encode_config_value(&DefaultLayoutDocument { workbench })
}

// 仅按默认布局模式读取；错误交宿主处理，不能以其它预设模式或旧裸布局兜底读取。
pub(crate) fn decode_default_layout_value(
    value: Value,
) -> Result<WorkbenchLayout, LayoutPersistenceDocumentError> {
    Ok(decode_config_value::<DefaultLayoutDocument>(value)?.workbench)
}

// 写入按名称检索的整套布局快照，保留独立模式身份以拒绝错用存储槽。
pub(crate) fn encode_named_layout_presets_value(
    presets: BTreeMap<String, WorkbenchLayout>,
) -> Result<Value, LayoutPersistenceDocumentError> {
    encode_config_value(&NamedLayoutPresetsDocument { presets })
}

// 只读取命名预设文档；非法内容由宿主丢弃，不在此隐式生成默认预设。
pub(crate) fn decode_named_layout_presets_value(
    value: Value,
) -> Result<BTreeMap<String, WorkbenchLayout>, LayoutPersistenceDocumentError> {
    Ok(decode_config_value::<NamedLayoutPresetsDocument>(value)?.presets)
}

// 页面与用户联合身份的预设存储走专用模式，区别于全局默认布局和任意名称集合。
pub(crate) fn encode_page_layout_presets_value(
    store: LayoutPresetPersistenceStore,
) -> Result<Value, LayoutPersistenceDocumentError> {
    encode_config_value(&PageLayoutPresetsDocument { store })
}

// 返回已通过文档模式校验的预设存储；页面是否仍存在及布局规范化留给恢复端。
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

// 版本迁移链明确退休旧裸载荷；调用端采用已有fallback，不保留并行的legacy reader。
fn reject_legacy_layout_document(_value: Value) -> Result<Value, MigrateError> {
    Err(MigrateError::invalid_payload(
        "unversioned workbench layout documents are retired",
    ))
}

#[cfg(test)]
#[path = "tests/layout_persistence_document.rs"]
mod tests;
