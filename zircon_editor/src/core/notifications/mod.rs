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
    DecisionReceiptSequence, DecisionResolveReport, DecisionTicket, MAX_DECISION_OPTION_ID_BYTES,
    MAX_DECISION_OPTIONS, MAX_LOCALIZATION_KEY_BYTES,
};
pub use identity::{
    MAX_NOTIFICATION_ID_BYTES, MAX_NOTIFICATION_SOURCE_ID_BYTES, NotificationId,
    NotificationIdentityError, NotificationSource, NotificationSourceKind,
};
pub use presentation::{
    LocalizedDecisionNotification, LocalizedDecisionOption, LocalizedProgressNotification,
    LocalizedToastNotification, present_decision, present_progress, present_toast,
};
pub use progress::{
    ProgressNotification, ProgressNotificationCenter, ProgressNotificationError,
    ProgressNotificationSnapshot,
};
pub use service::EditorNotificationService;
pub use toast::{
    ToastCenterConfig, ToastNotification, ToastNotificationCenter, ToastNotificationError,
    ToastNotificationSnapshot, ToastSeverity,
};
