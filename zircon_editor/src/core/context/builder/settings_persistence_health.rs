use std::sync::Arc;
use std::time::Duration;

use zircon_runtime::core::runtime::tasks::BoundedKeyedIoTerminal;

use crate::core::notifications::{
    EditorNotificationService, NotificationId, NotificationSource, ToastNotification, ToastSeverity,
};
use crate::core::settings::{
    SettingsPersistenceDocumentHealth, SettingsPersistenceHealthSnapshot,
    SettingsPersistenceHealthStatus, SettingsPersistenceHealthSubscriber, SettingsScope,
};

const SETTINGS_PERSISTENCE_FAILURE_TOAST_LIFETIME: Duration = Duration::from_secs(12);
const SETTINGS_PERSISTENCE_NOTIFICATION_SOURCE: &str = "editor17";

pub(super) struct EditorSettingsPersistenceHealthSubscriber {
    notifications: Arc<EditorNotificationService>,
}

impl EditorSettingsPersistenceHealthSubscriber {
    pub(super) fn new(notifications: Arc<EditorNotificationService>) -> Self {
        Self { notifications }
    }
}

impl SettingsPersistenceHealthSubscriber for EditorSettingsPersistenceHealthSubscriber {
    fn persistence_health_changed(&self, snapshot: &SettingsPersistenceHealthSnapshot) {
        for document in [snapshot.user(), snapshot.project()] {
            let Some(message_key) = failure_message_key(document) else {
                continue;
            };
            let Some(notification) =
                failure_notification(snapshot.generation(), document, message_key)
            else {
                tracing::error!(
                    health_generation = snapshot.generation(),
                    scope = ?document.scope(),
                    "settings persistence health notification identity was invalid"
                );
                continue;
            };
            if let Err(error) = self.notifications.publish_toast(notification) {
                tracing::warn!(
                    health_generation = snapshot.generation(),
                    scope = ?document.scope(),
                    ?error,
                    "settings persistence failure notification was not admitted"
                );
            }
        }
    }
}

fn failure_message_key(document: SettingsPersistenceDocumentHealth) -> Option<&'static str> {
    match document.status() {
        SettingsPersistenceHealthStatus::PendingAdmission(_) => {
            Some("editor.notification.settings_persistence_pending.message")
        }
        SettingsPersistenceHealthStatus::Terminal(BoundedKeyedIoTerminal::Failed(_)) => {
            Some("editor.notification.settings_persistence_failed.message")
        }
        _ => None,
    }
}

fn failure_notification(
    health_generation: u64,
    document: SettingsPersistenceDocumentHealth,
    message_key: &'static str,
) -> Option<ToastNotification> {
    let scope = match document.scope() {
        SettingsScope::User => "user",
        SettingsScope::Project => "project",
        SettingsScope::Session => return None,
    };
    let id = NotificationId::parse(format!(
        "editor.settings_persistence.{scope}.{health_generation}"
    ))
    .ok()?;
    let source = NotificationSource::builtin(SETTINGS_PERSISTENCE_NOTIFICATION_SOURCE).ok()?;
    ToastNotification::new(
        id,
        source,
        ToastSeverity::Error,
        "editor.notification.settings_persistence_failed.title",
        message_key,
        SETTINGS_PERSISTENCE_FAILURE_TOAST_LIFETIME,
    )
    .ok()
}
