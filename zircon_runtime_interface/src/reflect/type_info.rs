use serde::{Deserialize, Serialize};

use super::{ReflectFieldInfo, ReflectTypeKind};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReflectTypeInfo {
    pub kind: ReflectTypeKind,
    pub fields: Vec<ReflectFieldInfo>,
}

impl ReflectTypeInfo {
    pub fn new(kind: ReflectTypeKind, fields: Vec<ReflectFieldInfo>) -> Self {
        Self { kind, fields }
    }

    pub fn struct_with_fields(fields: Vec<ReflectFieldInfo>) -> Self {
        Self::new(ReflectTypeKind::Struct, fields)
    }

    pub fn json_with_fields(fields: Vec<ReflectFieldInfo>) -> Self {
        Self::new(ReflectTypeKind::Json, fields)
    }

    pub fn opaque() -> Self {
        Self::new(ReflectTypeKind::Opaque, Vec::new())
    }
}
