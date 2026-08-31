use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ops::Range;
use std::sync::Arc;

use crate::text::atlas::GlyphRasterKey;
use crate::text::native_bitmap_atlas::NativeBitmapAtlasGlyphRun;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct NativeBitmapAtlasGlyphLocation {
    pub(super) run_index: usize,
    pub(super) glyph_index: usize,
}

#[derive(Debug, Default)]
pub(super) struct NativeBitmapAtlasSegmentDependencyIndex {
    ordered_keys: Arc<[GlyphRasterKey]>,
    spans_by_key: HashMap<GlyphRasterKey, Range<usize>>,
    locations: Arc<[NativeBitmapAtlasGlyphLocation]>,
}

#[derive(Debug, Default)]
pub(super) struct NativeBitmapAtlasFrameDependencyIndex {
    spans_by_key: HashMap<GlyphRasterKey, Range<usize>>,
    segment_indices: Arc<[usize]>,
}

impl NativeBitmapAtlasSegmentDependencyIndex {
    pub(super) fn from_glyph_runs(glyph_runs: &[NativeBitmapAtlasGlyphRun]) -> Self {
        let mut ordered_keys = Vec::new();
        let mut counts_by_key: HashMap<GlyphRasterKey, usize> = HashMap::new();
        let mut glyph_count = 0usize;
        for glyph in glyph_runs.iter().flat_map(|run| &run.glyphs) {
            glyph_count = glyph_count.saturating_add(1);
            match counts_by_key.entry(glyph.raster_key) {
                Entry::Occupied(mut entry) => {
                    let count = entry.get_mut();
                    *count = count.saturating_add(1);
                }
                Entry::Vacant(entry) => {
                    ordered_keys.push(glyph.raster_key);
                    entry.insert(1usize);
                }
            }
        }

        let spans_by_key = dependency_spans(&ordered_keys, &counts_by_key);
        let mut next_by_key = spans_by_key
            .iter()
            .map(|(key, span)| (*key, span.start))
            .collect::<HashMap<_, _>>();
        let mut locations = vec![NativeBitmapAtlasGlyphLocation::default(); glyph_count];
        for (run_index, run) in glyph_runs.iter().enumerate() {
            for (glyph_index, glyph) in run.glyphs.iter().enumerate() {
                let next = next_by_key
                    .get_mut(&glyph.raster_key)
                    .expect("counted native glyph dependency must retain a write cursor");
                locations[*next] = NativeBitmapAtlasGlyphLocation {
                    run_index,
                    glyph_index,
                };
                *next = next.saturating_add(1);
            }
        }

        let index = Self {
            ordered_keys: Arc::from(ordered_keys),
            spans_by_key,
            locations: Arc::from(locations),
        };
        debug_assert!(index.locations_are_valid_for(glyph_runs));
        index
    }

    pub(super) fn dependency_count(&self) -> usize {
        self.spans_by_key.len()
    }

    pub(super) fn instance_count(&self) -> usize {
        self.locations.len()
    }

    fn ordered_keys(&self) -> &[GlyphRasterKey] {
        &self.ordered_keys
    }

    fn locations_are_valid_for(&self, glyph_runs: &[NativeBitmapAtlasGlyphRun]) -> bool {
        self.ordered_keys().iter().copied().all(|key| {
            self.locations_for(key).iter().all(|location| {
                glyph_runs
                    .get(location.run_index)
                    .is_some_and(|run| location.glyph_index < run.glyphs.len())
            })
        })
    }

    pub(super) fn locations_for(&self, key: GlyphRasterKey) -> &[NativeBitmapAtlasGlyphLocation] {
        self.spans_by_key
            .get(&key)
            .map_or(&[], |span| &self.locations[span.clone()])
    }
}

impl NativeBitmapAtlasFrameDependencyIndex {
    pub(super) fn from_segment_indexes<'a, Indexes>(segment_indexes: Indexes) -> Self
    where
        Indexes: Clone + Iterator<Item = &'a NativeBitmapAtlasSegmentDependencyIndex>,
    {
        let mut ordered_keys = Vec::new();
        let mut counts_by_key: HashMap<GlyphRasterKey, usize> = HashMap::new();
        for index in segment_indexes.clone() {
            for key in index.ordered_keys() {
                match counts_by_key.entry(*key) {
                    Entry::Occupied(mut entry) => {
                        let count = entry.get_mut();
                        *count = count.saturating_add(1);
                    }
                    Entry::Vacant(entry) => {
                        ordered_keys.push(*key);
                        entry.insert(1usize);
                    }
                }
            }
        }

        let spans_by_key = dependency_spans(&ordered_keys, &counts_by_key);
        let mut next_by_key = spans_by_key
            .iter()
            .map(|(key, span)| (*key, span.start))
            .collect::<HashMap<_, _>>();
        let segment_index_count = counts_by_key.values().copied().sum();
        let mut segment_indices = vec![0usize; segment_index_count];
        for (segment_index, index) in segment_indexes.enumerate() {
            for key in index.ordered_keys() {
                let next = next_by_key
                    .get_mut(key)
                    .expect("counted frame glyph dependency must retain a write cursor");
                segment_indices[*next] = segment_index;
                *next = next.saturating_add(1);
            }
        }

        let index = Self {
            spans_by_key,
            segment_indices: Arc::from(segment_indices),
        };
        debug_assert!(index.segment_fanout_is_strictly_ordered());
        index
    }

    pub(super) fn dependency_count(&self) -> usize {
        self.spans_by_key.len()
    }

    pub(super) fn segment_entry_count(&self) -> usize {
        self.segment_indices.len()
    }

    pub(super) fn segment_indices_for(&self, key: GlyphRasterKey) -> &[usize] {
        self.spans_by_key
            .get(&key)
            .map_or(&[], |span| &self.segment_indices[span.clone()])
    }

    fn segment_fanout_is_strictly_ordered(&self) -> bool {
        self.spans_by_key.keys().copied().all(|key| {
            self.segment_indices_for(key)
                .windows(2)
                .all(|pair| pair[0] < pair[1])
        })
    }
}

fn dependency_spans(
    ordered_keys: &[GlyphRasterKey],
    counts_by_key: &HashMap<GlyphRasterKey, usize>,
) -> HashMap<GlyphRasterKey, Range<usize>> {
    let mut spans_by_key = HashMap::with_capacity(ordered_keys.len());
    let mut start = 0usize;
    for key in ordered_keys {
        let count = counts_by_key.get(key).copied().unwrap_or_default();
        let end = start.saturating_add(count);
        spans_by_key.insert(*key, start..end);
        start = end;
    }
    spans_by_key
}

#[cfg(test)]
mod tests {
    use crate::text::InstancedFaceId;
    use crate::text::atlas::render_plan::GlyphAtlasScreenRect;
    use crate::text::atlas::{
        GlyphAtlasFormat, GlyphHintingMode, GlyphSmoothingMode, SyntheticGlyphStyle,
    };
    use crate::text::native_bitmap_atlas::{NativeBitmapAtlasGlyph, NativeBitmapAtlasGlyphRun};

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
        let second =
            NativeBitmapAtlasSegmentDependencyIndex::from_glyph_runs(&[glyph_run(&[shared])]);
        let indexes = [&first, &second];
        let frame =
            NativeBitmapAtlasFrameDependencyIndex::from_segment_indexes(indexes.into_iter());

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
}
