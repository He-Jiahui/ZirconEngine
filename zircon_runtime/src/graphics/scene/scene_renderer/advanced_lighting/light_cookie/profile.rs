#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct LightCookieAtlasProfile {
    rebuild_count: u64,
    input_cookie_count: u64,
    planned_entry_count: u64,
    resolved_draw_count: u64,
    unresolved_entry_count: u64,
    blit_bind_group_create_count: u64,
    full_clear_pixel_count: u64,
}

impl LightCookieAtlasProfile {
    pub(super) fn begin_frame(&mut self) {
        *self = Self::default();
    }

    pub(super) fn record_rebuild(
        &mut self,
        input_cookie_count: usize,
        planned_entry_count: usize,
        resolved_draw_count: usize,
        full_clear_pixel_count: u64,
    ) {
        let input_cookie_count = count_as_u64(input_cookie_count);
        let planned_entry_count = count_as_u64(planned_entry_count);
        let resolved_draw_count = count_as_u64(resolved_draw_count);
        self.rebuild_count = self.rebuild_count.saturating_add(1);
        self.input_cookie_count = self.input_cookie_count.saturating_add(input_cookie_count);
        self.planned_entry_count = self.planned_entry_count.saturating_add(planned_entry_count);
        self.resolved_draw_count = self.resolved_draw_count.saturating_add(resolved_draw_count);
        self.unresolved_entry_count = self
            .unresolved_entry_count
            .saturating_add(planned_entry_count.saturating_sub(resolved_draw_count));
        self.blit_bind_group_create_count = self
            .blit_bind_group_create_count
            .saturating_add(resolved_draw_count);
        self.full_clear_pixel_count = self
            .full_clear_pixel_count
            .saturating_add(full_clear_pixel_count);
    }

    pub(super) fn emit(&self) {
        crate::profile_counter!(
            "render",
            "light_cookie_atlas_rebuild_count",
            self.rebuild_count
        );
        crate::profile_counter!(
            "render",
            "light_cookie_input_count",
            self.input_cookie_count
        );
        crate::profile_counter!(
            "render",
            "light_cookie_planned_entry_count",
            self.planned_entry_count,
        );
        crate::profile_counter!(
            "render",
            "light_cookie_resolved_draw_count",
            self.resolved_draw_count,
        );
        crate::profile_counter!(
            "render",
            "light_cookie_unresolved_entry_count",
            self.unresolved_entry_count,
        );
        crate::profile_counter!(
            "render",
            "light_cookie_blit_bind_group_create_count",
            self.blit_bind_group_create_count,
        );
        crate::profile_counter!(
            "render",
            "light_cookie_full_clear_pixel_count",
            self.full_clear_pixel_count,
        );
    }
}

fn count_as_u64(count: usize) -> u64 {
    count.try_into().unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_cookie_profile_aggregates_rebuild_work_without_extra_scans() {
        let mut profile = LightCookieAtlasProfile::default();

        profile.record_rebuild(7, 5, 3, 1_048_576);
        profile.record_rebuild(2, 2, 2, 1_048_576);

        assert_eq!(profile.rebuild_count, 2);
        assert_eq!(profile.input_cookie_count, 9);
        assert_eq!(profile.planned_entry_count, 7);
        assert_eq!(profile.resolved_draw_count, 5);
        assert_eq!(profile.unresolved_entry_count, 2);
        assert_eq!(profile.blit_bind_group_create_count, 5);
        assert_eq!(profile.full_clear_pixel_count, 2_097_152);

        profile.begin_frame();
        assert_eq!(profile, LightCookieAtlasProfile::default());
    }
}
