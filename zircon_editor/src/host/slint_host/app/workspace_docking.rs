use super::*;
use crate::host::slint_host::tab_drag::resolve_workbench_tab_drop_route_with_root_frames;

const WORKBENCH_POINTER_DOWN: i32 = 0;
const WORKBENCH_POINTER_MOVE: i32 = 1;
const WORKBENCH_POINTER_UP: i32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorkbenchPointerFactKind {
    Down,
    Move,
    Up,
}

impl SlintEditorHost {
    pub(super) fn workbench_drag_pointer_event(&mut self, kind: i32, x: f32, y: f32) {
        self.recompute_if_dirty();
        let kind = match map_workbench_pointer_kind(kind, "drag") {
            Ok(kind) => kind,
            Err(error) => {
                self.set_status_line(error);
                return;
            }
        };

        match kind {
            WorkbenchPointerFactKind::Down | WorkbenchPointerFactKind::Move => {
                self.sync_drag_target_group(x, y);
            }
            WorkbenchPointerFactKind::Up => self.dispatch_drag_drop_from_pointer(x, y),
        }
    }

    pub(super) fn workbench_resize_pointer_event(&mut self, kind: i32, x: f32, y: f32) {
        self.recompute_if_dirty();
        let kind = match map_workbench_pointer_kind(kind, "resize") {
            Ok(kind) => kind,
            Err(error) => {
                self.set_status_line(error);
                return;
            }
        };

        match kind {
            WorkbenchPointerFactKind::Down => self.begin_drawer_resize_capture(x, y),
            WorkbenchPointerFactKind::Move => self.update_drawer_resize_capture(x, y),
            WorkbenchPointerFactKind::Up => self.finish_drawer_resize_capture(x, y),
        }
    }

    fn sync_drag_target_group(&mut self, x: f32, y: f32) {
        let value = self
            .shell_pointer_bridge
            .drag_route_at(UiPoint::new(x, y))
            .and_then(|route| workbench_shell_pointer_route_group_key(&route))
            .unwrap_or_default();
        self.ui.set_active_drag_target_group(value.into());
    }

    fn dispatch_drag_drop_from_pointer(&mut self, x: f32, y: f32) {
        self.sync_drag_target_group(x, y);

        let tab_id = self.ui.get_drag_tab_id().to_string();
        let target_group = self.ui.get_active_drag_target_group().to_string();
        if tab_id.is_empty() || target_group.is_empty() {
            return;
        }

        let layout = self.runtime.current_layout();
        let chrome = self.build_chrome();
        let model = WorkbenchViewModel::build(&chrome);
        let pointer_route = self.shell_pointer_bridge.drag_route_at(UiPoint::new(x, y));
        let root_shell_frames = self.template_bridge.root_shell_frames();
        let Some(resolved) = self.shell_geometry.as_ref().and_then(|geometry| {
            resolve_workbench_tab_drop_route_with_root_frames(
                &layout,
                &model,
                geometry,
                &self.chrome_metrics,
                &tab_id,
                pointer_route,
                target_group.as_str(),
                x,
                y,
                Some(&root_shell_frames),
            )
        }) else {
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
                WorkbenchShellPointerRoute::Resize(group) => Some(group.region()),
                WorkbenchShellPointerRoute::DragTarget(_)
                | WorkbenchShellPointerRoute::DocumentEdge(_)
                | WorkbenchShellPointerRoute::FloatingWindow(_)
                | WorkbenchShellPointerRoute::FloatingWindowEdge { .. } => None,
            })
        else {
            return;
        };
        let Some(geometry) = self.shell_geometry.as_ref() else {
            return;
        };
        let frame = geometry.region_frame(region);
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
                    self.presentation_dirty = true;
                }
            }
            Err(error) => self.set_status_line(error),
        }

        self.recompute_if_dirty();
    }
}

fn map_workbench_pointer_kind(
    kind: i32,
    channel: &str,
) -> Result<WorkbenchPointerFactKind, String> {
    match kind {
        WORKBENCH_POINTER_DOWN => Ok(WorkbenchPointerFactKind::Down),
        WORKBENCH_POINTER_MOVE => Ok(WorkbenchPointerFactKind::Move),
        WORKBENCH_POINTER_UP => Ok(WorkbenchPointerFactKind::Up),
        _ => Err(format!("unknown workbench {channel} pointer kind {kind}")),
    }
}
