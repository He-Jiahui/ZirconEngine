use std::sync::Arc;
use std::time::Duration;

use crate::core::i18n::{EditorI18nService, EditorLocale};
use crate::core::jobs::EditorJobProgressSnapshot;

use super::{
    DecisionNotificationSnapshot, DecisionOptionId, DecisionReceipt, DecisionTicket,
    NotificationId, NotificationSource, ProgressNotificationSnapshot, ToastNotificationSnapshot,
    ToastSeverity,
};

/// Localized display data for a decision option. The option id remains the sole action identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalizedDecisionOption {
    id: DecisionOptionId,
    label: Arc<str>,
}

impl LocalizedDecisionOption {
    pub fn id(&self) -> &DecisionOptionId {
        &self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

/// Read-only display projection for a decision. It does not own or resolve the decision state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalizedDecisionNotification {
    locale: EditorLocale,
    ticket: DecisionTicket,
    id: NotificationId,
    source: NotificationSource,
    title: Arc<str>,
    message: Arc<str>,
    options: Vec<LocalizedDecisionOption>,
    default_option: Option<DecisionOptionId>,
    cancel_option: Option<DecisionOptionId>,
    resolved: Option<DecisionReceipt>,
}

impl LocalizedDecisionNotification {
    pub fn locale(&self) -> &EditorLocale {
        &self.locale
    }

    pub fn ticket(&self) -> &DecisionTicket {
        &self.ticket
    }

    pub fn id(&self) -> &NotificationId {
        &self.id
    }

    pub fn source(&self) -> &NotificationSource {
        &self.source
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn options(&self) -> &[LocalizedDecisionOption] {
        &self.options
    }

    pub fn default_option(&self) -> Option<&DecisionOptionId> {
        self.default_option.as_ref()
    }

    pub fn cancel_option(&self) -> Option<&DecisionOptionId> {
        self.cancel_option.as_ref()
    }

    pub fn resolved(&self) -> Option<&DecisionReceipt> {
        self.resolved.as_ref()
    }
}

/// Read-only display projection for an expiring toast.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalizedToastNotification {
    locale: EditorLocale,
    id: NotificationId,
    source: NotificationSource,
    severity: ToastSeverity,
    title: Arc<str>,
    message: Arc<str>,
    expires_at: Duration,
}

impl LocalizedToastNotification {
    pub fn locale(&self) -> &EditorLocale {
        &self.locale
    }

    pub fn id(&self) -> &NotificationId {
        &self.id
    }

    pub fn source(&self) -> &NotificationSource {
        &self.source
    }

    pub const fn severity(&self) -> ToastSeverity {
        self.severity
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub const fn expires_at(&self) -> Duration {
        self.expires_at
    }
}

/// Read-only display projection for progress associated with one editor job.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalizedProgressNotification {
    locale: EditorLocale,
    id: NotificationId,
    source: NotificationSource,
    title: Arc<str>,
    job: EditorJobProgressSnapshot,
}

impl LocalizedProgressNotification {
    pub fn locale(&self) -> &EditorLocale {
        &self.locale
    }

    pub fn id(&self) -> &NotificationId {
        &self.id
    }

    pub fn source(&self) -> &NotificationSource {
        &self.source
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn job(&self) -> &EditorJobProgressSnapshot {
        &self.job
    }
}

/// Resolves display text at the edge while preserving the decision ticket and option ids.
pub fn present_decision(
    snapshot: &DecisionNotificationSnapshot,
    i18n: &EditorI18nService,
) -> LocalizedDecisionNotification {
    let locale = captured_locale(i18n);
    let notification = snapshot.notification();
    LocalizedDecisionNotification {
        locale: locale.clone(),
        ticket: snapshot.ticket().clone(),
        id: notification.id().clone(),
        source: notification.source().clone(),
        title: i18n.translate_for_locale(&locale, notification.title_key()),
        message: format_decision_message(i18n, &locale, notification),
        options: notification
            .options()
            .iter()
            .map(|option| LocalizedDecisionOption {
                id: option.id().clone(),
                label: i18n.translate_for_locale(&locale, option.label_key()),
            })
            .collect(),
        default_option: notification.default_option().cloned(),
        cancel_option: notification.cancel_option().cloned(),
        resolved: snapshot.resolved().cloned(),
    }
}

fn format_decision_message(
    i18n: &EditorI18nService,
    locale: &EditorLocale,
    notification: &super::DecisionNotification,
) -> Arc<str> {
    let mut message = i18n
        .translate_for_locale(locale, notification.message_key())
        .to_string();
    for (name, value) in notification.message_arguments() {
        message = message.replace(&format!("{{{name}}}"), &value.to_string());
    }
    Arc::from(message)
}

/// Resolves toast text without changing expiry, severity, or notification identity.
pub fn present_toast(
    snapshot: &ToastNotificationSnapshot,
    i18n: &EditorI18nService,
) -> LocalizedToastNotification {
    let locale = captured_locale(i18n);
    let notification = snapshot.notification();
    LocalizedToastNotification {
        locale: locale.clone(),
        id: notification.id().clone(),
        source: notification.source().clone(),
        severity: notification.severity(),
        title: i18n.translate_for_locale(&locale, notification.title_key()),
        message: i18n.translate_for_locale(&locale, notification.message_key()),
        expires_at: snapshot.expires_at(),
    }
}

/// Resolves progress text without changing its bound job snapshot.
pub fn present_progress(
    snapshot: &ProgressNotificationSnapshot,
    i18n: &EditorI18nService,
) -> LocalizedProgressNotification {
    let locale = captured_locale(i18n);
    let notification = snapshot.notification();
    LocalizedProgressNotification {
        locale: locale.clone(),
        id: notification.id().clone(),
        source: notification.source().clone(),
        title: i18n.translate_for_locale(&locale, notification.title_key()),
        job: snapshot.job().clone(),
    }
}

fn captured_locale(i18n: &EditorI18nService) -> EditorLocale {
    let locale = i18n.active_locale();
    #[cfg(test)]
    i18n.run_after_locale_capture_hook();
    locale
}
