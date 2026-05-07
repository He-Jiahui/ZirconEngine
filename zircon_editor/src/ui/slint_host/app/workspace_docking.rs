use super::*;
use crate::ui::slint_host::root_shell_projection::{
    resolve_root_bottom_region_frame, resolve_root_left_region_frame,
    resolve_root_right_region_frame,
};
use crate::ui::slint_host::tab_drag::{
    resolve_host_tab_drop_route_with_root_frames, HostDragTargetGroup, ResolvedHostTabDropRoute,
    ResolvedHostTabDropTarget,
};
use crate::ui::slint_host::UiHostContext;
use crate::ui::workbench::autolayout::ShellFrame;

const HOST_POINTER_DOWN: i32 = 0;
const HOST_POINTER_MOVE: i32 = 1;
const HOST_POINTER_UP: i32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostPointerFactKind {
    Down,
    Move,
    Up,
}

impl SlintEditorHost {
    pub(super) fn host_drag_pointer_event(&mut self, kind: i32, x: f32, y: f32) {
        self.recompute_if_dirty();
        let kind = match map_host_pointer_kind(kind, "drag") {
            Ok(kind) => kind,
            Err(error) => {
                self.set_status_line(error);
                return;
            }
        };

        match kind {
            HostPointerFactKind::Down | HostPointerFactKind::Move => {
                self.sync_drag_target_group(x, y);
            }
            HostPointerFactKind::Up => self.dispatch_drag_drop_from_pointer(x, y),
        }
    }

    pub(super) fn host_resize_pointer_event(&mut self, kind: i32, x: f32, y: f32) {
        self.recompute_if_dirty();
        let kind = match map_host_pointer_kind(kind, "resize") {
            Ok(kind) => kind,
            Err(error) => {
                self.set_status_line(error);
                return;
            }
        };

        match kind {
            HostPointerFactKind::Down => self.begin_drawer_resize_capture(x, y),
            HostPointerFactKind::Move => self.update_drawer_resize_capture(x, y),
            HostPointerFactKind::Up => self.finish_drawer_resize_capture(x, y),
        }
    }

    fn sync_drag_target_group(&mut self, x: f32, y: f32) {
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

    fn dispatch_drag_drop_from_pointer(&mut self, x: f32, y: f32) {
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
        let model = WorkbenchViewModel::build(&chrome);
        let pointer_route = self.shell_pointer_bridge.drag_route_at(UiPoint::new(x, y));
        let root_shell_frames = self.template_bridge.root_shell_frames();
        let resolved = if target_group.is_empty() && pointer_route.is_none() {
            Some(detached_window_drop_route(
                &tab_id,
                drag_state.drag_source_group.as_str(),
            ))
        } else {
            resolve_host_tab_drop_route_with_root_frames(
                &layout,
                &model,
                &self.chrome_metrics,
                &tab_id,
                pointer_route,
                target_group.as_str(),
                x,
                y,
                Some(&root_shell_frames),
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

    fn begin_drawer_resize_capture(&mut self, x: f32, y: f32) {
        let Some(region) = self
            .shell_pointer_bridge
            .begin_resize(UiPoint::new(x, y))
            .and_then(|route| match route {
                HostShellPointerRoute::Resize(group) => Some(group.region()),
                HostShellPointerRoute::DragTarget(_)
                | HostShellPointerRoute::DocumentEdge(_)
                | HostShellPointerRoute::FloatingWindow(_)
                | HostShellPointerRoute::FloatingWindowEdge { .. } => None,
            })
        else {
            return;
        };
        let root_shell_frames = self.template_bridge.root_shell_frames();
        let frame = match region {
            ShellRegionId::Left => resolve_root_left_region_frame(Some(&root_shell_frames)),
            ShellRegionId::Right => resolve_root_right_region_frame(Some(&root_shell_frames)),
            ShellRegionId::Bottom => resolve_root_bottom_region_frame(Some(&root_shell_frames)),
            ShellRegionId::Document => ShellFrame::default(),
        };
        let base_preferred = match region {
            ShellRegionId::Bottom => frame.height,
            ShellRegionId::Left | ShellRegionId::Right | ShellRegionId::Document => frame.width,
        };
        if base_preferred <= 0.0 {
            return;
        }

        self.active_drawer_resize = Some(ActiveDrawerResize {
            region,
            start_x: x,
            start_y: y,
            base_preferred,
        });
        self.update_drawer_resize_capture(x, y);
    }

    fn update_drawer_resize_capture(&mut self, x: f32, y: f32) {
        let Some(active) = self.active_drawer_resize else {
            return;
        };
        let _ = self.shell_pointer_bridge.update_resize(UiPoint::new(x, y));
        let preferred = match active.region {
            ShellRegionId::Left => active.base_preferred + (x - active.start_x),
            ShellRegionId::Right => active.base_preferred - (x - active.start_x),
            ShellRegionId::Bottom => active.base_preferred - (y - active.start_y),
            ShellRegionId::Document => active.base_preferred,
        }
        .max(0.0);

        self.transient_region_preferred
            .insert(active.region, preferred);
        self.mark_layout_dirty();
        self.recompute_if_dirty();
    }

    fn finish_drawer_resize_capture(&mut self, x: f32, y: f32) {
        self.update_drawer_resize_capture(x, y);
        let _ = self.shell_pointer_bridge.finish_resize(UiPoint::new(x, y));

        let Some(active) = self.active_drawer_resize.take() else {
            return;
        };
        let preferred = self
            .transient_region_preferred
            .get(&active.region)
            .copied()
            .unwrap_or(active.base_preferred);
        self.transient_region_preferred.remove(&active.region);

        match dispatch_resize_to_group(
            &self.runtime,
            shell_region_group_key(active.region),
            preferred,
        ) {
            Ok(effects) => {
                self.apply_dispatch_effects(effects);
                if !self.layout_dirty {
                    self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
                }
            }
            Err(error) => self.set_status_line(error),
        }

        self.recompute_if_dirty();
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

fn map_host_pointer_kind(kind: i32, channel: &str) -> Result<HostPointerFactKind, String> {
    match kind {
        HOST_POINTER_DOWN => Ok(HostPointerFactKind::Down),
        HOST_POINTER_MOVE => Ok(HostPointerFactKind::Move),
        HOST_POINTER_UP => Ok(HostPointerFactKind::Up),
        _ => Err(format!("unknown host {channel} pointer kind {kind}")),
    }
}
