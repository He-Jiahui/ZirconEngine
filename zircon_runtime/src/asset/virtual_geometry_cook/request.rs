use super::VirtualGeometryCookConfig;

const VIRTUAL_GEOMETRY_IMPORT_SETTING: &str = "virtual_geometry";
const ENABLED_IMPORT_SETTING: &str = "enabled";
const CLUSTER_TRIANGLE_COUNT_IMPORT_SETTING: &str = "cluster_triangle_count";
const PAGE_CLUSTER_COUNT_IMPORT_SETTING: &str = "page_cluster_count";

/// Typed import-time policy for optional virtual-geometry generation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum VirtualGeometryCookRequest {
    #[default]
    Disabled,
    Enabled(VirtualGeometryCookSettings),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 只控制烘焙分组密度；网格名和来源提示由每个导入原语在调用时附加。
pub struct VirtualGeometryCookSettings {
    pub cluster_triangle_count: usize,
    pub page_cluster_count: usize,
}

impl Default for VirtualGeometryCookSettings {
    fn default() -> Self {
        let config = VirtualGeometryCookConfig::default();
        Self {
            cluster_triangle_count: config.cluster_triangle_count,
            page_cluster_count: config.page_cluster_count,
        }
    }
}

impl VirtualGeometryCookRequest {
    /// Missing settings are disabled so MVP projects do not generate a payload
    /// without a runtime consumer.
    pub fn from_import_settings(import_settings: &toml::Table) -> Result<Self, String> {
        let Some(value) = import_settings.get(VIRTUAL_GEOMETRY_IMPORT_SETTING) else {
            return Ok(Self::Disabled);
        };
        let table = value.as_table().ok_or_else(|| {
            format!("import setting `{VIRTUAL_GEOMETRY_IMPORT_SETTING}` must be a table")
        })?;
        let enabled = required_bool(table, ENABLED_IMPORT_SETTING)?;
        if !enabled {
            return Ok(Self::Disabled);
        }

        let defaults = VirtualGeometryCookSettings::default();
        Ok(Self::Enabled(VirtualGeometryCookSettings {
            cluster_triangle_count: optional_positive_usize(
                table,
                CLUSTER_TRIANGLE_COUNT_IMPORT_SETTING,
                defaults.cluster_triangle_count,
            )?,
            page_cluster_count: optional_positive_usize(
                table,
                PAGE_CLUSTER_COUNT_IMPORT_SETTING,
                defaults.page_cluster_count,
            )?,
        }))
    }

    pub fn is_enabled(&self) -> bool {
        matches!(self, Self::Enabled(_))
    }

    /// 将一次导入的策略实例化为原语配置；未显式启用时调用方应跳过烘焙。
    pub fn cook_config_for(
        &self,
        mesh_name: Option<&str>,
        source_hint: &str,
    ) -> Option<VirtualGeometryCookConfig> {
        let Self::Enabled(settings) = self else {
            return None;
        };
        Some(VirtualGeometryCookConfig {
            cluster_triangle_count: settings.cluster_triangle_count,
            page_cluster_count: settings.page_cluster_count,
            mesh_name: mesh_name.map(str::to_owned),
            source_hint: Some(source_hint.to_string()),
        })
    }
}

fn required_bool(table: &toml::Table, key: &str) -> Result<bool, String> {
    table
        .get(key)
        .and_then(toml::Value::as_bool)
        .ok_or_else(|| {
            format!("import setting `{VIRTUAL_GEOMETRY_IMPORT_SETTING}.{key}` must be a boolean")
        })
}

fn optional_positive_usize(
    table: &toml::Table,
    key: &str,
    default: usize,
) -> Result<usize, String> {
    let Some(value) = table.get(key) else {
        return Ok(default);
    };
    let value = value.as_integer().ok_or_else(|| {
        format!(
            "import setting `{VIRTUAL_GEOMETRY_IMPORT_SETTING}.{key}` must be a positive integer"
        )
    })?;
    usize::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| {
            format!(
            "import setting `{VIRTUAL_GEOMETRY_IMPORT_SETTING}.{key}` must be a positive integer"
        )
        })
}

#[cfg(test)]
#[path = "tests/request.rs"]
mod tests;
