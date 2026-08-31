use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Maximum input retained for one editor command invocation.
pub const MAX_EDITOR_COMMAND_INPUT_BYTES: usize = 64 * 1024 * 1024;
/// Maximum output retained for one editor command result.
pub const MAX_EDITOR_COMMAND_OUTPUT_BYTES: usize = 256 * 1024 * 1024;
/// Maximum wall-clock budget accepted for one editor command invocation.
pub const MAX_EDITOR_COMMAND_EXECUTION_TIME_MS: u64 = 5 * 60 * 1000;

/// Stable, versioned identity of the codec used to decode an editor command result.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EditorCommandResultCodecId(String);

impl EditorCommandResultCodecId {
    pub fn parse(value: impl Into<String>) -> Result<Self, EditorCommandResultCodecIdError> {
        let value = value.into();
        validate_codec_id(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EditorCommandResultCodecId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for EditorCommandResultCodecId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for EditorCommandResultCodecId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(serde::de::Error::custom)
    }
}

/// Bounded resources available to one command endpoint invocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct EditorCommandResourceBudget {
    max_input_bytes: usize,
    max_output_bytes: usize,
    max_execution_time_ms: u64,
}

impl EditorCommandResourceBudget {
    pub fn new(
        max_input_bytes: usize,
        max_output_bytes: usize,
        max_execution_time_ms: u64,
    ) -> Result<Self, EditorCommandResourceBudgetError> {
        let budget = Self {
            max_input_bytes,
            max_output_bytes,
            max_execution_time_ms,
        };
        budget.validate()?;
        Ok(budget)
    }

    pub fn max_input_bytes(self) -> usize {
        self.max_input_bytes
    }

    pub fn max_output_bytes(self) -> usize {
        self.max_output_bytes
    }

    pub fn max_execution_time_ms(self) -> u64 {
        self.max_execution_time_ms
    }

    pub fn validate(self) -> Result<(), EditorCommandResourceBudgetError> {
        if self.max_input_bytes > MAX_EDITOR_COMMAND_INPUT_BYTES {
            return Err(EditorCommandResourceBudgetError::InputLimitTooLarge {
                limit: self.max_input_bytes,
                maximum: MAX_EDITOR_COMMAND_INPUT_BYTES,
            });
        }
        if self.max_output_bytes > MAX_EDITOR_COMMAND_OUTPUT_BYTES {
            return Err(EditorCommandResourceBudgetError::OutputLimitTooLarge {
                limit: self.max_output_bytes,
                maximum: MAX_EDITOR_COMMAND_OUTPUT_BYTES,
            });
        }
        if self.max_execution_time_ms == 0 {
            return Err(EditorCommandResourceBudgetError::ExecutionTimeZero);
        }
        if self.max_execution_time_ms > MAX_EDITOR_COMMAND_EXECUTION_TIME_MS {
            return Err(EditorCommandResourceBudgetError::ExecutionTimeTooLong {
                limit: self.max_execution_time_ms,
                maximum: MAX_EDITOR_COMMAND_EXECUTION_TIME_MS,
            });
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for EditorCommandResourceBudget {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RawBudget {
            max_input_bytes: usize,
            max_output_bytes: usize,
            max_execution_time_ms: u64,
        }

        let raw = RawBudget::deserialize(deserializer)?;
        Self::new(
            raw.max_input_bytes,
            raw.max_output_bytes,
            raw.max_execution_time_ms,
        )
        .map_err(serde::de::Error::custom)
    }
}

/// Immutable execution metadata shared by command definitions and endpoint bindings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditorCommandExecutionContract {
    result_codec: EditorCommandResultCodecId,
    resource_budget: EditorCommandResourceBudget,
}

impl EditorCommandExecutionContract {
    pub fn new(
        result_codec: EditorCommandResultCodecId,
        resource_budget: EditorCommandResourceBudget,
    ) -> Self {
        Self {
            result_codec,
            resource_budget,
        }
    }

    pub fn result_codec(&self) -> &EditorCommandResultCodecId {
        &self.result_codec
    }

    pub fn resource_budget(&self) -> EditorCommandResourceBudget {
        self.resource_budget
    }

    pub fn validate(&self) -> Result<(), EditorCommandResourceBudgetError> {
        self.resource_budget.validate()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorCommandResultCodecIdError {
    Empty,
    TooLong { actual_bytes: usize, maximum: usize },
    EmptySegment { index: usize },
    InvalidSegment { segment: String },
    MissingNamespace,
    MissingVersion,
    InvalidVersion { segment: String },
}

impl fmt::Display for EditorCommandResultCodecIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("editor command result codec id is empty"),
            Self::TooLong {
                actual_bytes,
                maximum,
            } => write!(
                formatter,
                "editor command result codec id uses {actual_bytes} bytes; maximum is {maximum}"
            ),
            Self::EmptySegment { index } => write!(
                formatter,
                "editor command result codec id segment {index} is empty"
            ),
            Self::InvalidSegment { segment } => write!(
                formatter,
                "editor command result codec id segment `{segment}` is invalid"
            ),
            Self::MissingNamespace => formatter.write_str(
                "editor command result codec id must contain at least two namespace segments",
            ),
            Self::MissingVersion => formatter.write_str(
                "editor command result codec id must end with a version segment such as `v1`",
            ),
            Self::InvalidVersion { segment } => write!(
                formatter,
                "editor command result codec id version segment `{segment}` is invalid"
            ),
        }
    }
}

