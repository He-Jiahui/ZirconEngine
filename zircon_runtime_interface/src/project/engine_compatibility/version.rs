use std::fmt::{Display, Formatter};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::ProjectEngineVersionParseError;

/// Canonical semantic version of the running editor engine selected for preflight.
/// 预检所用的运行中 Editor 引擎版本；序列化为文本，反序列化时再次按 semver 校验。
///
/// # 使用时机与生命周期
/// 宿主在项目预检时解析其选定版本，并将同一值传给兼容性评估；此包装类型不负责发现或更新宿主版本。
///
/// # 约束
/// 值只能通过 semver 解析建立；serde wire 形式为版本字符串，读取时重新执行相同解析。
///
/// # 参考
/// `zircon_editor/src/core/project/authority/preflight.rs` 从 `CARGO_PKG_VERSION` 创建准入输入；
/// `zircon_runtime_interface/src/project/tests/engine_compatibility.rs` 锁定字符串 wire 形式及往返结果。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectEngineVersion(semver::Version);

impl ProjectEngineVersion {
    /// 将宿主提供的版本文本解析为可用于项目要求比较的 semver 值。
    ///
    /// # Parameters
    /// - `value`：宿主选择的引擎版本文本。
    ///
    /// # Returns
    /// 成功时返回封装后的 [`semver::Version`]；显示时委托底层版本格式化。
    ///
    /// # Errors
    /// 输入不符合 semver 版本语法时返回 [`ProjectEngineVersionParseError::Invalid`]，并保留原文及解析原因。
    pub fn parse(value: impl Into<String>) -> Result<Self, ProjectEngineVersionParseError> {
        let value = value.into();
        semver::Version::parse(&value)
            .map(Self)
            .map_err(|source| ProjectEngineVersionParseError::Invalid { value, source })
    }

    /// 仅供同 crate 的兼容性实现借用内部版本，避免跨边界暴露 semver 存储类型。
    pub(crate) fn as_semver(&self) -> &semver::Version {
        &self.0
    }
}

/// 将版本按 semver 的显示形式写出；serde 使用相同文本作为 wire 值。
impl Display for ProjectEngineVersion {
    /// 委托内部 [`semver::Version`] 格式化，不改变解析后的版本。
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// 将包装类型编码为单个 semver 字符串，而不是版本字段对象。
impl Serialize for ProjectEngineVersion {
    /// 把显示形式交给 serde 序列化器；底层写入错误原样返回。
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// 从字符串 wire 值重建版本，并在构造包装值前重新进行 semver 校验。
impl<'de> Deserialize<'de> for ProjectEngineVersion {
    /// 先读取字符串，再复用 [`Self::parse`]；解析失败映射为 serde 数据错误。
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
