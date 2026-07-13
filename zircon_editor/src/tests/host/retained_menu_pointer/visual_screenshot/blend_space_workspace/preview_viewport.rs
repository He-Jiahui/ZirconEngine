use super::*;

#[test]
fn blend_space_workspace_collapses_secondary_panes_at_narrow_tier() {
    let narrow = open_blend_space_bridge(640, 520);
    let workspace = required_frame(&narrow, "WorkbenchExtensionBlendSpaceWorkspace");
    let left = required_frame(&narrow, "WorkbenchExtensionBlendSpaceLeftPanel");
    let center = required_frame(&narrow, "WorkbenchExtensionBlendSpaceCenterPanel");
    let canvas = required_frame(&narrow, "WorkbenchExtensionBlendSpaceSampleCanvas");
    let grid = required_frame(&narrow, "WorkbenchExtensionBlendSpaceSampleGrid");

    assert!(
        narrow
            .control_frame("WorkbenchExtensionBlendSpaceRightPanel")
            .is_none(),
        "narrow tier should collapse the secondary detail pane"
    );
    assert!(
        narrow
            .control_frame("WorkbenchExtensionBlendSpacePreviewCard")
            .is_none(),
        "narrow tier should collapse the secondary preview card"
    );
    assert!(
        narrow
            .control_frame("WorkbenchExtensionBlendSpaceOutputPanel")
            .is_none(),
        "narrow tier should collapse the fixed output pane in favor of the sample canvas"
    );
    assert!(left.x >= workspace.x && left.right() <= center.x + 0.5);
    assert!(center.right() <= workspace.right() + 0.5);
    assert!(canvas.x >= center.x && canvas.right() <= center.right() + 0.5);
    assert!(grid.x >= canvas.x && grid.right() <= canvas.right() + 0.5);
    assert!(grid.y >= canvas.y && grid.bottom() <= canvas.bottom() + 0.5);
    assert!(
        canvas.width >= 180.0,
        "sample canvas should own the narrow editing budget: {canvas:?}"
    );
    assert!(
        canvas.height >= 260.0,
        "sample canvas should own the narrow vertical editing budget: {canvas:?}"
    );
}

#[test]
fn blend_space_preview_viewport_keeps_mannequin_subject_legible() {
    std::env::set_var("SLINT_BACKEND", "software");

    let width = 1260;
    let height = 780;
    let ui = blend_space_window(width, height);
    let presentation = ui.get_host_presentation();
    let preview = (0..presentation.workbench_window_nodes.row_count())
        .filter_map(|row| presentation.workbench_window_nodes.row_data(row))
        .find(|node| node.control_id.as_str() == "WorkbenchExtensionBlendSpacePreviewViewportImage")
        .expect("wide Blend Space should project the shared preview viewport image");
    let painted = paint_host_frame_for_test(width, height, &presentation);
    let subject_pixels =
        neutral_bright_frame_pixel_count(&painted, width, &preview.frame, (0.34, 0.12, 0.66, 0.92));

    assert!(
        subject_pixels >= 520,
        "the shared preview asset must keep the mannequin legible inside the compact wide-tier viewport: subject_pixels={subject_pixels}, frame=({}, {}, {}, {})",
        preview.frame.x,
        preview.frame.y,
        preview.frame.width,
        preview.frame.height,
    );
}

fn neutral_bright_frame_pixel_count(
    bytes: &[u8],
    frame_width: u32,
    frame: &TemplateNodeFrameData,
    relative_region: (f32, f32, f32, f32),
) -> usize {
    let (left_ratio, top_ratio, right_ratio, bottom_ratio) = relative_region;
    let frame_height = (bytes.len() as u32 / 4) / frame_width;
    let left = (frame.x + frame.width * left_ratio)
        .max(0.0)
        .floor()
        .min(frame_width as f32) as u32;
    let top = (frame.y + frame.height * top_ratio)
        .max(0.0)
        .floor()
        .min(frame_height as f32) as u32;
    let right = (frame.x + frame.width * right_ratio)
        .max(0.0)
        .ceil()
        .min(frame_width as f32) as u32;
    let bottom = (frame.y + frame.height * bottom_ratio)
        .max(0.0)
        .ceil()
        .min(frame_height as f32) as u32;

    let mut count = 0;
    for y in top..bottom {
        for x in left..right {
            let offset = ((y * frame_width + x) * 4) as usize;
            let Some(pixel) = bytes.get(offset..offset + 4) else {
                continue;
            };
            let minimum = pixel[0].min(pixel[1]).min(pixel[2]);
            let maximum = pixel[0].max(pixel[1]).max(pixel[2]);
            let luminance = (u16::from(pixel[0]) + u16::from(pixel[1]) + u16::from(pixel[2])) / 3;
            if pixel[3] > 0 && luminance >= 90 && maximum - minimum <= 65 {
                count += 1;
            }
        }
    }
    count
}
