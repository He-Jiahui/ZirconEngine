use super::support::*;
use super::*;

#[test]
fn blend_space_workspace_adapts_between_compact_and_wide_windows() {
    let compact = open_blend_space_bridge(900, 620);
    let wide = open_blend_space_bridge(1260, 780);

    assert_compact_blend_space_geometry(&compact);
    assert_wide_blend_space_geometry(&wide);

    let compact_center = required_frame(&compact, "WorkbenchExtensionBlendSpaceCenterPanel");
    let wide_center = required_frame(&wide, "WorkbenchExtensionBlendSpaceCenterPanel");
    assert!(
        compact_center.width >= 560.0,
        "compact tier should preserve a useful primary editor lane: {compact_center:?}"
    );
    assert!(
        wide_center.width >= compact_center.width,
        "wide tier should add secondary panes without shrinking the primary editor lane: compact={compact_center:?}, wide={wide_center:?}"
    );
}
