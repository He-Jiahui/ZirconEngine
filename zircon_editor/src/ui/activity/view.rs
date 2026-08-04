use std::time::Duration;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::event_ui::UiNodePath;

use crate::core::i18n::EditorI18nService;
use crate::core::notifications::{
    LocalizedToastNotification, ToastNotificationSnapshot, ToastSeverity, present_toast,
};

use super::slot::ActivityDrawerSlotPreference;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ActivityToastView {
    id: String,
    title: String,
    message: String,
    severity: ToastSeverity,
    expires_at: Duration,
    remaining_lifetime: Duration,
}

impl ActivityToastView {
    pub(crate) fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        message: impl Into<String>,
        severity: ToastSeverity,
        expires_at: Duration,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            message: message.into(),
            severity,
            expires_at,
            remaining_lifetime: expires_at,
        }
    }

    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn title(&self) -> &str {
        &self.title
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }

    pub(crate) const fn severity(&self) -> ToastSeverity {
        self.severity
    }

    pub(crate) const fn expires_at(&self) -> Duration {
        self.expires_at
    }

    pub(crate) const fn remaining_lifetime(&self) -> Duration {
        self.remaining_lifetime
    }
}

pub(crate) fn activity_toast_views(
    snapshots: &[ToastNotificationSnapshot],
    i18n: &EditorI18nService,
    now: Duration,
) -> Vec<ActivityToastView> {
    snapshots
        .iter()
        .map(|snapshot| activity_toast_view(&present_toast(snapshot, i18n), now))
        .collect()
}

fn activity_toast_view(
    notification: &LocalizedToastNotification,
    now: Duration,
) -> ActivityToastView {
    let mut view = ActivityToastView::new(
        notification.id().as_str(),
        notification.title(),
        notification.message(),
        notification.severity(),
        notification.expires_at(),
    );
    view.remaining_lifetime = notification.expires_at().saturating_sub(now);
    view
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityViewDescriptor {
    pub view_id: String,
    pub title: String,
    pub icon_key: String,
    pub multi_instance: bool,
    pub supports_document_host: bool,
    pub supports_floating_window: bool,
    pub default_drawer: Option<ActivityDrawerSlotPreference>,
    pub reflection_root: UiNodePath,
}

impl ActivityViewDescriptor {
    pub fn new(
        view_id: impl Into<String>,
        title: impl Into<String>,
        icon_key: impl Into<String>,
    ) -> Self {
        let view_id = view_id.into();
        Self {
            reflection_root: UiNodePath::new(format!("editor/views/{view_id}")),
            view_id,
            title: title.into(),
            icon_key: icon_key.into(),
            multi_instance: false,
            supports_document_host: true,
            supports_floating_window: true,
            default_drawer: None,
        }
    }

    pub fn with_multi_instance(mut self, multi_instance: bool) -> Self {
        self.multi_instance = multi_instance;
        self
    }

    pub fn with_supports_document_host(mut self, supports: bool) -> Self {
        self.supports_document_host = supports;
        self
    }

    pub fn with_supports_floating_window(mut self, supports: bool) -> Self {
        self.supports_floating_window = supports;
        self
    }

    pub fn with_default_drawer(mut self, slot: ActivityDrawerSlotPreference) -> Self {
        self.default_drawer = Some(slot);
        self
    }

    pub fn with_reflection_root(mut self, root: UiNodePath) -> Self {
        self.reflection_root = root;
        self
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::core::i18n::{EditorI18nService, EditorLocale};
    use crate::core::notifications::{
        NotificationId, NotificationSource, ToastCenterConfig, ToastNotification,
        ToastNotificationCenter, ToastSeverity,
    };

    use super::activity_toast_views;

    #[test]
    fn activity_toast_views_localize_immutable_core_snapshots() {
        let center = ToastNotificationCenter::new(ToastCenterConfig::default());
        center
            .publish_at(
                ToastNotification::new(
                    NotificationId::parse("editor.activity.save").unwrap(),
                    NotificationSource::builtin("editor.activity").unwrap(),
                    ToastSeverity::Success,
                    "editor.notification.project_saved.title",
                    "editor.notification.project_saved.message",
                    Duration::from_secs(3),
                )
                .unwrap(),
                Duration::ZERO,
            )
            .unwrap();
        let i18n = EditorI18nService::default();
        i18n.set_active_locale(EditorLocale::parse("zh-CN").unwrap())
            .unwrap();

        let views =
            activity_toast_views(&center.snapshot_at(Duration::ZERO), &i18n, Duration::ZERO);

        assert_eq!(views.len(), 1);
        assert_eq!(views[0].id(), "editor.activity.save");
        assert_eq!(views[0].title(), "项目已保存");
        assert_eq!(views[0].message(), "项目状态已写入磁盘。");
        assert_eq!(views[0].severity(), ToastSeverity::Success);
        assert_eq!(views[0].expires_at(), Duration::from_secs(3));
        assert_eq!(views[0].remaining_lifetime(), Duration::from_secs(3));
    }
}
