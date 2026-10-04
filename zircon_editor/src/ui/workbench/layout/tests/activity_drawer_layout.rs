use super::*;

#[test]
fn new_right_drawers_reserve_the_token_panel_width_and_activity_rail() {
    let tokens = EditorDesignTokens::workbench_dark();
    for slot in [
        ActivityDrawerSlot::RightTop,
        ActivityDrawerSlot::RightBottom,
    ] {
        let drawer = ActivityDrawerLayout::new(slot);
        assert_eq!(
            drawer.extent - tokens.chrome.activity_rail_width,
            tokens.density.right_drawer_width
        );
    }
}

#[test]
fn persisted_right_extent_remains_a_user_preference_even_at_the_old_default() {
    for extent in [260.0, 417.0] {
        let mut drawer = ActivityDrawerLayout::new(ActivityDrawerSlot::RightTop);
        drawer.extent = extent;
        let encoded = serde_json::to_string(&drawer).unwrap();
        let restored: ActivityDrawerLayout = serde_json::from_str(&encoded).unwrap();
        assert_eq!(restored, drawer);
        assert_eq!(restored.extent, extent);
    }
    assert_eq!(
        ActivityDrawerLayout::new(ActivityDrawerSlot::LeftTop).extent,
        DEFAULT_SIDE_DRAWER_EXTENT
    );
}
