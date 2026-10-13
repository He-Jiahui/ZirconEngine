use serde::{Deserialize, Serialize};

use super::{ReflectFieldInfo, ReflectTypeKind};

/// 一个反射类型的形状标签和声明字段序列；字段次序也供当前目录槽位映射使用。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReflectTypeInfo {
    /// 类型的高层形状标签。
    pub kind: ReflectTypeKind,
    /// 按注册顺序排列的字段元数据；目录接纳时会验证重复名、身份和槽位关系。
    pub fields: Vec<ReflectFieldInfo>,
}

impl ReflectTypeInfo {
    /// 直接组合指定形状及字段序列，不在本构造器中做目录接纳校验。
    pub fn new(kind: ReflectTypeKind, fields: Vec<ReflectFieldInfo>) -> Self {
        Self { kind, fields }
    }

    /// 创建结构体形状的类型信息。
    pub fn struct_with_fields(fields: Vec<ReflectFieldInfo>) -> Self {
        Self::new(ReflectTypeKind::Struct, fields)
    }

    /// 创建 JSON 形状的类型信息。
    pub fn json_with_fields(fields: Vec<ReflectFieldInfo>) -> Self {
        Self::new(ReflectTypeKind::Json, fields)
    }

    /// 创建无字段的不透明类型信息。
    pub fn opaque() -> Self {
        Self::new(ReflectTypeKind::Opaque, Vec::new())
    }
}
