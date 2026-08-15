use crate::core::notifications::{
    NotificationId, NotificationSource, ToastNotification, ToastSeverity,
};
use crate::ui::retained_host::app::{HostInvalidationMask, RetainedEditorHost};
use crate::ui::retained_host::event_bridge::UiHostEventEffects;
use std::time::Duration;

mod side_effects;
mod status;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn apply_dispatch_effects(
        &mut self,
        effects: UiHostEventEffects,
    ) {
        if let Some(name) = effects.active_layout_preset_name.clone() {
            self.active_layout_preset = Some(name);
        }
        if effects.reset_active_layout_preset {
            self.active_layout_preset = None;
        }
        let dirty_domains = effects.dirty_domains();
        if let Some(scope) = effects.shell_content_scope() {
            self.invalidate_host_for_shell_content(scope, dirty_domains);
        } else {
            self.invalidate_host(dirty_domains);
        }
        self.apply_dispatch_side_effects(&effects);
    }

    pub(super) fn apply_viewport_resize_effects_in_active_recompute(
        &mut self,
        result: Result<UiHostEventEffects, String>,
    ) -> bool {
        match result {
            Ok(effects) => {
                if !effects.is_viewport_resize_recompute_compatible() {
                    self.apply_dispatch_effects(effects);
                    return false;
                }
                // The active recompute directly patches the two viewport-size
                // projections, so only render work remains for the submission
                // phase later in the current tick.
                if let Some(name) = effects.active_layout_preset_name.clone() {
                    self.active_layout_preset = Some(name);
                }
                if effects.reset_active_layout_preset {
                    self.active_layout_preset = None;
                }
                let mut dirty_domains = effects.dirty_domains();
                dirty_domains.remove(HostInvalidationMask::PRESENTATION_DATA);
                self.invalidate_host(dirty_domains);
                self.apply_dispatch_side_effects(&effects);
                true
            }
            Err(error) => {
                self.set_status_line(error);
                false
            }
        }
    }

    pub(in crate::ui::retained_host::app) fn apply_dispatch_result(
        &mut self,
        result: Result<UiHostEventEffects, String>,
    ) {
        match result {
            Ok(effects) => self.apply_dispatch_effects(effects),
            Err(error) => {
                if let Some(notification) = dispatch_error_toast(&error) {
                    self.publish_activity_toasts(std::slice::from_ref(&notification));
                }
                self.set_status_line(error);
            }
        }
    }
}

fn dispatch_error_toast(error: &str) -> Option<ToastNotification> {
    let id = NotificationId::parse("editor.dispatch.failed").ok()?;
    let source = NotificationSource::builtin("editor.retained_host").ok()?;
    let message =
        ToastNotification::bounded_message(error, "The editor command could not complete.");
    ToastNotification::new(
        id,
        source,
        ToastSeverity::Error,
        "editor.notification.command_failed.title",
        message,
        Duration::from_secs(7),
    )
    .ok()
}
