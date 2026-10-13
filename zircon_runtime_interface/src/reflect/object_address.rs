use serde::{Deserialize, Serialize};

use super::{type_path::validate_type_path, ReflectError};

/// 反射读写请求的组件或资源目标；构造器校验类型路径，是否已注册及实例是否存在仍由 World 反射入口检查。
///
/// 公开枚举变体可直接构造，且派生反序列化不重跑便利构造器的路径校验；由 World 执行目录查询时仍会检查目标是否注册。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ReflectObjectAddress {
    /// 指定实体上的组件；`entity` 为运行时实体 ID。
    Component { entity: u64, type_path: String },
    /// 指定 World 级资源。
    Resource { type_path: String },
}

impl ReflectObjectAddress {
    /// 校验类型路径并创建组件地址；实体是否存在留给 World 访问时检查。
    pub fn component(entity: u64, type_path: impl Into<String>) -> Result<Self, ReflectError> {
        let type_path = type_path.into();
        validate_type_path(&type_path)?;
        Ok(Self::Component { entity, type_path })
    }

    /// 校验类型路径并创建资源地址；资源是否已注册留给 World 访问时检查。
    pub fn resource(type_path: impl Into<String>) -> Result<Self, ReflectError> {
        let type_path = type_path.into();
        validate_type_path(&type_path)?;
        Ok(Self::Resource { type_path })
    }

    /// 返回两个地址形态共用的目标类型路径。
    pub fn type_path(&self) -> &str {
        match self {
            Self::Component { type_path, .. } | Self::Resource { type_path } => type_path,
        }
    }
}
