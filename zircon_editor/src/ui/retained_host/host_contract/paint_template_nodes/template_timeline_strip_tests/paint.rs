use super::super::super::super::paint_theme::{HostMaterialPalette, PALETTE};
use super::super::palette::timeline_palette_from_host;
use super::super::push_timeline_strip_commands;
use super::support::{changed_pixels, paint_timeline_strip, pixel_at, timeline_strip_node};
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn timeline_strip_palette_projects_from_shared_host_palette() {
    let mut host: HostMaterialPalette = PALETTE;
    host.surface_inset = [1, 2, 3, 255];
    host.separator_soft = [4, 5, 6, 255];
    host.accent = [7, 8, 9, 255];
    host.text = [10, 11, 12, 255];
    host.text_muted = [13, 14, 15, 255];

    let palette = timeline_palette_from_host(host);

    assert_eq!(palette.outer_surface, [1, 2, 3, 255]);
    assert_eq!(palette.grid_line, [4, 5, 6, 255]);
    assert_eq!(palette.playhead, [7, 8, 9, 255]);
    assert_eq!(palette.track_text, [10, 11, 12, 255]);
    assert_eq!(palette.tick_text, [13, 14, 15, 255]);
}

#[test]
fn timeline_strip_paints_ruler_track_playhead_and_selected_key() {
    let bytes = paint_timeline_strip(420, 150);

    assert!(changed_pixels(&bytes, [0, 0, 0, 255]) > 20_000);
    assert_ne!(
        pixel_at(&bytes, 420, 24, 28),
        pixel_at(&bytes, 420, 210, 28)
    );

    let accent_pixels = bytes
        .chunks_exact(4)
        .filter(|pixel| pixel[1] > 120 && pixel[2] > 135 && pixel[0] < 90)
        .count();
    assert!(
        accent_pixels > 80,
        "expected accent track/playhead/key pixels"
    );
}

#[test]
fn timeline_strip_runtime_text_and_geometry_scale_with_available_frame() {
    let compact = paint_timeline_strip(260, 112);
    let wide = paint_timeline_strip(620, 180);

    let compact_text = compact
        .chunks_exact(4)
        .filter(|pixel| pixel[0] > 135 && pixel[1] > 135 && pixel[2] > 135)
        .count();
    assert!(
        compact_text > 20,
        "expected Runtime Text pixels in compact strip"
    );
    assert!(changed_pixels(&wide, [0, 0, 0, 255]) > changed_pixels(&compact, [0, 0, 0, 255]));
}

#[test]
fn timeline_keys_emit_one_cached_anti_aliased_image_per_layer() {
    let node = timeline_strip_node(420.0, 150.0);
    let frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 420.0,
        height: 150.0,
    };
    let mut commands = Vec::new();

    assert!(push_timeline_strip_commands(
        &mut commands,
        &node,
        &frame,
        &frame,
        0,
        1.0,
    ));

    let diamond_images = commands
        .iter()
        .filter(|command| (9..=11).contains(&command.z_index))
        .map(|command| {
            assert_eq!(command.background_color, None);
            command
                .image_pixels
                .as_ref()
                .expect("timeline diamond layers should be cached pixel images")
        })
        .collect::<Vec<_>>();
    assert_eq!(diamond_images.len(), 5);
    assert_eq!(
        diamond_images
            .iter()
            .map(|image| (image.width, image.height))
            .collect::<Vec<_>>(),
        vec![(7, 7), (7, 7), (3, 3), (9, 9), (3, 3)]
    );
    assert!(diamond_images.iter().all(|image| image
        .resource_key
        .starts_with("icon-raster:analytic-diamond:")));
    assert!(diamond_images.iter().all(|image| image
        .rgba
        .chunks_exact(4)
        .any(|pixel| (1..=254).contains(&pixel[3]))));
}
