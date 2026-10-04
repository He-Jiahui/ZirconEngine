use std::fs;
use std::path::Path;

use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiAssetMigrationOutcome, UiRawAssetPrototype,
};

use super::schema::{load_flat_prototype_toml_str, UiAssetSchemaMigrator};

/// 将文件或编辑器文本载入作者文档，失败时保留为资产错误交给宿主诊断。
/// 导入注册、组件展开和资源解析由后续阶段负责，加载成功不表示已经生成可运行表面。
#[derive(Default)]
pub struct UiAssetLoader;

impl UiAssetLoader {
    /// 走带迁移结果的文档入口后只取文档；需要向用户解释版本变化时使用保留报告的入口。
    pub fn load_toml_str(input: &str) -> Result<UiAssetDocument, UiAssetError> {
        Ok(Self::load_toml_str_with_migration_report(input)?.document)
    }

    /// 为原型存储与共享实例编译保留扁平句柄，避免先物化作者树再转回原型。
    pub fn load_flat_prototype_toml_str(input: &str) -> Result<UiRawAssetPrototype, UiAssetError> {
        load_flat_prototype_toml_str(input)
    }

    pub fn load_toml_str_with_migration_report(
        input: &str,
    ) -> Result<UiAssetMigrationOutcome, UiAssetError> {
        UiAssetSchemaMigrator::migrate_toml_str(input)
    }

    pub fn load_toml_file(path: impl AsRef<Path>) -> Result<UiAssetDocument, UiAssetError> {
        let input =
            fs::read_to_string(path).map_err(|error| UiAssetError::Io(error.to_string()))?;
        Self::load_toml_str(&input)
    }

    pub fn load_flat_prototype_toml_file(
        path: impl AsRef<Path>,
    ) -> Result<UiRawAssetPrototype, UiAssetError> {
        let input =
            fs::read_to_string(path).map_err(|error| UiAssetError::Io(error.to_string()))?;
        Self::load_flat_prototype_toml_str(&input)
    }

    pub fn load_toml_file_with_migration_report(
        path: impl AsRef<Path>,
    ) -> Result<UiAssetMigrationOutcome, UiAssetError> {
        let input =
            fs::read_to_string(path).map_err(|error| UiAssetError::Io(error.to_string()))?;
        Self::load_toml_str_with_migration_report(&input)
    }
}
