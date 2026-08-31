use std::collections::HashSet;
use std::sync::{Arc, Weak};

mod native_dependency_index;
mod run_index;

use crate::core::math::UVec2;

use self::native_dependency_index::{
    NativeBitmapAtlasFrameDependencyIndex, NativeBitmapAtlasSegmentDependencyIndex,
};
use self::run_index::ScreenSpaceUiTextFrameRunIndex;
use super::ScreenSpaceUiTextFrameProductGeneration;
use super::font_assets::UiFontAssetCache;
use super::font_id_report::ScreenSpaceUiTextFontIdReport;
use super::native_bitmap_atlas_glyph_runs;
use super::native_glyph_run::NativeBitmapAtlasGlyphRunProjection;
use super::prepare_report::ScreenSpaceUiResolvedTextReport;
use super::resolved_batches::{
    AutoTextRasterRouter, ResolvedScreenSpaceUiTextBatches,
    resolve_text_batches_after_font_dependencies,
};
use crate::graphics::scene::scene_renderer::ui::render::{
    PlannedScreenSpaceUi, ScreenSpaceUiTextBatch, ScreenSpaceUiTextRouteIdentity,
};
use crate::text::font::{FontCollectionRevision, FontCollectionService};
use crate::text::native_bitmap_atlas::NativeBitmapAtlasGlyphRun;

#[derive(Default)]
pub(super) struct ScreenSpaceUiTextSegmentCache {
    font_dependency_entries: Vec<ScreenSpaceUiTextFontDependencyEntry>,
    active_font_dependencies: Vec<Arc<str>>,
    active_font_dependency_set: HashSet<Arc<str>>,
    segment_product_entries: Vec<ScreenSpaceUiTextSegmentProductEntry>,
    frame_product_generation_counter: u64,
    frame_segments: Vec<Weak<PlannedScreenSpaceUi>>,
    frame_viewport_size: UVec2,
    frame_font_revision: Option<FontCollectionRevision>,
    frame_product: Option<Arc<ScreenSpaceUiTextFrameProduct>>,
}

struct ScreenSpaceUiTextFontDependencyEntry {
    plan: Weak<PlannedScreenSpaceUi>,
    assets: Arc<[Arc<str>]>,
}

struct ScreenSpaceUiTextSegmentProductEntry {
    plan: Weak<PlannedScreenSpaceUi>,
    viewport_size: UVec2,
    font_revision: FontCollectionRevision,
    product: Arc<ScreenSpaceUiTextSegmentProduct>,
}

pub(super) struct ScreenSpaceUiTextSegmentProduct {
    resolved_texts: ResolvedScreenSpaceUiTextBatches,
    resolved_report: ScreenSpaceUiResolvedTextReport,
    native_glyph_runs: NativeBitmapAtlasGlyphRunProjection,
    native_glyph_dependencies: NativeBitmapAtlasSegmentDependencyIndex,
    input_batch_counts: [usize; 3],
    auto_routes: Arc<[ScreenSpaceUiTextRouteIdentity]>,
}

pub(super) struct ScreenSpaceUiTextFrameProduct {
    generation: ScreenSpaceUiTextFrameProductGeneration,
    segment_products: Arc<[Arc<ScreenSpaceUiTextSegmentProduct>]>,
    resolved_report: ScreenSpaceUiResolvedTextReport,
    native_font_ids: ScreenSpaceUiTextFontIdReport,
    input_batch_counts: [usize; 3],
    native_glyph_dependencies: NativeBitmapAtlasFrameDependencyIndex,
    native_reverse_instance_entry_count: usize,
    run_index: ScreenSpaceUiTextFrameRunIndex,
}

impl ScreenSpaceUiTextSegmentCache {
    pub(super) fn refresh_font_dependencies(
        &mut self,
        render_segments: &[Arc<PlannedScreenSpaceUi>],
    ) {
        let previous_entries = std::mem::take(&mut self.font_dependency_entries);
        let mut previous_entries = previous_entries.into_iter();
        let mut next_entries = Vec::with_capacity(render_segments.len());
        for plan in render_segments {
            let previous = previous_entries.next();
            if previous
                .as_ref()
                .is_some_and(|entry| segment_plan_reused(Some(&entry.plan), plan))
            {
                next_entries.push(previous.expect("matching dependency entry must exist"));
                continue;
            }
            next_entries.push(ScreenSpaceUiTextFontDependencyEntry {
                plan: Arc::downgrade(plan),
                assets: collect_segment_font_dependencies(plan),
            });
        }
        self.font_dependency_entries = next_entries;

        self.active_font_dependencies.clear();
        self.active_font_dependency_set.clear();
        for entry in &self.font_dependency_entries {
            for asset in entry.assets.iter() {
                if self.active_font_dependency_set.insert(Arc::clone(asset)) {
                    self.active_font_dependencies.push(Arc::clone(asset));
                }
            }
        }
    }

