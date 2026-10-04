use super::apply_rounded_alpha_mask;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_frame::{
    HostPaintAtlasImage, HostPaintImageUvRect,
};
use crate::ui::retained_host::host_contract::paint_template_nodes::visual_assets::HostPaintImagePixels;

fn image(alpha: u8, atlas: Option<HostPaintAtlasImage>) -> HostPaintImagePixels {
    let mut rgba = vec![255; 8 * 8 * 4];
    for pixel in rgba.chunks_exact_mut(4) {
        pixel[3] = alpha;
    }
    HostPaintImagePixels {
        resource_key: "avatar-source".to_owned(),
        width: 8,
        height: 8,
        rgba: rgba.into(),
        atlas,
    }
}

#[test]
fn rounded_avatar_mask_uses_fractional_coverage_and_preserves_source_alpha() {
    let mut image = image(128, None);

    apply_rounded_alpha_mask(
        &mut image,
        4.0,
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 8.0,
            height: 8.0,
        },
    );

    let alpha = image
        .rgba
        .chunks_exact(4)
        .map(|pixel| pixel[3])
        .collect::<Vec<_>>();
    assert!(alpha.contains(&0));
    assert!(alpha.contains(&128));
    assert!(alpha.iter().any(|value| *value > 0 && *value < 128));
}

#[test]
fn rounded_avatar_mask_invalidates_the_unmasked_atlas_fast_path() {
    let atlas = HostPaintAtlasImage {
        resource_key: "avatar-atlas".to_owned(),
        resource_generation: 7,
        width: 64,
        height: 64,
        rgba: None,
        uv: HostPaintImageUvRect {
            min: [0.0, 0.0],
            max: [0.5, 0.5],
        },
    };
    let mut image = image(255, Some(atlas));

    apply_rounded_alpha_mask(
        &mut image,
        4.0,
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 8.0,
            height: 8.0,
        },
    );

    assert!(image.atlas.is_none());
}
