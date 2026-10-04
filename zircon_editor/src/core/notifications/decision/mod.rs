//! 挂载需要操作者选择的决策、发布票据和不可变回执契约；生产者的后续变更通过消费回执完成。
mod center;
mod error;
mod id;
mod model;
mod receipt;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

#[cfg(test)]
use super::identity::{NotificationId, NotificationSource};

pub use center::{DecisionCenterConfig, DecisionNotificationCenter};
pub use error::DecisionNotificationError;
pub use id::{
    DecisionCenterInstanceId, DecisionOptionId, DecisionReceiptCursor, DecisionReceiptSequence,
    DecisionTicket, MAX_DECISION_OPTION_ID_BYTES,
};
pub use model::{
    DecisionNotification, DecisionNotificationSnapshot, DecisionOption,
    MAX_DECISION_DISPLAY_SUBJECT_BYTES, MAX_DECISION_OPTIONS, MAX_LOCALIZATION_KEY_BYTES,
};
pub use receipt::{DecisionReceipt, DecisionReceiptBatch, DecisionResolveReport};
