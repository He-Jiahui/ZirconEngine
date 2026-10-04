use crate::text::atlas::render_plan::GlyphAtlasScreenRect;
use crate::text::atlas::{
    GlyphAtlasFormat, GlyphHintingMode, GlyphSmoothingMode, SyntheticGlyphStyle,
};
use crate::text::native_bitmap_atlas::{NativeBitmapAtlasGlyph, NativeBitmapAtlasGlyphRun};
use crate::text::InstancedFaceId;

use super::*;

#[test]
fn segment_dependency_index_preserves_run_and_glyph_order_per_key() {
    let first = raster_key(1);
    let second = raster_key(2);
    let index = NativeBitmapAtlasSegmentDependencyIndex::from_glyph_runs(&[
        glyph_run(&[first, second]),
        glyph_run(&[first]),
    ]);

    assert_eq!(index.dependency_count(), 2);
    assert_eq!(index.instance_count(), 3);
    assert_eq!(
        index.locations_for(first),
        &[
            NativeBitmapAtlasGlyphLocation {
                run_index: 0,
                glyph_index: 0,
            },
            NativeBitmapAtlasGlyphLocation {
                run_index: 1,
                glyph_index: 0,
            },
        ]
    );
    assert_eq!(
        index.locations_for(second),
        &[NativeBitmapAtlasGlyphLocation {
            run_index: 0,
            glyph_index: 1,
        }]
    );
}

#[test]
fn frame_dependency_index_deduplicates_keys_within_each_segment() {
    let shared = raster_key(3);
    let first_only = raster_key(4);
    let first = NativeBitmapAtlasSegmentDependencyIndex::from_glyph_runs(&[glyph_run(&[
        shared, shared, first_only,
    ])]);
    let second = NativeBitmapAtlasSegmentDependencyIndex::from_glyph_runs(&[glyph_run(&[shared])]);
    let indexes = [&first, &second];
    let frame = NativeBitmapAtlasFrameDependencyIndex::from_segment_indexes(indexes.into_iter());

    assert_eq!(frame.dependency_count(), 2);
    assert_eq!(frame.segment_entry_count(), 3);
    assert_eq!(frame.segment_indices_for(shared), &[0, 1]);
    assert_eq!(frame.segment_indices_for(first_only), &[0]);
}

fn glyph_run(keys: &[GlyphRasterKey]) -> NativeBitmapAtlasGlyphRun {
    NativeBitmapAtlasGlyphRun::new(
        GlyphAtlasScreenRect::new(0.0, 0.0, 128.0, 64.0),
        keys.iter()
            .copied()
            .map(|raster_key| NativeBitmapAtlasGlyph {
                raster_key,
                screen_x: 0.0,
                baseline_y: 0.0,
                placeholder_rect: GlyphAtlasScreenRect::new(0.0, 0.0, 1.0, 1.0),
                foreground_color: [1.0; 4],
                background_color: None,
            })
            .collect(),
    )
}

fn raster_key(glyph_id: u32) -> GlyphRasterKey {
    GlyphRasterKey {
        face: InstancedFaceId(1),
        glyph_id,
        px_size_bucket: 16,
        subpixel_bin: 0,
        vertical_subpixel_bin: 0,
        format: GlyphAtlasFormat::AlphaMask,
        hinting: GlyphHintingMode::Full,
        smoothing: GlyphSmoothingMode::Grayscale,
        synthetic: SyntheticGlyphStyle::default(),
    }
}