    pub(super) fn active_font_dependencies(&self) -> &[Arc<str>] {
        &self.active_font_dependencies
    }

    pub(super) fn prepare_frame_product(
        &mut self,
        render_segments: &[Arc<PlannedScreenSpaceUi>],
        viewport_size: UVec2,
        font_revision: FontCollectionRevision,
        font_assets: &UiFontAssetCache,
        auto_router: &mut AutoTextRasterRouter,
        shaping_changed: bool,
        font_collection: &Arc<FontCollectionService>,
    ) -> Arc<ScreenSpaceUiTextFrameProduct> {
        if self.frame_matches(render_segments, viewport_size, font_revision) {
            let product = self
                .frame_product
                .as_ref()
                .expect("matching text frame cache key must retain its product");
            record_segment_cache_profile(
                true,
                render_segments.len(),
                0,
                0,
                product.native_glyph_dependencies.dependency_count(),
                product.native_reverse_instance_entry_count,
                product.native_glyph_dependencies.segment_entry_count(),
                product.run_index.spans().len(),
                product.run_index.native_run_count(),
                product.run_index.sdf_run_count(),
                0,
                0,
            );
            return Arc::clone(product);
        }

        let previous_entries = std::mem::take(&mut self.segment_product_entries);
        let mut previous_entries = previous_entries.into_iter();
        let mut next_entries = Vec::with_capacity(render_segments.len());
        let mut segment_products = Vec::with_capacity(render_segments.len());
        let mut segment_product_reuse_count = 0_usize;
        let mut text_batch_visit_count = 0_usize;
        let mut glyph_projection_count = 0_usize;
        for plan in render_segments {
            let previous = previous_entries.next();
            if let Some(previous) = previous.filter(|entry| {
                segment_product_entry_reused(entry, plan, viewport_size, font_revision)
            }) {
                segment_product_reuse_count = segment_product_reuse_count.saturating_add(1);
                segment_products.push(Arc::clone(&previous.product));
                next_entries.push(previous);
                continue;
            }

            let product = Arc::new(build_segment_product(
                plan,
                viewport_size,
                font_assets,
                auto_router,
                shaping_changed,
                font_revision,
                font_collection,
            ));
            text_batch_visit_count = text_batch_visit_count
                .saturating_add(product.input_batch_counts.iter().copied().sum::<usize>());
            glyph_projection_count = glyph_projection_count.saturating_add(
                product
                    .native_glyph_runs
                    .glyph_runs
                    .iter()
                    .map(|run| run.glyphs.len())
                    .sum::<usize>(),
            );
            next_entries.push(ScreenSpaceUiTextSegmentProductEntry {
                plan: Arc::downgrade(plan),
                viewport_size,
                font_revision,
                product: Arc::clone(&product),
            });
            segment_products.push(product);
        }
        self.segment_product_entries = next_entries;
        auto_router.replace_active_routes(
            segment_products
                .iter()
                .flat_map(|product| product.auto_routes.iter().cloned()),
        );

        let mut resolved_report = ScreenSpaceUiResolvedTextReport::default();
        let mut native_font_ids = ScreenSpaceUiTextFontIdReport::default();
        let mut input_batch_counts = [0_usize; 3];
        for product in &segment_products {
            resolved_report.merge(product.resolved_report);
            accumulate_font_id_report(&mut native_font_ids, product.native_glyph_runs.font_ids);
            for (total, count) in input_batch_counts
                .iter_mut()
                .zip(product.input_batch_counts)
            {
                *total = total.saturating_add(count);
            }
        }
        let compatibility_batch_clone_count = 0;
        let compatibility_glyph_run_clone_count = 0;
        let native_reverse_instance_entry_count = segment_products
            .iter()
            .map(|product| product.native_glyph_dependencies.instance_count())
            .sum();
        let native_glyph_dependencies = NativeBitmapAtlasFrameDependencyIndex::from_segment_indexes(
            segment_products
                .iter()
                .map(|product| &product.native_glyph_dependencies),
        );
        let active_native_glyph_dependency_count = native_glyph_dependencies.dependency_count();
        let native_reverse_segment_entry_count = native_glyph_dependencies.segment_entry_count();
        let run_index = ScreenSpaceUiTextFrameRunIndex::from_segment_run_counts(
            segment_products.iter().map(|product| {
                [
                    product.resolved_texts.native_texts().len(),
                    product.resolved_texts.sdf_texts().len(),
                ]
            }),
        );
        let generation = ScreenSpaceUiTextFrameProductGeneration::next(
            &mut self.frame_product_generation_counter,
        );
        let product = Arc::new(ScreenSpaceUiTextFrameProduct {
            generation,
            segment_products: Arc::from(segment_products),
            resolved_report,
            native_font_ids,
            input_batch_counts,
            native_glyph_dependencies,
            native_reverse_instance_entry_count,
            run_index,
        });
        self.frame_segments = render_segments.iter().map(Arc::downgrade).collect();
        self.frame_viewport_size = viewport_size;
        self.frame_font_revision = Some(font_revision);
        self.frame_product = Some(Arc::clone(&product));
        record_segment_cache_profile(
            false,
            segment_product_reuse_count,
            text_batch_visit_count,
            glyph_projection_count,
            active_native_glyph_dependency_count,
            native_reverse_instance_entry_count,
            native_reverse_segment_entry_count,
            product.run_index.spans().len(),
            product.run_index.native_run_count(),
            product.run_index.sdf_run_count(),
            compatibility_batch_clone_count,
            compatibility_glyph_run_clone_count,
        );
        product
    }

