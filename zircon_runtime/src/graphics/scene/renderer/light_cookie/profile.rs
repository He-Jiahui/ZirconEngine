#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// 汇总同一渲染帧中图集重建的输入、实际绘制与未解析纹理开销；
/// 帧入口清零，成功提交时由场景渲染器统一发出计数。
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
#[path = "tests/profile.rs"]
mod tests;
