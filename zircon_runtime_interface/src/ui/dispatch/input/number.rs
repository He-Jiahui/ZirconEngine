use serde::{Deserialize, Serialize};

pub const UI_NUMBER_INPUT_RECEIPT_VERSION_V1: u16 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// V1 数值编辑回执声明的输入格式身份；InvariantAscii 是当前唯一变体。
pub enum UiNumberFormatIdentityV1 {
    #[default]
    InvariantAscii,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 一次编辑解析阶段的结果分类，不包含用户输入文本或解析出的数值。
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
/// 触发或尝试提交数值编辑的交互来源。
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
/// 数值提交阶段报告的应用、调整、冲突、拒绝或取消结果。
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
/// 不含文本或数值的单次编辑/提交摘要；version 使用 V1 常量，默认值代表空编辑且未请求提交。
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
    /// 仅确认版本号受支持，不验证解析状态与提交状态的组合。
    pub const fn validate(self) -> bool {
        self.version == UI_NUMBER_INPUT_RECEIPT_VERSION_V1
    }
}

#[cfg(test)]
#[path = "tests/number.rs"]
mod tests;
