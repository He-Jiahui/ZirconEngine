use serde::{Deserialize, Serialize};

pub const UI_NUMBER_INPUT_RECEIPT_VERSION_V1: u16 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiNumberFormatIdentityV1 {
    #[default]
    InvariantAscii,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiNumberInputParseStatus {
    #[default]
    Empty,
    Intermediate,
    Valid,
    OutOfRange,
    TooLong,
    NonFinite,
    InvalidCharacter,
    InvalidSyntax,
    InvalidPolicy,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiNumberInputCommitMethod {
    #[default]
    Edit,
    Enter,
    KeyboardStep,
    Accessibility,
    FocusLoss,
    Escape,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiNumberInputCommitStatus {
    #[default]
    NotRequested,
    Applied,
    Unchanged,
    Clamped,
    Snapped,
    Conflict,
    Rejected,
    Cancelled,
}

/// Content-free V1 receipt for one invariant numeric edit or commit decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiNumberInputReceiptV1 {
    pub version: u16,
    pub format: UiNumberFormatIdentityV1,
    pub parse_status: UiNumberInputParseStatus,
    pub commit_method: UiNumberInputCommitMethod,
    pub commit_status: UiNumberInputCommitStatus,
}

impl Default for UiNumberInputReceiptV1 {
    fn default() -> Self {
        Self {
            version: UI_NUMBER_INPUT_RECEIPT_VERSION_V1,
            format: UiNumberFormatIdentityV1::InvariantAscii,
            parse_status: UiNumberInputParseStatus::Empty,
            commit_method: UiNumberInputCommitMethod::Edit,
            commit_status: UiNumberInputCommitStatus::NotRequested,
        }
    }
}

impl UiNumberInputReceiptV1 {
    pub const fn validate(self) -> bool {
        self.version == UI_NUMBER_INPUT_RECEIPT_VERSION_V1
    }
}

#[cfg(test)]
mod tests {
    use super::{
        UiNumberInputCommitMethod, UiNumberInputCommitStatus, UiNumberInputParseStatus,
        UiNumberInputReceiptV1, UI_NUMBER_INPUT_RECEIPT_VERSION_V1,
    };

    #[test]
    fn number_input_receipt_roundtrips_without_source_text() {
        let receipt = UiNumberInputReceiptV1 {
            parse_status: UiNumberInputParseStatus::OutOfRange,
            commit_method: UiNumberInputCommitMethod::Enter,
            commit_status: UiNumberInputCommitStatus::Clamped,
            ..UiNumberInputReceiptV1::default()
        };

        let json = serde_json::to_string(&receipt).expect("receipt serializes");
        let roundtrip: UiNumberInputReceiptV1 =
            serde_json::from_str(&json).expect("receipt deserializes");

        assert_eq!(roundtrip, receipt);
        assert!(roundtrip.validate());
        assert_eq!(roundtrip.version, UI_NUMBER_INPUT_RECEIPT_VERSION_V1);
        assert!(!json.contains("123456"));
    }

    #[test]
    fn keyboard_step_receipt_has_a_stable_snake_case_wire_value() {
        let receipt = UiNumberInputReceiptV1 {
            parse_status: UiNumberInputParseStatus::Valid,
            commit_method: UiNumberInputCommitMethod::KeyboardStep,
            commit_status: UiNumberInputCommitStatus::Applied,
            ..UiNumberInputReceiptV1::default()
        };

        let json = serde_json::to_string(&receipt).expect("receipt serializes");

        assert!(json.contains("\"commit_method\":\"keyboard_step\""));
    }
}