    pub(super) fn invalidate_frame_product(&mut self) {
        self.segment_product_entries.clear();
        self.frame_segments.clear();
        self.frame_product = None;
    }

    fn frame_matches(
        &self,
        render_segments: &[Arc<PlannedScreenSpaceUi>],
        viewport_size: UVec2,
        font_revision: FontCollectionRevision,
    ) -> bool {
        self.frame_product.is_some()
            && self.frame_viewport_size == viewport_size
            && self.frame_font_revision == Some(font_revision)
            && self.frame_segments.len() == render_segments.len()
            && self
                .frame_segments
                .iter()
                .zip(render_segments)
                .all(|(current, next)| segment_plan_reused(Some(current), next))
    }
}

impl ScreenSpaceUiTextFrameProduct {
    pub(super) fn generation(&self) -> ScreenSpaceUiTextFrameProductGeneration {
        self.generation
    }

    pub(super) fn segment_products(&self) -> &[Arc<ScreenSpaceUiTextSegmentProduct>] {
        &self.segment_products
    }

    pub(super) fn native_text_segments(
        &self,
    ) -> impl Clone + Iterator<Item = &[ScreenSpaceUiTextBatch]> {
        self.segment_products
            .iter()
            .map(|product| product.native_texts())
    }

    pub(super) fn sdf_text_segments(
        &self,
    ) -> impl Clone + Iterator<Item = &[ScreenSpaceUiTextBatch]> {
        self.segment_products
            .iter()
            .map(|product| product.sdf_texts())
    }

    pub(super) fn resolved_report(&self) -> ScreenSpaceUiResolvedTextReport {
        self.resolved_report
    }

    pub(super) fn materialize_resolved_texts(&self) -> ResolvedScreenSpaceUiTextBatches {
        let mut resolved = ResolvedScreenSpaceUiTextBatches::default();
        for product in self.segment_products.iter() {
            resolved.append_segment_cloned(&product.resolved_texts);
        }
        resolved
    }

    pub(super) fn native_font_ids(&self) -> ScreenSpaceUiTextFontIdReport {
        self.native_font_ids
    }

    pub(super) fn input_batch_counts(&self) -> [usize; 3] {
        self.input_batch_counts
    }

    pub(super) fn native_run_count(&self) -> usize {
        self.run_index.native_run_count()
    }

    pub(super) fn sdf_run_count(&self) -> usize {
        self.run_index.sdf_run_count()
    }
}

impl ScreenSpaceUiTextSegmentProduct {
    fn native_texts(&self) -> &[ScreenSpaceUiTextBatch] {
        self.resolved_texts.native_texts()
    }

    fn sdf_texts(&self) -> &[ScreenSpaceUiTextBatch] {
        self.resolved_texts.sdf_texts()
    }

    pub(super) fn native_glyph_runs(&self) -> &[NativeBitmapAtlasGlyphRun] {
        &self.native_glyph_runs.glyph_runs
    }
}

fn accumulate_font_id_report(
    total: &mut ScreenSpaceUiTextFontIdReport,
    segment: ScreenSpaceUiTextFontIdReport,
) {
    total.text_batch_count = total
        .text_batch_count
        .saturating_add(segment.text_batch_count);
    total.glyph_count = total.glyph_count.saturating_add(segment.glyph_count);
    total.fallback_glyph_count = total
        .fallback_glyph_count
        .saturating_add(segment.fallback_glyph_count);
    total.unmapped_glyph_count = total
        .unmapped_glyph_count
        .saturating_add(segment.unmapped_glyph_count);
}

