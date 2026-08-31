use std::fmt;
use std::num::NonZeroU64;
use std::sync::Arc;

use serde::de;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub const MAX_TOOL_INSTANCE_ID_BYTES: usize = 128;
const MAX_TOOL_OWNER_GENERATION_DIGITS: usize = 20;
const MAX_TOOL_INSTANCE_ORDINAL_DIGITS: usize = 20;
pub const MAX_TOOL_DEFINITION_ID_BYTES: usize = MAX_TOOL_INSTANCE_ID_BYTES
    - 2
    - MAX_TOOL_OWNER_GENERATION_DIGITS
    - MAX_TOOL_INSTANCE_ORDINAL_DIGITS;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ToolOwnerGeneration(NonZeroU64);

impl ToolOwnerGeneration {
    pub const BUILTIN: Self = Self(NonZeroU64::MIN);

    pub fn new(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(Self)
    }

    pub const fn value(self) -> u64 {
        self.0.get()
    }

    pub(crate) const fn checked_next(self) -> Option<Self> {
        match self.0.get().checked_add(1) {
            Some(value) => match NonZeroU64::new(value) {
                Some(value) => Some(Self(value)),
                None => None,
            },
            None => None,
        }
    }
}

impl fmt::Display for ToolOwnerGeneration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolDefinitionIdError {
    Empty,
    TooLong {
        actual_bytes: usize,
        max_bytes: usize,
    },
    InvalidCharacter {
        index: usize,
        character: char,
    },
}

impl fmt::Display for ToolDefinitionIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("tool definition id cannot be empty"),
            Self::TooLong {
                actual_bytes,
                max_bytes,
            } => write!(
                formatter,
                "tool definition id is {actual_bytes} bytes; maximum is {max_bytes} bytes"
            ),
            Self::InvalidCharacter { index, character } => write!(
                formatter,
                "tool definition id contains invalid character `{character}` at byte {index}"
            ),
        }
    }
}

impl std::error::Error for ToolDefinitionIdError {}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ToolDefinitionId(Arc<str>);

impl ToolDefinitionId {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, ToolDefinitionIdError> {
        let value = value.as_ref();
        if value.is_empty() {
            return Err(ToolDefinitionIdError::Empty);
        }
        if value.len() > MAX_TOOL_DEFINITION_ID_BYTES {
            return Err(ToolDefinitionIdError::TooLong {
                actual_bytes: value.len(),
                max_bytes: MAX_TOOL_DEFINITION_ID_BYTES,
            });
        }
        for (index, character) in value.char_indices() {
            let valid = character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_');
            if !valid {
                return Err(ToolDefinitionIdError::InvalidCharacter { index, character });
            }
        }
        Ok(Self(Arc::from(value)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ToolDefinitionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for ToolDefinitionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ToolDefinitionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolInstanceIdError {
    Definition(ToolDefinitionIdError),
    ZeroOwnerGeneration,
    ZeroOrdinal,
}

impl fmt::Display for ToolInstanceIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Definition(error) => error.fmt(formatter),
            Self::ZeroOwnerGeneration => {
                formatter.write_str("tool owner generation must be non-zero")
            }
            Self::ZeroOrdinal => formatter.write_str("tool instance ordinal must be non-zero"),
        }
    }
}

impl std::error::Error for ToolInstanceIdError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Definition(error) => Some(error),
            Self::ZeroOwnerGeneration | Self::ZeroOrdinal => None,
        }
    }
}

impl From<ToolDefinitionIdError> for ToolInstanceIdError {
    fn from(error: ToolDefinitionIdError) -> Self {
        Self::Definition(error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ToolInstanceId {
    definition: ToolDefinitionId,
    owner_generation: ToolOwnerGeneration,
    ordinal: NonZeroU64,
    qualified: Arc<str>,
}

impl ToolInstanceId {
    pub(crate) fn new(
        definition: ToolDefinitionId,
        owner_generation: ToolOwnerGeneration,
        ordinal: NonZeroU64,
    ) -> Self {
        let qualified = Arc::from(format!("{definition}@{owner_generation}.{ordinal}"));
        Self {
            definition,
            owner_generation,
            ordinal,
            qualified,
        }
    }

    pub fn from_parts(
        definition: impl AsRef<str>,
        owner_generation: u64,
        ordinal: u64,
    ) -> Result<Self, ToolInstanceIdError> {
        let definition = ToolDefinitionId::parse(definition)?;
        let owner_generation = ToolOwnerGeneration::new(owner_generation)
            .ok_or(ToolInstanceIdError::ZeroOwnerGeneration)?;
        let ordinal = NonZeroU64::new(ordinal).ok_or(ToolInstanceIdError::ZeroOrdinal)?;
        Ok(Self::new(definition, owner_generation, ordinal))
    }

    pub fn definition(&self) -> &ToolDefinitionId {
        &self.definition
    }

    pub const fn owner_generation(&self) -> ToolOwnerGeneration {
        self.owner_generation
    }

    pub const fn ordinal(&self) -> NonZeroU64 {
        self.ordinal
    }

    pub fn as_str(&self) -> &str {
        &self.qualified
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        definition: &str,
        owner_generation: ToolOwnerGeneration,
    ) -> Result<Self, ToolInstanceIdError> {
        Self::from_parts(definition, owner_generation.value(), 1)
    }
}

impl fmt::Display for ToolInstanceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for ToolInstanceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ToolInstanceId", 3)?;
        state.serialize_field("definition", &self.definition)?;
        state.serialize_field("owner_generation", &self.owner_generation)?;
        state.serialize_field("ordinal", &self.ordinal)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for ToolInstanceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SerializedToolInstanceId {
            definition: ToolDefinitionId,
            owner_generation: ToolOwnerGeneration,
            ordinal: NonZeroU64,
        }

        let value = SerializedToolInstanceId::deserialize(deserializer)?;
        Ok(Self::new(
            value.definition,
            value.owner_generation,
            value.ordinal,
        ))
    }
}

macro_rules! define_claim_id {
    ($name:ident) => {
        #[derive(
            Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(NonZeroU64);

        impl $name {
            pub const fn value(self) -> u64 {
                self.0.get()
            }

            pub(crate) const fn from_ordinal(ordinal: NonZeroU64) -> Self {
                Self(ordinal)
            }

            pub(crate) const fn checked_next(self) -> Option<Self> {
                match self.0.get().checked_add(1) {
                    Some(value) => match NonZeroU64::new(value) {
                        Some(value) => Some(Self(value)),
                        None => None,
                    },
                    None => None,
                }
            }

            pub(crate) const fn first() -> Self {
                Self::from_ordinal(NonZeroU64::MIN)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

define_claim_id!(ToolRequestId);
define_claim_id!(ToolLeaseId);
