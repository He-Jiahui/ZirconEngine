use std::collections::BTreeSet;

use super::super::GlyphAtlasPageKey;
use super::GlyphAtlasBitmapPageShadowPatch;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapPageShadowCommit {
    pub(crate) patches: Vec<GlyphAtlasBitmapPageShadowPatch>,
    pub(crate) zero_initialized_pages: BTreeSet<GlyphAtlasPageKey>,
    pub(crate) failed_zero_initialized_pages: BTreeSet<GlyphAtlasPageKey>,
}

impl GlyphAtlasBitmapPageShadowCommit {
    pub(crate) fn extend(&mut self, other: Self) {
        let Self {
            patches,
            mut zero_initialized_pages,
            mut failed_zero_initialized_pages,
        } = other;
        append_or_adopt_owned_vec(&mut self.patches, patches);
        self.zero_initialized_pages
            .append(&mut zero_initialized_pages);
        self.failed_zero_initialized_pages
            .append(&mut failed_zero_initialized_pages);
    }
}

fn append_or_adopt_owned_vec<T>(target: &mut Vec<T>, mut source: Vec<T>) {
    if target.is_empty() && target.capacity() == 0 {
        *target = source;
        return;
    }
    target.append(&mut source);
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;
    use crate::text::atlas::{GlyphAtlasFormat, GlyphAtlasRect};

    #[test]
    fn optimization_batch_ds_shadow_commit_adopts_owned_patch_storage() {
        let mut incoming = GlyphAtlasBitmapPageShadowCommit::default();
        incoming.patches = vec![patch(3), patch(4)];
        incoming
            .zero_initialized_pages
            .insert(GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 3));
        incoming
            .failed_zero_initialized_pages
            .insert(GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 4));
        let incoming_pointer = incoming.patches.as_ptr();
        let incoming_capacity = incoming.patches.capacity();

        let mut target = GlyphAtlasBitmapPageShadowCommit::default();
        target.extend(incoming);

        assert_eq!(target.patches.len(), 2);
        assert_eq!(target.patches.as_ptr(), incoming_pointer);
        assert_eq!(target.patches.capacity(), incoming_capacity);
        assert!(
            target
                .zero_initialized_pages
                .contains(&GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 3))
        );
        assert!(
            target
                .failed_zero_initialized_pages
                .contains(&GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 4))
        );
    }

    #[test]
    fn optimization_batch_ds_shadow_commit_uses_owned_collection_merges() {
        let production = include_str!("commit.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("shadow commit production source");

        assert!(production.contains("append_or_adopt_owned_vec(&mut self.patches, patches)"));
        assert!(production.contains(".append(&mut zero_initialized_pages)"));
        assert!(production.contains(".append(&mut failed_zero_initialized_pages)"));
        assert!(!production.contains("self.patches.extend(other.patches)"));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_ds_shadow_commit_owned_patch_storage_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const MERGES_PER_SAMPLE: usize = 2_048;
        const PATCHES_PER_MERGE: usize = 4_096;

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_owned_merges(
                    MERGES_PER_SAMPLE,
                    PATCHES_PER_MERGE,
                    false,
                ));
                optimized_samples.push(measure_owned_merges(
                    MERGES_PER_SAMPLE,
                    PATCHES_PER_MERGE,
                    true,
                ));
            } else {
                optimized_samples.push(measure_owned_merges(
                    MERGES_PER_SAMPLE,
                    PATCHES_PER_MERGE,
                    true,
                ));
                legacy_samples.push(measure_owned_merges(
                    MERGES_PER_SAMPLE,
                    PATCHES_PER_MERGE,
                    false,
                ));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "RUNTIME427_SHADOW_COMMIT_OWNED_PATCH_STORAGE_BENCH_V1 merges_per_sample={MERGES_PER_SAMPLE} patches_per_merge={PATCHES_PER_MERGE} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "shadow commit owned patch storage p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );

        fn measure_owned_merges(merge_count: usize, patch_count: usize, optimized: bool) -> u128 {
            let started_at = Instant::now();
            let mut checksum = 0_usize;
            for merge_index in 0..merge_count {
                let source = vec![merge_index as u64; patch_count];
                let mut target = Vec::new();
                if optimized {
                    super::append_or_adopt_owned_vec(&mut target, source);
                } else {
                    target.extend(source);
                }
                checksum = checksum.wrapping_add(target.len() ^ target.capacity());
                black_box(&target);
            }
            black_box(checksum);
            started_at.elapsed().as_nanos()
        }

        fn p95(samples: &mut [u128]) -> u128 {
            samples.sort_unstable();
            samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
        }
    }

    fn patch(page_index: u32) -> GlyphAtlasBitmapPageShadowPatch {
        GlyphAtlasBitmapPageShadowPatch {
            page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, page_index),
            page_generation: 1,
            target_rect: GlyphAtlasRect {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
            bytes_per_row: 1,
            bytes: vec![page_index as u8].into(),
        }
    }
}