fn build_segment_product(
    plan: &Arc<PlannedScreenSpaceUi>,
    viewport_size: UVec2,
    font_assets: &UiFontAssetCache,
    auto_router: &mut AutoTextRasterRouter,
    shaping_changed: bool,
    font_revision: FontCollectionRevision,
    font_collection: &Arc<FontCollectionService>,
) -> ScreenSpaceUiTextSegmentProduct {
    let auto_texts = plan.auto_text_batches();
    let native_texts = plan.native_text_batches();
    let sdf_texts = plan.sdf_text_batches();
    let input_batch_counts = [auto_texts.len(), native_texts.len(), sdf_texts.len()];
    let resolved_texts = resolve_text_batches_after_font_dependencies(
        font_assets,
        auto_router,
        auto_texts,
        native_texts,
        sdf_texts,
        shaping_changed,
        false,
        font_revision,
        font_collection,
    );
    let native_glyph_runs =
        native_bitmap_atlas_glyph_runs(viewport_size, resolved_texts.native_texts());
    let native_glyph_dependencies =
        NativeBitmapAtlasSegmentDependencyIndex::from_glyph_runs(&native_glyph_runs.glyph_runs);
    let resolved_report = ScreenSpaceUiResolvedTextReport::from_resolved_texts(&resolved_texts);
    ScreenSpaceUiTextSegmentProduct {
        resolved_texts,
        resolved_report,
        native_glyph_runs,
        native_glyph_dependencies,
        input_batch_counts,
        auto_routes: Arc::from(
            auto_texts
                .iter()
                .map(|text| text.route_identity.clone())
                .collect::<Vec<_>>(),
        ),
    }
}

fn segment_product_entry_reused(
    current: &ScreenSpaceUiTextSegmentProductEntry,
    next: &Arc<PlannedScreenSpaceUi>,
    viewport_size: UVec2,
    font_revision: FontCollectionRevision,
) -> bool {
    current.viewport_size == viewport_size
        && current.font_revision == font_revision
        && segment_plan_reused(Some(&current.plan), next)
}

fn collect_segment_font_dependencies(plan: &Arc<PlannedScreenSpaceUi>) -> Arc<[Arc<str>]> {
    let mut seen = HashSet::new();
    let mut assets = Vec::new();
    for text in plan.text_batches() {
        let asset = text
            .font
            .as_deref()
            .filter(|asset| !asset.trim().is_empty())
            .unwrap_or(super::DEFAULT_FONT_ASSET);
        push_font_dependency(asset, &mut seen, &mut assets);
        if text.style.code {
            push_font_dependency(super::DEFAULT_FONT_ASSET, &mut seen, &mut assets);
        }
    }
    Arc::from(assets)
}

fn push_font_dependency<'a>(
    asset: &'a str,
    seen: &mut HashSet<&'a str>,
    assets: &mut Vec<Arc<str>>,
) {
    if seen.insert(asset) {
        assets.push(Arc::from(asset));
    }
}

pub(super) fn segment_plan_reused(
    current: Option<&Weak<PlannedScreenSpaceUi>>,
    next: &Arc<PlannedScreenSpaceUi>,
) -> bool {
    current
        .and_then(Weak::upgrade)
        .is_some_and(|current| Arc::ptr_eq(&current, next))
}

