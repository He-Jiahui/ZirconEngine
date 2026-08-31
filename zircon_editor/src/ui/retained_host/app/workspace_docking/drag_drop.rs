use super::super::*;
use crate::ui::retained_host::UiHostContext;

mod route;

impl RetainedEditorHost {
    pub(super) fn sync_drag_target_group(
        &mut self,
        x: f32,
        y: f32,
    ) -> Option<HostShellPointerRoute> {
        let route = self.shell_pointer_bridge.drag_route_at(UiPoint::new(x, y));
        let host_shell = self.ui.global::<UiHostContext>();
        let unchanged = host_shell.drag_target_group_matches(|group_key| match route.as_ref() {
            Some(route) => host_shell_pointer_route_matches_group_key(route, group_key),
            None => group_key.is_empty(),
        });
        if unchanged {
            return route;
        }
        let value = route
            .as_ref()
            .and_then(host_shell_pointer_route_group_key)
            .unwrap_or_default();
        host_shell.set_drag_target_group(value);
        route
    }

    pub(super) fn dispatch_drag_drop_from_pointer(&mut self, x: f32, y: f32) {
        let pointer_route = self.sync_drag_target_group(x, y);

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
            pointer_route,
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

#[cfg(test)]
mod performance_tests {
    #[test]
    fn repeated_drag_target_group_does_not_republish_ui_state() {
        let source = include_str!("drag_drop.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);

        assert!(production.contains("drag_target_group_matches"));
        assert!(production.contains("host_shell.set_drag_target_group(value);"));
        assert!(!production.contains("host_shell.set_drag_state"));
    }
}
