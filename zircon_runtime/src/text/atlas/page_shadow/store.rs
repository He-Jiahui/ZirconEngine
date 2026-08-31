use std::collections::{BTreeSet, HashMap};

use super::super::{GlyphAtlasPageKey, GlyphAtlasPageSpec};
use super::{
    GlyphAtlasBitmapPageShadow, GlyphAtlasBitmapPageShadowCommit, GlyphAtlasBitmapPageShadowPatch,
};

// This is a crate-local runtime budget: at the current 512x512 page size and
// default residency limit it covers every resident bitmap format (28 MiB),
// while retaining a hard cap if either policy changes independently.
const GLYPH_ATLAS_BITMAP_PAGE_SHADOW_MAX_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapPageShadowReport {
    pub(crate) resident_page_count: usize,
    pub(crate) resident_byte_count: usize,
    pub(crate) max_byte_count: usize,
    pub(crate) budget_rejection_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapPageShadowStore {
    pages: HashMap<GlyphAtlasPageKey, GlyphAtlasBitmapPageShadow>,
    byte_len: usize,
    max_byte_count: usize,
    budget_rejection_count: u64,
}

impl Default for GlyphAtlasBitmapPageShadowStore {
    fn default() -> Self {
        Self {
            pages: HashMap::new(),
            byte_len: 0,
            max_byte_count: GLYPH_ATLAS_BITMAP_PAGE_SHADOW_MAX_BYTES,
            budget_rejection_count: 0,
        }
    }
}

impl GlyphAtlasBitmapPageShadowStore {
    #[cfg(test)]
    fn with_max_bytes(max_byte_count: usize) -> Self {
        Self {
            max_byte_count: max_byte_count.max(1),
            ..Self::default()
        }
    }

    pub(crate) fn report(&self) -> GlyphAtlasBitmapPageShadowReport {
        GlyphAtlasBitmapPageShadowReport {
            resident_page_count: self.pages.len(),
            resident_byte_count: self.byte_len,
            max_byte_count: self.max_byte_count,
            budget_rejection_count: self.budget_rejection_count,
        }
    }

    pub(crate) fn bytes_for_page(&self, page: &GlyphAtlasPageSpec) -> Option<&[u8]> {
        self.pages
            .get(&page.key)
            .filter(|shadow| {
                shadow.generation == page.generation && shadow.bytes.len() == page.byte_len()
            })
            .map(|shadow| shadow.bytes.as_slice())
    }

    pub(crate) fn apply(
        &mut self,
        resident_pages: &[GlyphAtlasPageSpec],
        commit: GlyphAtlasBitmapPageShadowCommit,
    ) {
        self.retain_current_pages(resident_pages);
        let pages_by_key = resident_pages
            .iter()
            .map(|page| (page.key, page))
            .collect::<HashMap<_, _>>();
        let failed_zero_initialized_pages = commit.failed_zero_initialized_pages;
        let zero_initialized_pages = commit
            .zero_initialized_pages
            .into_iter()
            .filter(|page_key| !failed_zero_initialized_pages.contains(page_key))
            .collect::<BTreeSet<_>>();

        let mut unavailable_pages = BTreeSet::new();
        for page_key in &zero_initialized_pages {
            if let Some(page) = pages_by_key.get(page_key) {
                if !self.ensure_page(page) {
                    unavailable_pages.insert(*page_key);
                }
            }
        }

        for patch in commit.patches {
            if unavailable_pages.contains(&patch.page_key) {
                continue;
            }
            let Some(page) = pages_by_key.get(&patch.page_key) else {
                continue;
            };
            if page.generation != patch.page_generation {
                continue;
            }
            if !self.pages.contains_key(&patch.page_key)
                && zero_initialized_pages.contains(&patch.page_key)
            {
                self.ensure_page(page);
            }
            self.apply_patch(page, patch);
        }
    }

    pub(crate) fn remove_page(&mut self, page_key: GlyphAtlasPageKey) {
        if let Some(shadow) = self.pages.remove(&page_key) {
            self.byte_len = self.byte_len.saturating_sub(shadow.bytes.len());
        }
    }

    fn retain_current_pages(&mut self, resident_pages: &[GlyphAtlasPageSpec]) {
        let current_generations = resident_pages
            .iter()
            .map(|page| (page.key, page.generation))
            .collect::<HashMap<_, _>>();
        self.pages.retain(|page_key, shadow| {
            current_generations.get(page_key) == Some(&shadow.generation)
        });
        self.byte_len = self.pages.values().map(|shadow| shadow.bytes.len()).sum();
    }

    fn ensure_page(&mut self, page: &GlyphAtlasPageSpec) -> bool {
        if self
            .pages
            .get(&page.key)
            .is_some_and(|shadow| shadow.generation == page.generation)
        {
            return true;
        }

        self.remove_page(page.key);
        let page_byte_len = page.byte_len();
        if self.byte_len.saturating_add(page_byte_len) > self.max_byte_count {
            self.budget_rejection_count = self.budget_rejection_count.saturating_add(1);
            return false;
        }
        self.pages.insert(
            page.key,
            GlyphAtlasBitmapPageShadow {
                generation: page.generation,
                bytes: vec![0; page_byte_len],
            },
        );
        self.byte_len = self.byte_len.saturating_add(page_byte_len);
        true
    }

    fn apply_patch(&mut self, page: &GlyphAtlasPageSpec, patch: GlyphAtlasBitmapPageShadowPatch) {
        let bytes_per_pixel = page.storage_format.bytes_per_pixel() as usize;
        let target = patch.target_rect;
        let page_width = page.size.x as usize;
        let page_height = page.size.y as usize;
        let target_width = target.width as usize;
        let target_height = target.height as usize;
        let target_x = target.x as usize;
        let target_y = target.y as usize;
        let expected_bytes_per_row = target_width.saturating_mul(bytes_per_pixel);
        if target_x.saturating_add(target_width) > page_width
            || target_y.saturating_add(target_height) > page_height
            || patch.bytes_per_row as usize != expected_bytes_per_row
            || patch.bytes.len() != expected_bytes_per_row.saturating_mul(target_height)
        {
            return;
        }
        let Some(shadow) = self.pages.get_mut(&page.key) else {
            return;
        };
        if shadow.generation != page.generation {
            return;
        }

        for row in 0..target_height {
            let source_start = row.saturating_mul(expected_bytes_per_row);
            let source_end = source_start.saturating_add(expected_bytes_per_row);
            let destination_start = target_y
                .saturating_add(row)
                .saturating_mul(page_width)
                .saturating_add(target_x)
                .saturating_mul(bytes_per_pixel);
            let destination_end = destination_start.saturating_add(expected_bytes_per_row);
            shadow.bytes[destination_start..destination_end]
                .copy_from_slice(&patch.bytes[source_start..source_end]);
        }
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::{BTreeMap, HashMap};
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use crate::core::math::UVec2;

    use super::super::super::{GlyphAtlasFormat, GlyphAtlasRect};
    use super::*;

    const PAGE_INDEX_COUNT: usize = 32_768;
    const PAGE_LOOKUP_COUNT: usize = 262_144;
    const SAMPLE_COUNT: usize = 17;

    fn nearest_rank(samples: &mut [Duration], percentile: usize) -> Duration {
        samples.sort_unstable();
        let rank = samples.len().saturating_mul(percentile).div_ceil(100);
        samples[rank.saturating_sub(1)]
    }

    fn page_keys() -> Vec<GlyphAtlasPageKey> {
        let formats = GlyphAtlasFormat::supported_formats();
        (0..PAGE_INDEX_COUNT)
            .map(|index| GlyphAtlasPageKey::new(formats[index % formats.len()], index as u32))
            .collect()
    }

    fn lookup_keys(page_keys: &[GlyphAtlasPageKey]) -> Vec<GlyphAtlasPageKey> {
        (0..PAGE_LOOKUP_COUNT)
            .map(|index| page_keys[(index * 32_749) % page_keys.len()])
            .collect()
    }

    fn ordered_lookup_sum(
        index: &BTreeMap<GlyphAtlasPageKey, usize>,
        lookups: &[GlyphAtlasPageKey],
    ) -> usize {
        lookups
            .iter()
            .filter_map(|page_key| index.get(page_key))
            .copied()
            .sum()
    }

    fn hash_lookup_sum(
        index: &HashMap<GlyphAtlasPageKey, usize>,
        lookups: &[GlyphAtlasPageKey],
    ) -> usize {
        lookups
            .iter()
            .filter_map(|page_key| index.get(page_key))
            .copied()
            .sum()
    }

    #[test]
    fn runtime11c_batch_hash_shadow_store_preserves_generation_filter() {
        let page = GlyphAtlasPageSpec::new(
            GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 7),
            UVec2::new(8, 8),
        )
        .with_generation(3);
        let mut store = GlyphAtlasBitmapPageShadowStore::default();
        let mut commit = GlyphAtlasBitmapPageShadowCommit::default();
        commit.zero_initialized_pages.insert(page.key);
        store.apply(std::slice::from_ref(&page), commit);

        assert_eq!(store.bytes_for_page(&page), Some(&[0; 64][..]));

        let next_generation = page.clone().with_generation(4);
        store.apply(
            std::slice::from_ref(&next_generation),
            GlyphAtlasBitmapPageShadowCommit::default(),
        );
        assert!(store.bytes_for_page(&page).is_none());
        assert!(store.bytes_for_page(&next_generation).is_none());
    }

    #[test]
    fn bitmap_page_shadow_report_exposes_residency_budget_and_rejections() {
        let page = GlyphAtlasPageSpec::new(
            GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 9),
            UVec2::new(8, 8),
        );
        let mut store = GlyphAtlasBitmapPageShadowStore::with_max_bytes(63);
        let mut commit = GlyphAtlasBitmapPageShadowCommit::default();
        commit.zero_initialized_pages.insert(page.key);
        commit.patches.push(GlyphAtlasBitmapPageShadowPatch {
            page_key: page.key,
            page_generation: page.generation,
            target_rect: GlyphAtlasRect {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
            bytes_per_row: 1,
            bytes: vec![255].into(),
        });

        store.apply(std::slice::from_ref(&page), commit);

        assert_eq!(
            store.report(),
            GlyphAtlasBitmapPageShadowReport {
                resident_page_count: 0,
                resident_byte_count: 0,
                max_byte_count: 63,
                budget_rejection_count: 1,
            }
        );
    }

    #[test]
    fn runtime11c_batch_page_shadow_uses_hash_indexes() {
        let source = include_str!("store.rs");
        let production = source.split("mod optimization_tests").next().unwrap();

        assert!(production.contains("use std::collections::{BTreeSet, HashMap};"));
        assert!(production.contains("pages: HashMap<GlyphAtlasPageKey"));
        assert_eq!(production.matches("collect::<HashMap<_, _>>()").count(), 2);
        assert_eq!(production.matches("collect::<BTreeSet<_>>()").count(), 1);
        assert!(!production.contains("BTreeMap"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn runtime11c_batch_page_shadow_hash_index_performance_evidence() {
        let page_keys = page_keys();
        let lookups = lookup_keys(&page_keys);
        let ordered_index = page_keys
            .iter()
            .enumerate()
            .map(|(value, page_key)| (*page_key, value))
            .collect::<BTreeMap<_, _>>();
        let hash_index = page_keys
            .iter()
            .enumerate()
            .map(|(value, page_key)| (*page_key, value))
            .collect::<HashMap<_, _>>();
        assert_eq!(
            ordered_lookup_sum(&ordered_index, &lookups),
            hash_lookup_sum(&hash_index, &lookups)
        );

        let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(ordered_lookup_sum(
                    black_box(&ordered_index),
                    black_box(&lookups),
                ));
                ordered_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(hash_lookup_sum(black_box(&hash_index), black_box(&lookups)));
                hash_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(hash_lookup_sum(black_box(&hash_index), black_box(&lookups)));
                hash_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(ordered_lookup_sum(
                    black_box(&ordered_index),
                    black_box(&lookups),
                ));
                ordered_samples.push(started.elapsed());
            }
        }

        let ordered_p50 = nearest_rank(&mut ordered_samples.clone(), 50);
        let ordered_p95 = nearest_rank(&mut ordered_samples, 95);
        let hash_p50 = nearest_rank(&mut hash_samples.clone(), 50);
        let hash_p95 = nearest_rank(&mut hash_samples, 95);
        println!(
            "RUNTIME11C_PAGE_SHADOW_HASH_INDEX_BENCH_V1 pages={PAGE_INDEX_COUNT} \
             lookups={PAGE_LOOKUP_COUNT} sample_pairs={SAMPLE_COUNT} \
             pair_order=alternating_ordered_even ordered_first_pairs=9 hash_first_pairs=8 \
             ordered_lookup_class=log_n hash_lookup_class=average_constant \
             ordered_p50_ns={} ordered_p95_ns={} hash_p50_ns={} hash_p95_ns={} \
             persistent_hash_indexes=3 ordered_zero_init_sets=1",
            ordered_p50.as_nanos(),
            ordered_p95.as_nanos(),
            hash_p50.as_nanos(),
            hash_p95.as_nanos(),
        );
        assert!(
            hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
            "hash-index P95 {:?} exceeded 60% of ordered-index P95 {:?}",
            hash_p95,
            ordered_p95,
        );
    }
}
