use super::super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::workspace_docking) fn update_drawer_resize_capture(
        &mut self,
        x: f32,
        y: f32,
    ) {
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

        let previous_preferred = self
            .transient_region_preferred
            .get(&active.region)
            .copied()
            .unwrap_or(active.base_preferred);
        if previous_preferred == preferred {
            return;
        }
        self.transient_region_preferred
            .insert(active.region, preferred);
        self.mark_layout_dirty();
        self.use_committed_pointer_layout();
    }

    pub(in crate::ui::retained_host::app::workspace_docking) fn finish_drawer_resize_capture(
        &mut self,
        x: f32,
        y: f32,
    ) {
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
