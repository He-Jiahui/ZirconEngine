use super::*;

#[test]
fn cookie_frame_plan_is_sorted_deduplicated_and_uses_fixed_atlas_cells() {
    let texture_a = ResourceId::from_stable_label("runtime://cookie/a");
    let texture_b = ResourceId::from_stable_label("runtime://cookie/b");
    let plan = build_cookie_frame_plan(&[
        LightCookieData {
            light_id: 9,
            texture: texture_a,
            projection: CookieProjection::Spot,
        },
        LightCookieData {
            light_id: 3,
            texture: texture_b,
            projection: CookieProjection::PointOctahedral,
        },
        LightCookieData {
            light_id: 9,
            texture: texture_b,
            projection: CookieProjection::Directional {
                offset: Vec2::new(0.25, 0.5),
                scale: Vec2::new(2.0, 3.0),
                wrap: CookieWrapMode::Repeat,
            },
        },
    ]);

    assert_eq!(plan.entries().len(), 2);
    assert_eq!(plan.entries()[0].light_id, 3);
    assert_eq!(plan.entries()[1].light_id, 9);
    assert_eq!(plan.entries()[0].slot, 0);
    assert_eq!(plan.entries()[1].slot, 1);
    assert_eq!(plan.entries()[0].metadata.uv_rect, [0.0, 0.0, 0.125, 0.125]);
    assert_eq!(
        plan.entries()[1].metadata.uv_rect,
        [0.125, 0.0, 0.125, 0.125]
    );
    assert_eq!(
        plan.entries()[1].metadata.misc,
        [COOKIE_PROJECTION_DIRECTIONAL, 1, 0, 0]
    );
    assert_eq!(
        plan.entries()[1].metadata.directional_offset_scale,
        [0.25, 0.5, 2.0, 3.0]
    );
}