fn record_segment_cache_profile(
    segment_plan_reused: bool,
    segment_product_reuse_count: usize,
    text_batch_visit_count: usize,
    glyph_projection_count: usize,
    active_native_glyph_dependency_count: usize,
    native_reverse_instance_entry_count: usize,
    native_reverse_segment_entry_count: usize,
    run_index_segment_count: usize,
    native_run_index_run_count: usize,
    sdf_run_index_run_count: usize,
    compatibility_batch_clone_count: usize,
    compatibility_glyph_run_clone_count: usize,
) {
    crate::core::diagnostics::profiling::record_counter_batch(
        "runtime",
        &[
            (
                "ui_text.segment_cache.frame_product_reuse_count",
                usize::from(segment_plan_reused) as f64,
            ),
            (
                "ui_text.segment_cache.segment_product_reuse_count",
                segment_product_reuse_count as f64,
            ),
            (
                "ui_text.segment_cache.text_batch_visit_count",
                text_batch_visit_count as f64,
            ),
            (
                "ui_text.segment_cache.glyph_projection_count",
                glyph_projection_count as f64,
            ),
            (
                "ui_text.segment_cache.active_native_glyph_dependency_count",
                active_native_glyph_dependency_count as f64,
            ),
            (
                "ui_text.segment_cache.native_reverse_instance_entry_count",
                native_reverse_instance_entry_count as f64,
            ),
            (
                "ui_text.segment_cache.native_reverse_segment_entry_count",
                native_reverse_segment_entry_count as f64,
            ),
            (
                "ui_text.segment_cache.run_index_segment_count",
                run_index_segment_count as f64,
            ),
            (
                "ui_text.segment_cache.native_run_index_run_count",
                native_run_index_run_count as f64,
            ),
            (
                "ui_text.segment_cache.sdf_run_index_run_count",
                sdf_run_index_run_count as f64,
            ),
            (
                "ui_text.segment_cache.compatibility_batch_clone_count",
                compatibility_batch_clone_count as f64,
            ),
            (
                "ui_text.segment_cache.compatibility_glyph_run_clone_count",
                compatibility_glyph_run_clone_count as f64,
            ),
        ],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_font_revision(generation: u64) -> FontCollectionRevision {
        FontCollectionRevision::new(
            crate::text::font::shared_font_collection_handle(),
            generation,
        )
    }

    #[test]
    fn screen_space_ui_text_segment_plan_reuses_only_exact_arc_identity() {
        let plan = Arc::new(PlannedScreenSpaceUi::default());
        let same = Arc::clone(&plan);
        let replacement = Arc::new(PlannedScreenSpaceUi::default());
        let current = Arc::downgrade(&plan);

        assert!(segment_plan_reused(Some(&current), &same));
        assert!(!segment_plan_reused(Some(&current), &replacement));
        assert!(!segment_plan_reused(None, &same));
    }

    #[test]
    fn text_segment_product_reuse_requires_plan_viewport_and_font_revision_identity() {
        let plan = Arc::new(PlannedScreenSpaceUi::default());
        let revision = test_font_revision(7);
        let entry = ScreenSpaceUiTextSegmentProductEntry {
            plan: Arc::downgrade(&plan),
            viewport_size: UVec2::new(800, 600),
            font_revision: revision,
            product: Arc::new(ScreenSpaceUiTextSegmentProduct {
                resolved_texts: Default::default(),
                resolved_report: Default::default(),
                native_glyph_runs: Default::default(),
                native_glyph_dependencies: Default::default(),
                input_batch_counts: [0; 3],
                auto_routes: Arc::from([]),
            }),
        };

        assert!(segment_product_entry_reused(
            &entry,
            &plan,
            UVec2::new(800, 600),
            revision,
        ));
        assert!(!segment_product_entry_reused(
            &entry,
            &plan,
            UVec2::new(801, 600),
            revision,
        ));
        assert!(!segment_product_entry_reused(
            &entry,
            &plan,
            UVec2::new(800, 600),
            test_font_revision(8),
        ));
        assert!(!segment_product_entry_reused(
            &entry,
            &Arc::new(PlannedScreenSpaceUi::default()),
            UVec2::new(800, 600),
            revision,
        ));
    }

    #[test]
    fn text_segment_product_reuse_rejects_foreign_collection_at_same_generation() {
        let plan = Arc::new(PlannedScreenSpaceUi::default());
        let first_collection = crate::text::font::FontCollectionService::from_database(
            crate::text::font::runtime_default_font_database_for_test(),
        );
        let foreign_collection = crate::text::font::FontCollectionService::from_database(
            crate::text::font::runtime_default_font_database_for_test(),
        );
        assert_eq!(
            first_collection.generation(),
            foreign_collection.generation()
        );
        let revision = first_collection.revision();
        let entry = ScreenSpaceUiTextSegmentProductEntry {
            plan: Arc::downgrade(&plan),
            viewport_size: UVec2::new(800, 600),
            font_revision: revision,
            product: Arc::new(ScreenSpaceUiTextSegmentProduct {
                resolved_texts: Default::default(),
                resolved_report: Default::default(),
                native_glyph_runs: Default::default(),
                native_glyph_dependencies: Default::default(),
                input_batch_counts: [0; 3],
                auto_routes: Arc::from([]),
            }),
        };

        assert!(!segment_product_entry_reused(
            &entry,
            &plan,
            UVec2::new(800, 600),
            foreign_collection.revision(),
        ));
    }

    #[test]
    fn text_frame_product_generation_advances_only_when_a_product_is_published() {
        let mut counter = 0;
        let first = ScreenSpaceUiTextFrameProductGeneration::next(&mut counter);
        let second = ScreenSpaceUiTextFrameProductGeneration::next(&mut counter);

        assert_ne!(first, second);
        assert_eq!(counter, 2);
    }
}
