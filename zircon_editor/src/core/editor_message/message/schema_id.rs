use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub(crate) const MAX_EDITOR_MESSAGE_SCHEMA_ID_BYTES: usize = 256;

const ROOT_NAMESPACE: &str = "zircon";
const EDITOR_NAMESPACE: &str = "editor";
const PLUGIN_NAMESPACE: &str = "plugin";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EditorMessageSchemaId(Arc<str>);

impl EditorMessageSchemaId {
    pub(crate) fn editor(
        local_schema: impl AsRef<str>,
    ) -> Result<Self, EditorMessageSchemaIdError> {
        Self::parse(format!(
            "{ROOT_NAMESPACE}.{EDITOR_NAMESPACE}.{}",
            local_schema.as_ref()
        ))
    }

    pub(crate) fn plugin(
        plugin_id: impl AsRef<str>,
        local_schema: impl AsRef<str>,
    ) -> Result<Self, EditorMessageSchemaIdError> {
        Self::parse(format!(
            "{ROOT_NAMESPACE}.{PLUGIN_NAMESPACE}.{}.{}",
            plugin_id.as_ref(),
            local_schema.as_ref()
        ))
    }

    pub(crate) fn parse(value: impl Into<String>) -> Result<Self, EditorMessageSchemaIdError> {
        let value = value.into();
        if value.is_empty() {
            return Err(EditorMessageSchemaIdError::Empty);
        }
        if value.len() > MAX_EDITOR_MESSAGE_SCHEMA_ID_BYTES {
            return Err(EditorMessageSchemaIdError::TooLong {
                max_bytes: MAX_EDITOR_MESSAGE_SCHEMA_ID_BYTES,
                actual_bytes: value.len(),
            });
        }

        let mut segment_count = 0usize;
        let mut root = None;
        let mut namespace = None;
        for (index, segment) in value.split('.').enumerate() {
            if segment.is_empty() {
                return Err(EditorMessageSchemaIdError::EmptySegment { index });
            }
            if !segment_is_valid(segment) {
                return Err(EditorMessageSchemaIdError::InvalidSegment {
                    segment: segment.to_owned(),
                });
            }
            if index == 0 {
                root = Some(segment);
            } else if index == 1 {
                namespace = Some(segment);
            }
            segment_count += 1;
        }

        match (root, namespace, segment_count) {
            (Some(ROOT_NAMESPACE), Some(EDITOR_NAMESPACE), count) if count >= 3 => {}
            (Some(ROOT_NAMESPACE), Some(PLUGIN_NAMESPACE), count) if count >= 4 => {}
            (Some(ROOT_NAMESPACE), Some(EDITOR_NAMESPACE), _) => {
                return Err(EditorMessageSchemaIdError::MissingEditorSchema)
            }
            (Some(ROOT_NAMESPACE), Some(PLUGIN_NAMESPACE), _) => {
                return Err(EditorMessageSchemaIdError::MissingPluginIdentityOrSchema)
            }
            _ => {
                return Err(EditorMessageSchemaIdError::UnsupportedNamespace {
                    value: value.clone(),
                });
            }
        }

        Ok(Self(Arc::from(value)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}

impl fmt::Display for EditorMessageSchemaId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for EditorMessageSchemaId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for EditorMessageSchemaId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum EditorMessageSchemaIdError {
    Empty,
    TooLong {
        max_bytes: usize,
        actual_bytes: usize,
    },
    EmptySegment {
        index: usize,
    },
    InvalidSegment {
        segment: String,
    },
    MissingEditorSchema,
    MissingPluginIdentityOrSchema,
    UnsupportedNamespace {
        value: String,
    },
}

impl fmt::Display for EditorMessageSchemaIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("editor message schema id is empty"),
            Self::TooLong {
                max_bytes,
                actual_bytes,
            } => write!(
                formatter,
                "editor message schema id uses {actual_bytes} bytes but the limit is {max_bytes}"
            ),
            Self::EmptySegment { index } => {
                write!(
                    formatter,
                    "editor message schema id segment {index} is empty"
                )
            }
            Self::InvalidSegment { segment } => write!(
                formatter,
                "editor message schema id segment `{segment}` is invalid"
            ),
            Self::MissingEditorSchema => {
                formatter.write_str("zircon.editor schema id requires a local schema")
            }
            Self::MissingPluginIdentityOrSchema => formatter.write_str(
                "zircon.plugin schema id requires both a plugin identity and a local schema",
            ),
            Self::UnsupportedNamespace { value } => write!(
                formatter,
                "editor message schema id `{value}` is outside zircon.editor or zircon.plugin"
            ),
        }
    }
}

impl std::error::Error for EditorMessageSchemaIdError {}

fn segment_is_valid(segment: &str) -> bool {
    segment.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
    })
}
