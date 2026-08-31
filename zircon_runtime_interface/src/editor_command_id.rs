use serde::{de, Deserialize, Deserializer, Serialize};

/// Canonical command/operation identity shared by plugin DTOs, the SDK, and the editor host.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct EditorCommandId(String);

impl EditorCommandId {
    pub fn parse(value: impl Into<String>) -> Result<Self, EditorCommandIdError> {
        let value = value.into();
        if !Self::is_valid(&value) {
            return Err(EditorCommandIdError(value));
        }
        Ok(Self(value))
    }

    pub fn is_valid(value: &str) -> bool {
        let mut segment_count = 1;
        let mut segment_is_empty = true;
        for byte in value.bytes() {
            if byte == b'.' {
                if segment_is_empty {
                    return false;
                }
                segment_count += 1;
                segment_is_empty = true;
            } else if command_id_byte(byte) {
                segment_is_empty = false;
            } else {
                return false;
            }
        }
        !segment_is_empty && segment_count >= MIN_COMMAND_ID_SEGMENTS
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for EditorCommandId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(de::Error::custom)
    }
}

impl std::fmt::Display for EditorCommandId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorCommandIdError(String);

impl EditorCommandIdError {
    pub fn into_value(self) -> String {
        self.0
    }
}

impl std::fmt::Display for EditorCommandIdError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "editor command id `{}` is invalid", self.0)
    }
}

impl std::error::Error for EditorCommandIdError {}

const MIN_COMMAND_ID_SEGMENTS: usize = 3;

fn command_id_byte(value: u8) -> bool {
    value.is_ascii_lowercase() || value.is_ascii_digit() || value == b'_'
}

#[cfg(test)]
mod tests {
    use super::EditorCommandId;

    #[test]
    fn command_id_golden_grammar_is_shared_at_the_wire_boundary() {
        for valid in [
            "editor.asset.open",
            "plugin_2.graph.compile",
            "view.editor.settings",
        ] {
            let id = EditorCommandId::parse(valid).unwrap();
            assert_eq!(id.as_str(), valid);
            assert_eq!(serde_json::to_string(&id).unwrap(), format!("\"{valid}\""));
        }
        for invalid in [
            "",
            "editor",
            "editor.asset",
            ".editor.asset",
            "editor..asset",
            "editor.asset.",
            "Editor.asset.open",
            "editor.asset-open",
        ] {
            assert!(EditorCommandId::parse(invalid).is_err(), "{invalid:?}");
            assert!(serde_json::from_str::<EditorCommandId>(&format!("\"{invalid}\"")).is_err());
        }
    }
}
