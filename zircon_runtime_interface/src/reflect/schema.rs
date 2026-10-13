use serde::{Deserialize, Serialize};

use super::{
    ReflectSchemaFingerprint, ReflectTypeRegistration, REFLECT_SCHEMA_CATALOG_ALGORITHM_VERSION,
};

/// 类型目录查询条件；类别标记均为 false 时不按角色筛选，插件类型仍需显式纳入。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflectSchemaFilter {
    /// 可选的精确类型路径；目录查询仍会叠加其余启用的过滤条件。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_path: Option<String>,
    /// 纳入组件角色；与资源角色开关按“任一启用即匹配”组合。
    pub include_components: bool,
    /// 纳入资源角色；与组件角色开关按“任一启用即匹配”组合。
    pub include_resources: bool,
    /// 要求注册声明对编辑器可见。
    pub editor_visible: bool,
    /// 要求注册声明对远端调用可见。
    pub remote_visible: bool,
    /// 是否把插件所有的类型纳入候选目录。
    pub include_plugin_owned: bool,
}

impl ReflectSchemaFilter {
    /// 创建供编辑器浏览的过滤条件，纳入组件和资源并要求编辑器可见。
    pub fn editor_visible() -> Self {
        Self {
            include_components: true,
            include_resources: true,
            editor_visible: true,
            ..Self::default()
        }
    }

    /// 创建供远端浏览的过滤条件，纳入组件和资源并要求远端可见。
    pub fn remote_visible() -> Self {
        Self {
            include_components: true,
            include_resources: true,
            remote_visible: true,
            ..Self::default()
        }
    }

    /// 创建指定类型路径的查询条件，同时允许组件和资源角色。
    pub fn for_type(type_path: impl Into<String>) -> Self {
        Self {
            type_path: Some(type_path.into()),
            include_components: true,
            include_resources: true,
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 反射类型目录查询请求。
pub struct ReflectSchemaRequest {
    /// 目录筛选条件。
    pub filter: ReflectSchemaFilter,
}

impl ReflectSchemaRequest {
    /// 使用完整筛选条件创建查询请求。
    pub fn new(filter: ReflectSchemaFilter) -> Self {
        Self { filter }
    }

    /// 创建编辑器可见类型查询请求。
    pub fn editor_visible() -> Self {
        Self::new(ReflectSchemaFilter::editor_visible())
    }

    /// 创建远端可见类型查询请求。
    pub fn remote_visible() -> Self {
        Self::new(ReflectSchemaFilter::remote_visible())
    }

    /// 创建指定路径查询请求；实际目录查询仍会应用筛选条件。
    pub fn for_type(type_path: impl Into<String>) -> Self {
        Self::new(ReflectSchemaFilter::for_type(type_path))
    }
}

/// 运行时查询可过滤返回的注册，但仍附带当期完整目录的指纹。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReflectSchemaResponse {
    /// 生成当前目录指纹所用的算法版本。
    pub catalog_algorithm_version: u32,
    /// 查询时完整注册目录的指纹，即使 `registrations` 已按请求筛选。
    pub catalog_fingerprint: ReflectSchemaFingerprint,
    /// 满足查询筛选条件的注册项，不意味着目录指纹只覆盖这些项。
    pub registrations: Vec<ReflectTypeRegistration>,
}

impl ReflectSchemaResponse {
    /// 构造目录响应，并记录当前 schema catalog 指纹算法版本。
    pub fn new(
        catalog_fingerprint: ReflectSchemaFingerprint,
        registrations: Vec<ReflectTypeRegistration>,
    ) -> Self {
        Self {
            catalog_algorithm_version: REFLECT_SCHEMA_CATALOG_ALGORITHM_VERSION,
            catalog_fingerprint,
            registrations,
        }
    }
}
