use super::*;

#[test]
fn sprite_atlas_uv_rect_derives_from_pixel_rect() {
    let uv = SpriteAtlasUvRect::from_pixel_rect(
        SpriteAtlasRect {
            x: 16,
            y: 8,
            width: 32,
            height: 16,
        },
        128,
        64,
    )
    .expect("uv rect should derive from atlas extent");

    assert_eq!(uv.min, [0.125, 0.125]);
    assert_eq!(uv.max, [0.375, 0.375]);
}

#[test]
fn sprite_atlas_uv_rect_rejects_zero_atlas_extent() {
    let error = SpriteAtlasUvRect::from_pixel_rect(
        SpriteAtlasRect {
            x: 0,
            y: 0,
            width: 16,
            height: 16,
        },
        0,
        64,
    )
    .expect_err("zero atlas width must be rejected");

    assert_eq!(
        error,
        SpriteAtlasValidationError::ZeroAtlasDimensions {
            width: 0,
            height: 64
        }
    );
}

#[test]
fn sprite_atlas_uv_rect_rejects_zero_pixel_extent() {
    let error = SpriteAtlasUvRect::from_pixel_rect(
        SpriteAtlasRect {
            x: 0,
            y: 0,
            width: 0,
            height: 16,
        },
        64,
        64,
    )
    .expect_err("zero pixel width must be rejected");

    assert_eq!(
        error,
        SpriteAtlasValidationError::ZeroEntryDimensions {
            name: None,
            width: 0,
            height: 16,
        }
    );
}
