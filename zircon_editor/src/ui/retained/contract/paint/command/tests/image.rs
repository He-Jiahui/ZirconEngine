use super::*;
use crate::ui::retained_host::host_contract::paint_theme::{METRICS, PALETTE};

#[test]
fn render_command_image_border_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.border = [10, 11, 12, 255];
    palette.focus_ring = [13, 14, 15, 255];

    assert_eq!(fallback_image_border_from_host(palette), [10, 11, 12, 255]);
}

#[test]
fn render_command_image_frame_metrics_project_from_host_metrics() {
    assert_eq!(
        fallback_image_frame_metrics_from_host(METRICS),
        (METRICS.border_width, METRICS.radius_control)
    );
}

#[test]
fn render_command_image_fallback_text_metrics_project_from_host_metrics() {
    assert_eq!(
        HostPaintCommand::fallback_text_metrics_from_host(METRICS),
        (METRICS.font_body, METRICS.line_height(METRICS.font_body))
    );
}

#[test]
fn cached_image_commands_keep_the_resolved_source_path() {
    let path = Arc::new(std::path::PathBuf::from("resolved/actual-source.png"));
    let image = HostPaintImagePixels {
        resource_key: "test:cached".into(),
        source_path: Some(path.clone()),
        width: 1,
        height: 1,
        rgba: vec![255; 4].into(),
        atlas: None,
    };
    let image = std::thread::spawn(move || image).join().unwrap();
    let command = HostPaintCommand::from_image_pixels(FrameRect::default(), None, 0, image, 1.0);
    assert!(Arc::ptr_eq(
        command.image_pixels.unwrap().source_path.as_ref().unwrap(),
        &path
    ));
}
