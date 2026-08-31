use std::fmt;

use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use thiserror::Error;

use crate::ui::component::UiValueKind;

pub const UI_MODEL_IDENTITY_MAX_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiModelIdentityKind {
    Schema,
    Field,
    Provider,
}

impl fmt::Display for UiModelIdentityKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Schema => "model schema",
            Self::Field => "model field",
            Self::Provider => "model provider",
        })
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiModelIdentityError {
    #[error("{kind} identity cannot be empty")]
    Empty { kind: UiModelIdentityKind },
    #[error("{kind} identity uses {actual_bytes} bytes, exceeding the {maximum_bytes}-byte limit")]
    TooLong {
        kind: UiModelIdentityKind,
        actual_bytes: usize,
        maximum_bytes: usize,
    },
    #[error("{kind} identity contains an empty segment at index {segment_index}")]
    EmptySegment {
        kind: UiModelIdentityKind,
        segment_index: usize,
    },
    #[error("{kind} identity contains invalid character `{character}` at byte {byte_index}")]
    InvalidCharacter {
        kind: UiModelIdentityKind,
        character: char,
        byte_index: usize,
    },
}

impl UiModelIdentityError {
    pub const fn kind(&self) -> UiModelIdentityKind {
        match self {
            Self::Empty { kind }
            | Self::TooLong { kind, .. }
            | Self::EmptySegment { kind, .. }
            | Self::InvalidCharacter { kind, .. } => *kind,
        }
    }
}

macro_rules! model_identity {
    ($name:ident, $kind:expr) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn try_new(value: impl Into<String>) -> Result<Self, UiModelIdentityError> {
                let value = value.into();
                validate_model_identity($kind, &value)?;
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::try_new(value).map_err(D::Error::custom)
            }
        }
    };
}

model_identity!(UiModelSchemaId, UiModelIdentityKind::Schema);
model_identity!(UiModelFieldId, UiModelIdentityKind::Field);
model_identity!(UiModelProviderId, UiModelIdentityKind::Provider);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiModelVersionKind {
    Schema,
    Provider,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("{kind:?} version must be non-zero")]
pub struct UiModelVersionError {
    pub kind: UiModelVersionKind,
}

macro_rules! model_version {
    ($name:ident, $kind:expr) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(u64);

        impl $name {
            pub const fn try_new(value: u64) -> Result<Self, UiModelVersionError> {
                if value == 0 {
                    Err(UiModelVersionError { kind: $kind })
                } else {
                    Ok(Self(value))
                }
            }

            pub const fn get(self) -> u64 {
                self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = u64::deserialize(deserializer)?;
                Self::try_new(value).map_err(D::Error::custom)
            }
        }
    };
}

model_version!(UiModelSchemaVersion, UiModelVersionKind::Schema);
model_version!(UiModelProviderVersion, UiModelVersionKind::Provider);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiModelSchemaKey {
    pub id: UiModelSchemaId,
    pub version: UiModelSchemaVersion,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiModelProviderKey {
    pub id: UiModelProviderId,
    pub version: UiModelProviderVersion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiModelFieldAccess {
    ReadOnly,
    ReadWrite,
}

impl UiModelFieldAccess {
    pub const fn readable(self) -> bool {
        true
    }

    pub const fn writable(self) -> bool {
        matches!(self, Self::ReadWrite)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiModelFieldSchema {
    pub id: UiModelFieldId,
    pub value_kind: UiValueKind,
    pub access: UiModelFieldAccess,
}

impl UiModelFieldSchema {
    pub fn new(id: UiModelFieldId, value_kind: UiValueKind, access: UiModelFieldAccess) -> Self {
        Self {
            id,
            value_kind,
            access,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiModelSchema {
    #[serde(flatten)]
    pub key: UiModelSchemaKey,
    pub fields: Vec<UiModelFieldSchema>,
}

impl UiModelSchema {
    pub fn new(
        id: UiModelSchemaId,
        version: UiModelSchemaVersion,
        fields: Vec<UiModelFieldSchema>,
    ) -> Self {
        Self {
            key: UiModelSchemaKey { id, version },
            fields,
        }
    }

    pub fn key(&self) -> &UiModelSchemaKey {
        &self.key
    }

    pub fn field(&self, field_id: &UiModelFieldId) -> Option<&UiModelFieldSchema> {
        self.fields.iter().find(|field| field.id == *field_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiModelProviderSchema {
    #[serde(flatten)]
    pub key: UiModelProviderKey,
    pub model_schema: UiModelSchemaKey,
}

impl UiModelProviderSchema {
    pub fn new(
        id: UiModelProviderId,
        version: UiModelProviderVersion,
        model_schema: UiModelSchemaKey,
    ) -> Self {
        Self {
            key: UiModelProviderKey { id, version },
            model_schema,
        }
    }

    pub fn key(&self) -> &UiModelProviderKey {
        &self.key
    }

    pub fn id(&self) -> &UiModelProviderId {
        &self.key.id
    }

    pub const fn version(&self) -> UiModelProviderVersion {
        self.key.version
    }
}

fn validate_model_identity(
    kind: UiModelIdentityKind,
    value: &str,
) -> Result<(), UiModelIdentityError> {
    if value.is_empty() {
        return Err(UiModelIdentityError::Empty { kind });
    }
    if value.len() > UI_MODEL_IDENTITY_MAX_BYTES {
        return Err(UiModelIdentityError::TooLong {
            kind,
            actual_bytes: value.len(),
            maximum_bytes: UI_MODEL_IDENTITY_MAX_BYTES,
        });
    }
    for (segment_index, segment) in value.split('.').enumerate() {
        if segment.is_empty() {
            return Err(UiModelIdentityError::EmptySegment {
                kind,
                segment_index,
            });
        }
    }
    for (byte_index, character) in value.char_indices() {
        if !(character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')) {
            return Err(UiModelIdentityError::InvalidCharacter {
                kind,
                character,
                byte_index,
            });
        }
    }
    Ok(())
}
