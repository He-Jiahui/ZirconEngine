mod center;
mod error;
mod id;
mod model;
mod receipt;

#[cfg(test)]
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
