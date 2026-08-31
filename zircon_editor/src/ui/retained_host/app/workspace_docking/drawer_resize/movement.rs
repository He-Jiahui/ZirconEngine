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
        self.apply_drawer_resize_pointer_position(active, x, y);
    }

    fn apply_drawer_resize_pointer_position(&mut self, active: ActiveDrawerResize, x: f32, y: f32) {
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
        self.invalidate_host(HostInvalidationMask::WINDOW_METRICS);
        self.use_committed_pointer_layout();
    }

    pub(in crate::ui::retained_host::app::workspace_docking) fn finish_drawer_resize_capture(
        &mut self,
        x: f32,
        y: f32,
    ) {
        let _ = self.shell_pointer_bridge.finish_resize(UiPoint::new(x, y));

        let Some(active) = self.active_drawer_resize.take() else {
            return;
        };
        self.apply_drawer_resize_pointer_position(active, x, y);
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

#[cfg(test)]
mod tests {
    #[test]
    fn transient_drawer_resize_reuses_the_committed_shell_metrics_stage() {
        let source = include_str!("movement.rs");
        let update = source
            .split("fn update_drawer_resize_capture")
            .nth(1)
            .and_then(|tail| tail.split("fn finish_drawer_resize_capture").next())
            .expect("drawer resize movement implementation");

        assert!(update.contains("HostInvalidationMask::WINDOW_METRICS"));
        assert!(!update.contains("mark_layout_dirty"));
        assert!(update.contains("if previous_preferred == preferred"));
        assert!(update.contains("return;"));
    }
}