impl std::error::Error for EditorCommandResultCodecIdError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorCommandResourceBudgetError {
    InputLimitTooLarge { limit: usize, maximum: usize },
    OutputLimitTooLarge { limit: usize, maximum: usize },
    ExecutionTimeZero,
    ExecutionTimeTooLong { limit: u64, maximum: u64 },
}

impl fmt::Display for EditorCommandResourceBudgetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputLimitTooLarge { limit, maximum } => write!(
                formatter,
                "editor command input budget {limit} bytes exceeds maximum {maximum}"
            ),
            Self::OutputLimitTooLarge { limit, maximum } => write!(
                formatter,
                "editor command output budget {limit} bytes exceeds maximum {maximum}"
            ),
            Self::ExecutionTimeZero => formatter
                .write_str("editor command execution time budget must be greater than zero"),
            Self::ExecutionTimeTooLong { limit, maximum } => write!(
                formatter,
                "editor command execution time budget {limit} ms exceeds maximum {maximum} ms"
            ),
        }
    }
}

impl std::error::Error for EditorCommandResourceBudgetError {}

const MAX_RESULT_CODEC_ID_BYTES: usize = 256;

fn validate_codec_id(value: &str) -> Result<(), EditorCommandResultCodecIdError> {
    if value.is_empty() {
        return Err(EditorCommandResultCodecIdError::Empty);
    }
    if value.len() > MAX_RESULT_CODEC_ID_BYTES {
        return Err(EditorCommandResultCodecIdError::TooLong {
            actual_bytes: value.len(),
            maximum: MAX_RESULT_CODEC_ID_BYTES,
        });
    }

    let mut segment_count = 0;
    for segment in value.split('.') {
        if segment.is_empty() {
            return Err(EditorCommandResultCodecIdError::EmptySegment {
                index: segment_count,
            });
        }
        if !segment.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        }) {
            return Err(EditorCommandResultCodecIdError::InvalidSegment {
                segment: segment.to_owned(),
            });
        }
        segment_count += 1;
    }
    if segment_count < 3 {
        return Err(EditorCommandResultCodecIdError::MissingNamespace);
    }

    let version = value.rsplit('.').next().unwrap_or_default();
    if !version.starts_with('v') {
        return Err(EditorCommandResultCodecIdError::MissingVersion);
    }
    let digits = &version[1..];
    if digits.is_empty()
        || (digits.len() > 1 && digits.starts_with('0'))
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
        || digits.parse::<u64>().ok().is_none_or(|value| value == 0)
    {
        return Err(EditorCommandResultCodecIdError::InvalidVersion {
            segment: version.to_owned(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_codec_id_requires_a_namespaced_positive_version() {
        let codec = EditorCommandResultCodecId::parse("zircon.editor.command-result.v1")
            .expect("versioned result codec id should parse");
        assert_eq!(codec.as_str(), "zircon.editor.command-result.v1");
        for invalid in [
            "zircon.editor.result",
            "zircon.editor.result.v0",
            "zircon.editor.result.v01",
            "Zircon.editor.result.v1",
            "zircon..result.v1",
        ] {
            assert!(
                EditorCommandResultCodecId::parse(invalid).is_err(),
                "codec id `{invalid}` must be rejected"
            );
        }
    }

    #[test]
    fn resource_budget_enforces_finite_input_output_and_time_limits() {
        let budget = EditorCommandResourceBudget::new(
            MAX_EDITOR_COMMAND_INPUT_BYTES,
            0,
            MAX_EDITOR_COMMAND_EXECUTION_TIME_MS,
        )
        .expect("zero output is valid for a command without a result payload");
        assert_eq!(budget.max_output_bytes(), 0);
        assert!(matches!(
            EditorCommandResourceBudget::new(0, 0, 0),
            Err(EditorCommandResourceBudgetError::ExecutionTimeZero)
        ));
        assert!(matches!(
            EditorCommandResourceBudget::new(MAX_EDITOR_COMMAND_INPUT_BYTES + 1, 0, 1),
            Err(EditorCommandResourceBudgetError::InputLimitTooLarge { .. })
        ));
        assert!(matches!(
            EditorCommandResourceBudget::new(0, MAX_EDITOR_COMMAND_OUTPUT_BYTES + 1, 1),
            Err(EditorCommandResourceBudgetError::OutputLimitTooLarge { .. })
        ));
        assert!(matches!(
            EditorCommandResourceBudget::new(0, 0, MAX_EDITOR_COMMAND_EXECUTION_TIME_MS + 1),
            Err(EditorCommandResourceBudgetError::ExecutionTimeTooLong { .. })
        ));
    }

    #[test]
    fn execution_contract_roundtrips_and_rejects_invalid_budget_on_decode() {
        let contract = EditorCommandExecutionContract::new(
            EditorCommandResultCodecId::parse("zircon.editor.command-result.v1").unwrap(),
            EditorCommandResourceBudget::new(4096, 8192, 250).unwrap(),
        );
        let encoded = serde_json::to_vec(&contract).expect("contract should serialize");
        let decoded: EditorCommandExecutionContract =
            serde_json::from_slice(&encoded).expect("contract should deserialize");
        assert_eq!(decoded, contract);

        let invalid = br#"{
            "result_codec":"zircon.editor.command-result.v1",
            "resource_budget":{"max_input_bytes":0,"max_output_bytes":0,"max_execution_time_ms":0}
        }"#;
        assert!(serde_json::from_slice::<EditorCommandExecutionContract>(invalid).is_err());
    }
}
