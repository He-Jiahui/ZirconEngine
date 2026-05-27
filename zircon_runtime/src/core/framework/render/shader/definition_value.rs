use serde::{Deserialize, Deserializer, Serialize};

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

    pub fn normalized_name(&self) -> String {
        self.name().trim().to_string()
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
            LegacyFlag(String),
            Tagged(TaggedDefinitionValue),
        }

        Ok(match DefinitionValueRepr::deserialize(deserializer)? {
            DefinitionValueRepr::LegacyFlag(name) => Self::from(name),
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
