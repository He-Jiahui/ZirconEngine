use super::super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::workspace_docking) fn update_drawer_resize_capture(
        &mut self,
        source_window_id: Option<&MainPageId>,
        x: f32,
        y: f32,
    ) {
        let Some(active) = self.active_drawer_resize.as_ref() else {
            return;
        };
        if active.source_window.as_ref() != source_window_id {
            return;
        }
        let (region, start_x, start_y, base_preferred) = (
            active.region,
            active.start_x,
            active.start_y,
            active.base_preferred,
        );
        self.apply_drawer_resize_pointer_position(region, start_x, start_y, base_preferred, x, y);
    }

    fn apply_drawer_resize_pointer_position(
        &mut self,
        region: ShellRegionId,
        start_x: f32,
        start_y: f32,
        base_preferred: f32,
        x: f32,
        y: f32,
    ) {
        let preferred = match region {
            ShellRegionId::Left => base_preferred + (x - start_x),
            ShellRegionId::Right => base_preferred - (x - start_x),
            ShellRegionId::Bottom => base_preferred - (y - start_y),
            ShellRegionId::Document => base_preferred,
        }
        .max(0.0);

        let previous_preferred = self
            .transient_region_preferred
            .get(&region)
            .copied()
            .unwrap_or(base_preferred);
        if previous_preferred == preferred {
            return;
        }
        self.transient_region_preferred.insert(region, preferred);
        self.invalidate_host(HostInvalidationMask::WINDOW_METRICS);
        self.use_committed_pointer_layout();
    }

    pub(in crate::ui::retained_host::app::workspace_docking) fn finish_drawer_resize_capture(
        &mut self,
        source_window_id: Option<&MainPageId>,
        x: f32,
        y: f32,
    ) {
        let Some(active) = self.active_drawer_resize.as_ref() else {
            return;
        };
        if active.source_window.as_ref() != source_window_id {
            return;
        }
        let (region, start_x, start_y, base_preferred) = (
            active.region,
            active.start_x,
            active.start_y,
            active.base_preferred,
        );
        let _ = self.shell_pointer_bridge.finish_resize(UiPoint::new(x, y));
        self.active_drawer_resize.take();
        self.apply_drawer_resize_pointer_position(region, start_x, start_y, base_preferred, x, y);
        let preferred = self
            .transient_region_preferred
            .get(&region)
            .copied()
            .unwrap_or(base_preferred);
        self.transient_region_preferred.remove(&region);

        match dispatch_resize_to_group(&self.runtime, shell_region_group_key(region), preferred) {
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

    pub(in crate::ui::retained_host::app::workspace_docking) fn cancel_drawer_resize_capture(
        &mut self,
        source_window_id: Option<&MainPageId>,
    ) {
        let Some(active) = self.active_drawer_resize.as_ref() else {
            return;
        };
        if active.source_window.as_ref() != source_window_id {
            return;
        }
        let region = active.region;
        self.shell_pointer_bridge.cancel_resize();
        self.active_drawer_resize.take();
        if self.transient_region_preferred.remove(&region).is_some() {
            self.invalidate_host(HostInvalidationMask::WINDOW_METRICS);
        }
        self.use_committed_pointer_layout();
    }
}

#[cfg(test)]
#[path = "tests/movement.rs"]
mod tests;
