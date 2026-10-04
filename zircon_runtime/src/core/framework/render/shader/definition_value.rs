use serde::{Deserialize, Deserializer, Serialize};

/// 模板组装使用的有类型定义值；旧资产可用裸字符串表示 true 布尔标志，写回时统一为带类型形式。
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RenderShaderDefinitionValue {
    #[serde(rename = "bool")]
    Bool { name: String, value: bool },
    #[serde(rename = "int")]
    Int { name: String, value: i32 },
    #[serde(rename = "uint")]
    UInt { name: String, value: u32 },
}

impl RenderShaderDefinitionValue {
    pub fn bool(name: impl Into<String>, value: bool) -> Self {
        Self::Bool {
            name: name.into(),
            value,
        }
    }

    pub fn int(name: impl Into<String>, value: i32) -> Self {
        Self::Int {
            name: name.into(),
            value,
        }
    }

    pub fn uint(name: impl Into<String>, value: u32) -> Self {
        Self::UInt {
            name: name.into(),
            value,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Bool { name, .. } | Self::Int { name, .. } | Self::UInt { name, .. } => name,
        }
    }

    /// Returns the normalized view without copying the owned definition name.
    pub fn normalized_name(&self) -> &str {
        self.name().trim()
    }

    pub fn value_as_string(&self) -> String {
        match self {
            Self::Bool { value, .. } => value.to_string(),
            Self::Int { value, .. } => value.to_string(),
            Self::UInt { value, .. } => value.to_string(),
        }
    }
}

impl From<&str> for RenderShaderDefinitionValue {
    fn from(name: &str) -> Self {
        Self::bool(name, true)
    }
}

impl From<String> for RenderShaderDefinitionValue {
    fn from(name: String) -> Self {
        Self::bool(name, true)
    }
}

// 读取阶段兼容旧的裸标志，避免既有资产在新的类型化定义格式下失效。
impl<'de> Deserialize<'de> for RenderShaderDefinitionValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case")]
        enum TaggedDefinitionValue {
            #[serde(rename = "bool")]
            Bool { name: String, value: bool },
            #[serde(rename = "int")]
            Int { name: String, value: i32 },
            #[serde(rename = "uint")]
            UInt { name: String, value: u32 },
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum DefinitionValueRepr {
            BareFlag(String),
            Tagged(TaggedDefinitionValue),
        }

        Ok(match DefinitionValueRepr::deserialize(deserializer)? {
            DefinitionValueRepr::BareFlag(name) => Self::from(name),
            DefinitionValueRepr::Tagged(TaggedDefinitionValue::Bool { name, value }) => {
                Self::bool(name, value)
            }
            DefinitionValueRepr::Tagged(TaggedDefinitionValue::Int { name, value }) => {
                Self::int(name, value)
            }
            DefinitionValueRepr::Tagged(TaggedDefinitionValue::UInt { name, value }) => {
                Self::uint(name, value)
            }
        })
    }
}

#[cfg(test)]
#[path = "tests/definition_value.rs"]
mod tests;
