use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use super::ReflectError;
use validation::{validate_module_path, validate_plugin_id, validate_short_type_path};

#[path = "type_path/validation.rs"]
mod validation;

pub(super) use validation::validate_type_path;
pub use validation::{
    MAX_REFLECT_MODULE_PATH_BYTES, MAX_REFLECT_PLUGIN_ID_BYTES, MAX_REFLECT_SHORT_TYPE_PATH_BYTES,
    MAX_REFLECT_TYPE_PATH_BYTES,
};

/// 已校验的完整类型路径及其短名；反序列化也重走相同的语法和长度约束。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReflectTypePath {
    type_path: String,
    short_type_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    module_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    plugin_id: Option<String>,
}

impl ReflectTypePath {
    /// 校验并创建完整路径及其短名；Rust 路径使用 `::`，VM 路径使用 `.`，短名须匹配末段。
    ///
    /// 路径语法和字节上限由本模块的校验器统一执行；模块路径和插件 ID 可在后续 builder 中附加。
    pub fn new(
        type_path: impl Into<String>,
        short_type_path: impl Into<String>,
    ) -> Result<Self, ReflectError> {
        let type_path = type_path.into();
        let short_type_path = short_type_path.into();
        validate_type_path(&type_path)?;
        validate_short_type_path(&type_path, &short_type_path)?;

        Ok(Self {
            type_path,
            short_type_path,
            module_path: None,
            plugin_id: None,
        })
    }

    /// 返回构造时校验的完整路径。
    pub fn type_path(&self) -> &str {
        self.type_path.as_str()
    }

    /// 返回与完整路径末段一致的短类型名。
    pub fn short_type_path(&self) -> &str {
        self.short_type_path.as_str()
    }

    /// 返回已验证且与完整路径前缀一致的模块路径；未附加时为 `None`。
    pub fn module_path(&self) -> Option<&str> {
        self.module_path.as_deref()
    }

    /// 返回可选的规范插件归属键。
    pub fn plugin_id(&self) -> Option<&str> {
        self.plugin_id.as_deref()
    }

    /// 校验模块路径与完整类型路径前缀一致后，返回附加该模块路径的新值。
    pub fn with_module_path(
        mut self,
        module_path: impl Into<String>,
    ) -> Result<Self, ReflectError> {
        let module_path = module_path.into();
        validate_module_path(&self.type_path, &module_path)?;
        self.module_path = Some(module_path);
        Ok(self)
    }

    /// 附加规范化的插件归属键；VM 注册端另核对完整类型路径的插件前缀。
    pub fn with_plugin_id(mut self, plugin_id: impl Into<String>) -> Result<Self, ReflectError> {
        let plugin_id = plugin_id.into();
        validate_plugin_id(&self.type_path, &plugin_id)?;
        self.plugin_id = Some(plugin_id);
        Ok(self)
    }
}

impl<'de> Deserialize<'de> for ReflectTypePath {
    /// 通过拒绝未知字段的 wire DTO 解码，并逐个调用统一构造器和可选键校验。
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 解码载体拒绝未知键；字段值随后通过相同构造器和 builder 逐项校验。
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct ReflectTypePathWire {
            type_path: String,
            short_type_path: String,
            #[serde(default)]
            module_path: Option<String>,
            #[serde(default)]
            plugin_id: Option<String>,
        }

        let wire = ReflectTypePathWire::deserialize(deserializer)?;
        let mut type_path =
            Self::new(wire.type_path, wire.short_type_path).map_err(D::Error::custom)?;
        if let Some(module_path) = wire.module_path {
            type_path = type_path
                .with_module_path(module_path)
                .map_err(D::Error::custom)?;
        }
        if let Some(plugin_id) = wire.plugin_id {
            type_path = type_path
                .with_plugin_id(plugin_id)
                .map_err(D::Error::custom)?;
        }
        Ok(type_path)
    }
}
