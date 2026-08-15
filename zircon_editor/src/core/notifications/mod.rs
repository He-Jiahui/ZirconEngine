mod decision;
mod identity;
mod presentation;
mod progress;
mod service;
mod toast;

#[cfg(test)]
mod presentation_tests;

pub use decision::{
    DecisionCenterConfig, DecisionCenterInstanceId, DecisionNotification,
    DecisionNotificationCenter, DecisionNotificationError, DecisionNotificationSnapshot,
    DecisionOption, DecisionOptionId, DecisionReceipt, DecisionReceiptBatch, DecisionReceiptCursor,
    DecisionReceiptSequence, DecisionResolveReport, DecisionTicket, MAX_DECISION_OPTIONS,
    MAX_DECISION_OPTION_ID_BYTES, MAX_LOCALIZATION_KEY_BYTES,
};
pub use identity::{
    NotificationId, NotificationIdentityError, NotificationSource, NotificationSourceKind,
    MAX_NOTIFICATION_ID_BYTES, MAX_NOTIFICATION_SOURCE_ID_BYTES,
};
pub use presentation::{
    present_decision, present_progress, present_toast, LocalizedDecisionNotification,
    LocalizedDecisionOption, LocalizedProgressNotification, LocalizedToastNotification,
};
pub(crate) use progress::AUTOMATIC_PROGRESS_SOURCE_ID;
pub use progress::{
    ProgressNotification, ProgressNotificationCenter, ProgressNotificationError,
    ProgressNotificationSnapshot, MAX_PROGRESS_NOTIFICATIONS,
};
pub use service::EditorNotificationService;
pub use toast::{
    ToastCenterConfig, ToastNotification, ToastNotificationCenter, ToastNotificationError,
    ToastNotificationSnapshot, ToastSeverity,
};
