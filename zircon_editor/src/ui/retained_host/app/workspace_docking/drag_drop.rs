use super::super::*;
use crate::ui::retained_host::tab_drag::{
    resolve_host_tab_drop_route_with_workbench_layout_frames, HostDragTargetGroup,
    ResolvedHostTabDropRoute, ResolvedHostTabDropTarget,
};
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};
use crate::ui::retained_host::UiHostContext;

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

        let layout = self.runtime.current_layout();
        let chrome = self.build_chrome();
        record_current_ui_perf_counter(UiPerfCounter::WorkbenchModelBuildCount, 1.0);
        let model = WorkbenchViewModel::build(&chrome);
        let pointer_route = self.shell_pointer_bridge.drag_route_at(UiPoint::new(x, y));
        let resolved = if target_group.is_empty() && pointer_route.is_none() {
            Some(detached_window_drop_route(
                &tab_id,
                drag_state.drag_source_group.as_str(),
            ))
        } else {
            resolve_host_tab_drop_route_with_workbench_layout_frames(
                &layout,
                &model,
                &self.chrome_metrics,
                &tab_id,
                pointer_route,
                target_group.as_str(),
                x,
                y,
                self.workbench_window_bridge.layout_frames(),
            )
        };
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

fn detached_window_drop_route(instance_id: &str, source_group: &str) -> ResolvedHostTabDropRoute {
    let drawer_source = matches!(source_group, "left" | "right" | "bottom");
    ResolvedHostTabDropRoute {
        target_group: HostDragTargetGroup::Document,
        target_label: if drawer_source {
            "detached drawer window"
        } else {
            "detached window"
        },
        target: ResolvedHostTabDropTarget::DetachToWindow {
            new_window: detached_window_id(instance_id, drawer_source),
        },
    }
}

fn detached_window_id(instance_id: &str, drawer_source: bool) -> MainPageId {
    let prefix = if drawer_source {
        "drawer-window"
    } else {
        "window"
    };
    let suffix = instance_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_' {
                ch
            } else {
                ':'
            }
        })
        .collect::<String>();
    MainPageId::new(format!("{prefix}:{suffix}"))
}
