use super::*;

#[test]
fn blend_space_workspace_preserves_the_sample_canvas_at_ultra_tier() {
    let ultra = open_blend_space_bridge(480, 520);
    let workspace = required_frame(&ultra, "WorkbenchExtensionBlendSpaceWorkspace");
    let center = required_frame(&ultra, "WorkbenchExtensionBlendSpaceCenterPanel");
    let canvas = required_frame(&ultra, "WorkbenchExtensionBlendSpaceSampleCanvas");
    let grid = required_frame(&ultra, "WorkbenchExtensionBlendSpaceSampleGrid");

    for control_id in [
        "WorkbenchExtensionBlendSpaceRailGap",
        "WorkbenchExtensionBlendSpaceLeftPanel",
        "WorkbenchExtensionBlendSpaceRightPanel",
        "WorkbenchExtensionBlendSpacePreviewCard",
        "WorkbenchExtensionBlendSpaceOutputPanel",
    ] {
        assert!(
            ultra.control_frame(control_id).is_none(),
            "ultra tier should collapse `{control_id}` before reducing the sample canvas"
        );
    }

    assert!(center.x >= workspace.x && center.right() <= workspace.right() + 0.5);
    assert!(canvas.x >= center.x && canvas.right() <= center.right() + 0.5);
    assert!(grid.x >= canvas.x && grid.right() <= canvas.right() + 0.5);
    assert!(grid.y >= canvas.y && grid.bottom() <= canvas.bottom() + 0.5);
    assert!(
        grid.width >= workspace.width * 0.7,
        "ultra tier should reserve most horizontal workspace for the sample grid: {grid:?}"
    );
    assert!(
        grid.height >= workspace.height * 0.65,
        "ultra tier should reserve most vertical workspace for the sample grid: {grid:?}"
    );
}
