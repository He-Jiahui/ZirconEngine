use super::super::*;
use crate::ui::retained_host::route_intent::EditorRouteIntentHandle;
use crate::ui::retained_host::UiHostContext;
use crate::ui::workbench::autolayout::ShellFrame;

mod route;

impl RetainedEditorHost {
    pub(super) fn sync_drag_target_group(
        &mut self,
        source_window_id: Option<&MainPageId>,
        x: f32,
        y: f32,
    ) -> Option<EditorRouteIntentHandle> {
        let (source_ui, point) = self.drag_pointer_source(source_window_id, x, y)?;
        self.sync_drag_target_group_at(&source_ui, point)
    }

    fn sync_drag_target_group_at(
        &mut self,
        source_ui: &UiHostWindow,
        point: UiPoint,
    ) -> Option<EditorRouteIntentHandle> {
        let handle = self.shell_pointer_bridge.drag_route_handle_at(point);
        let host_shell = source_ui.global::<UiHostContext>();
        let unchanged = host_shell.drag_target_group_matches(|group_key| {
            match handle.and_then(|handle| self.shell_pointer_bridge.shell_route_for_handle(handle))
            {
                Some(route) => host_shell_pointer_route_matches_group_key(route, group_key),
                None => group_key.is_empty(),
            }
        });
        if unchanged {
            return handle;
        }
        let value = handle
            .and_then(|handle| self.shell_pointer_bridge.shell_route_for_handle(handle))
            .and_then(host_shell_pointer_route_group_key)
            .unwrap_or_default();
        host_shell.set_drag_target_group(value);
        handle
    }

    pub(super) fn dispatch_drag_drop_from_pointer(
        &mut self,
        source_window_id: Option<&MainPageId>,
        x: f32,
        y: f32,
    ) {
        let Some((source_ui, point)) = self.drag_pointer_source(source_window_id, x, y) else {
            return;
        };
        let pointer_route = self
            .sync_drag_target_group_at(&source_ui, point)
            .and_then(|handle| self.shell_pointer_bridge.shell_route_for_handle(handle))
            .cloned();

        let host_shell = source_ui.global::<UiHostContext>();
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
            point.x,
            point.y,
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

    fn drag_pointer_source(
        &self,
        source_window_id: Option<&MainPageId>,
        x: f32,
        y: f32,
    ) -> Option<(UiHostWindow, UiPoint)> {
        let Some(source_window_id) = source_window_id else {
            return Some((self.ui.clone(), UiPoint::new(x, y)));
        };

        let source_ui = self.native_window_presenters.window(source_window_id)?;
        if !source_ui.window().is_visible() {
            return None;
        }
        let frames = self
            .floating_window_projection_bundle
            .frames(source_window_id)?;
        if !frames.native_host_present {
            return None;
        }

        Some((
            source_ui,
            child_pointer_to_workbench_point(frames.outer_frame, UiPoint::new(x, y)),
        ))
    }
}

fn child_pointer_to_workbench_point(frame: ShellFrame, point: UiPoint) -> UiPoint {
    UiPoint::new(frame.x + point.x, frame.y + point.y)
}

#[cfg(test)]
#[path = "tests/drag_drop_performance_tests.rs"]
mod performance_tests;
