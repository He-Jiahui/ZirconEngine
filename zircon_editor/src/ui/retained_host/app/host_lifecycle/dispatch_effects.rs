use crate::ui::retained_host::app::{HostInvalidationMask, RetainedEditorHost};
use crate::ui::retained_host::event_bridge::UiHostEventEffects;
use crate::ui::retained_host::workbench_notifications::workbench_dispatch_error_notification;

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
        self.invalidate_host(effects.dirty_domains());
        self.apply_dispatch_side_effects(&effects);
    }

    pub(super) fn apply_viewport_resize_effects_in_active_recompute(
        &mut self,
        result: Result<UiHostEventEffects, String>,
    ) {
        match result {
            Ok(effects) => {
                // The active recompute immediately rebuilds chrome/model after
                // viewport resize, so only the render-domain part should carry
                // into the next tick.
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
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(in crate::ui::retained_host::app) fn apply_dispatch_result(
        &mut self,
        result: Result<UiHostEventEffects, String>,
    ) {
        match result {
            Ok(effects) => self.apply_dispatch_effects(effects),
            Err(error) => {
                let notification = workbench_dispatch_error_notification(&error);
                self.publish_workbench_notifications(std::slice::from_ref(&notification));
                self.set_status_line(error);
            }
        }
    }
}
