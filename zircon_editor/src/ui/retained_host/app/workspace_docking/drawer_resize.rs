use super::super::*;
use crate::ui::workbench::autolayout::ShellFrame;

impl RetainedEditorHost {
    pub(super) fn begin_drawer_resize_capture(&mut self, x: f32, y: f32) {
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
        let workbench_layout_frames = self.workbench_window_bridge.layout_frames();
        let Some(frame) = workbench_layout_frames
            .drawer_shell_frame(region)
            .filter(ui_frame_is_visible)
            .map(shell_frame)
        else {
            return;
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

    pub(super) fn update_drawer_resize_capture(&mut self, x: f32, y: f32) {
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
        self.use_committed_pointer_layout();
    }

    pub(super) fn finish_drawer_resize_capture(&mut self, x: f32, y: f32) {
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

        self.use_committed_pointer_layout();
    }
}

fn shell_frame(frame: UiFrame) -> ShellFrame {
    ShellFrame::new(frame.x, frame.y, frame.width, frame.height)
}

fn ui_frame_is_visible(frame: &UiFrame) -> bool {
    frame.width > f32::EPSILON && frame.height > f32::EPSILON
}
