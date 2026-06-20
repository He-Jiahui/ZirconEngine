use super::super::*;
use crate::ui::retained_host::UiHostContext;

mod route;

impl RetainedEditorHost {
    pub(super) fn sync_drag_target_group(&mut self, x: f32, y: f32) {
        let value = self
            .shell_pointer_bridge
            .drag_route_at(UiPoint::new(x, y))
            .and_then(|route| host_shell_pointer_route_group_key(&route))
            .unwrap_or_default();
        let host_shell = self.ui.global::<UiHostContext>();
        let mut drag_state = host_shell.get_drag_state();
        drag_state.active_drag_target_group = value.into();
        host_shell.set_drag_state(drag_state);
    }

    pub(super) fn dispatch_drag_drop_from_pointer(&mut self, x: f32, y: f32) {
        self.sync_drag_target_group(x, y);

        let host_shell = self.ui.global::<UiHostContext>();
        let drag_state = host_shell.get_drag_state();
        let tab_id = drag_state.drag_tab_id.to_string();
        let target_group = drag_state.active_drag_target_group.to_string();
        if tab_id.is_empty() {
            return;
        }

        let resolved = self.resolve_drag_drop_route_from_pointer(
            &tab_id,
            drag_state.drag_source_group.as_str(),
            target_group.as_str(),
            x,
            y,
        );
        let Some(resolved) = resolved else {
            self.set_status_line(format!("Unsupported drop target {target_group}"));
            return;
        };

        match callback_dispatch::dispatch_tab_drop(&self.runtime, &tab_id, &resolved) {
            Ok(effects) => {
                self.apply_dispatch_effects(effects);
                self.set_status_line(format!("Moved {} to {}", tab_id, resolved.target_label));
            }
            Err(error) => self.set_status_line(error),
        }
    }
}
