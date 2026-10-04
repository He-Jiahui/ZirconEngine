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
