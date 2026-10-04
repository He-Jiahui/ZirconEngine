use std::collections::{HashMap, HashSet};
use std::sync::Arc;

mod frame_journal;
mod native_dependency_index;

use crate::core::math::UVec2;

use self::frame_journal::ScreenSpaceUiTextFrameJournalInputs;
pub(in crate::graphics::scene::scene_renderer::ui) use self::frame_journal::{
    ScreenSpaceUiTextFrameChangeJournal, ScreenSpaceUiTextFrameFullRebuildReason,
};
use self::native_dependency_index::NativeBitmapAtlasSegmentDependencyIndex;
use super::font_assets::UiFontAssetCache;
use super::font_id_report::ScreenSpaceUiTextFontIdReport;
use super::native_bitmap_atlas_glyph_runs;
use super::native_glyph_run::NativeBitmapAtlasGlyphRunProjection;
use super::prepare_report::ScreenSpaceUiResolvedTextReport;
use super::resolved_batches::{
    resolve_text_batches_after_font_dependencies, AutoTextRasterRouter,
    ResolvedScreenSpaceUiTextBatches,
};
use super::ScreenSpaceUiTextFrameProductGeneration;
use crate::graphics::scene::scene_renderer::ui::render::{
    PlannedScreenSpaceUi, PreparedScreenSpaceUi, ScreenSpaceUiTextBatch,
    ScreenSpaceUiTextRouteIdentity,
};
use crate::text::atlas::GlyphRasterKey;
use crate::text::font::{FontCollectionRevision, FontCollectionService};
use crate::text::native_bitmap_atlas::NativeBitmapAtlasGlyphRun;

#[derive(Default)]
pub(super) struct ScreenSpaceUiTextSegmentCache {
    font_dependency_entries: Vec<ScreenSpaceUiTextFontDependencyEntry>,
    active_font_dependencies: Vec<Arc<str>>,
    font_dependency_ref_counts: HashMap<Arc<str>, usize>,
    font_dependency_generation: Option<u64>,
    segment_product_entries: Vec<ScreenSpaceUiTextSegmentProductEntry>,
    frame_product_generation_counter: u64,
    frame_aggregate_index: ScreenSpaceUiTextFrameAggregateIndex,
    native_glyph_dependency_ref_counts: HashMap<GlyphRasterKey, usize>,
    native_reverse_instance_entry_count: usize,
    native_reverse_segment_entry_count: usize,
    frame_generation: Option<u64>,
    frame_viewport_size: UVec2,
    frame_font_revision: Option<FontCollectionRevision>,
    frame_product: Option<Arc<ScreenSpaceUiTextFrameProduct>>,
    pending_full_rebuild_reason: Option<ScreenSpaceUiTextFrameFullRebuildReason>,
}

struct ScreenSpaceUiTextFontDependencyEntry {
    assets: Arc<[Arc<str>]>,
}

struct ScreenSpaceUiTextSegmentProductEntry {
    plan: Arc<PlannedScreenSpaceUi>,
    viewport_size: UVec2,
    font_revision: FontCollectionRevision,
    product: Arc<ScreenSpaceUiTextSegmentProduct>,
}

pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiTextSegmentProduct {
    resolved_texts: ResolvedScreenSpaceUiTextBatches,
    resolved_report: ScreenSpaceUiResolvedTextReport,
    native_glyph_runs: NativeBitmapAtlasGlyphRunProjection,
    native_glyph_dependencies: NativeBitmapAtlasSegmentDependencyIndex,
    native_glyph_dependency_keys: Arc<[GlyphRasterKey]>,
    input_batch_counts: [usize; 3],
    auto_routes: Arc<[ScreenSpaceUiTextRouteIdentity]>,
}

pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiTextFrameProduct {
    generation: ScreenSpaceUiTextFrameProductGeneration,
    change_journal: ScreenSpaceUiTextFrameChangeJournal,
    segment_products: Arc<[Arc<ScreenSpaceUiTextSegmentProduct>]>,
    resolved_report: ScreenSpaceUiResolvedTextReport,
    native_font_ids: ScreenSpaceUiTextFontIdReport,
    input_batch_counts: [usize; 3],
    active_native_glyph_dependency_count: usize,
    native_reverse_instance_entry_count: usize,
    native_reverse_segment_entry_count: usize,
    native_run_count: usize,
    sdf_run_count: usize,
}

#[derive(Clone, Copy, Debug, Default)]
struct ScreenSpaceUiTextFrameAggregate {
    resolved_report: ScreenSpaceUiResolvedTextReport,
    native_font_ids: ScreenSpaceUiTextFontIdReport,
    input_batch_counts: [usize; 3],
    native_run_count: usize,
    sdf_run_count: usize,
}

#[derive(Default)]
struct ScreenSpaceUiTextFrameAggregateIndex {
    leaf_count: usize,
    leaf_base: usize,
    nodes: Vec<ScreenSpaceUiTextFrameAggregate>,
}

impl ScreenSpaceUiTextFrameAggregate {
    fn from_product(product: &ScreenSpaceUiTextSegmentProduct) -> Self {
        Self {
            resolved_report: product.resolved_report,
            native_font_ids: product.native_glyph_runs.font_ids,
            input_batch_counts: product.input_batch_counts,
            native_run_count: product.resolved_texts.native_texts().len(),
            sdf_run_count: product.resolved_texts.sdf_texts().len(),
        }
    }

    fn merge(mut self, next: Self) -> Self {
        self.resolved_report.merge(next.resolved_report);
        accumulate_font_id_report(&mut self.native_font_ids, next.native_font_ids);
        for (total, count) in self
            .input_batch_counts
            .iter_mut()
            .zip(next.input_batch_counts)
        {
            *total = total.saturating_add(count);
        }
        self.native_run_count = self.native_run_count.saturating_add(next.native_run_count);
        self.sdf_run_count = self.sdf_run_count.saturating_add(next.sdf_run_count);
        self
    }
}

impl ScreenSpaceUiTextFrameAggregateIndex {
    fn rebuild(&mut self, entries: &[ScreenSpaceUiTextSegmentProductEntry]) {
        self.leaf_count = entries.len();
        self.leaf_base = entries.len().next_power_of_two().max(1);
        self.nodes = vec![ScreenSpaceUiTextFrameAggregate::default(); self.leaf_base * 2];
        for (index, entry) in entries.iter().enumerate() {
            self.nodes[self.leaf_base + index] =
                ScreenSpaceUiTextFrameAggregate::from_product(&entry.product);
        }
        for index in (1..self.leaf_base).rev() {
            self.nodes[index] = self.nodes[index * 2].merge(self.nodes[index * 2 + 1]);
        }
    }

    fn patch(&mut self, index: usize, product: &ScreenSpaceUiTextSegmentProduct) -> bool {
        if index >= self.leaf_count || self.nodes.len() != self.leaf_base.saturating_mul(2) {
            return false;
        }
        let mut node_index = self.leaf_base + index;
        self.nodes[node_index] = ScreenSpaceUiTextFrameAggregate::from_product(product);
        while node_index > 1 {
            node_index /= 2;
            self.nodes[node_index] =
                self.nodes[node_index * 2].merge(self.nodes[node_index * 2 + 1]);
        }
        true
    }

    fn root(&self) -> ScreenSpaceUiTextFrameAggregate {
        self.nodes.get(1).copied().unwrap_or_default()
    }

    fn leaf_count(&self) -> usize {
        self.leaf_count
    }
}

impl ScreenSpaceUiTextSegmentCache {
    pub(super) fn refresh_font_dependencies(&mut self, prepared: &PreparedScreenSpaceUi) {
        if self.font_dependency_generation == Some(prepared.generation()) {
            return;
        }
        let render_segments = prepared.render_segments();
        let journal = prepared.change_journal();
        let can_patch = !journal.is_full_rebuild()
            && journal.base_generation() == self.font_dependency_generation;
        if !can_patch {
            self.rebuild_font_dependencies(render_segments);
            self.font_dependency_generation = Some(prepared.generation());
            return;
        }

        while self.font_dependency_entries.len() > render_segments.len() {
            let entry = self
                .font_dependency_entries
                .pop()
                .expect("font dependency entry length must be non-zero while truncating");
            self.remove_font_dependency_assets(&entry.assets);
        }
        for &index in journal.changed_segment_indices() {
            let Some(plan) = render_segments.get(index) else {
                self.rebuild_font_dependencies(render_segments);
                self.font_dependency_generation = Some(prepared.generation());
                return;
            };
            let next = ScreenSpaceUiTextFontDependencyEntry {
                assets: collect_segment_font_dependencies(plan),
            };
            if index < self.font_dependency_entries.len() {
                let previous = std::mem::replace(&mut self.font_dependency_entries[index], next);
                self.remove_font_dependency_assets(&previous.assets);
            } else if index == self.font_dependency_entries.len() {
                self.font_dependency_entries.push(next);
            } else {
                self.rebuild_font_dependencies(render_segments);
                self.font_dependency_generation = Some(prepared.generation());
                return;
            }
            let assets = Arc::clone(&self.font_dependency_entries[index].assets);
            self.add_font_dependency_assets(&assets);
        }
        if self.font_dependency_entries.len() != render_segments.len() {
            self.rebuild_font_dependencies(render_segments);
        }
        self.font_dependency_generation = Some(prepared.generation());
    }

    fn rebuild_font_dependencies(&mut self, render_segments: &[Arc<PlannedScreenSpaceUi>]) {
        self.font_dependency_entries.clear();
        self.active_font_dependencies.clear();
        self.font_dependency_ref_counts.clear();
        self.font_dependency_entries = render_segments
            .iter()
            .map(|plan| ScreenSpaceUiTextFontDependencyEntry {
                assets: collect_segment_font_dependencies(plan),
            })
            .collect();
        let assets = self
            .font_dependency_entries
            .iter()
            .flat_map(|entry| entry.assets.iter().cloned())
            .collect::<Vec<_>>();
        for asset in &assets {
            self.add_font_dependency_asset(asset);
        }
    }

    fn add_font_dependency_assets(&mut self, assets: &[Arc<str>]) {
        for asset in assets {
            self.add_font_dependency_asset(asset);
        }
    }

    fn add_font_dependency_asset(&mut self, asset: &Arc<str>) {
        let count = self
            .font_dependency_ref_counts
            .entry(Arc::clone(asset))
            .or_default();
        if *count == 0 {
            self.active_font_dependencies.push(Arc::clone(asset));
        }
        *count = count.saturating_add(1);
    }

    fn remove_font_dependency_assets(&mut self, assets: &[Arc<str>]) {
        for asset in assets {
            let Some(count) = self.font_dependency_ref_counts.get_mut(asset) else {
                continue;
            };
            *count = count.saturating_sub(1);
            if *count > 0 {
                continue;
            }
            self.font_dependency_ref_counts.remove(asset);
            self.active_font_dependencies
                .retain(|current| current.as_ref() != asset.as_ref());
        }
    }

    pub(super) fn active_font_dependencies(&self) -> &[Arc<str>] {
        &self.active_font_dependencies
    }

    pub(super) fn prepare_frame_product(
        &mut self,
        prepared: &PreparedScreenSpaceUi,
        viewport_size: UVec2,
        font_revision: FontCollectionRevision,
        font_assets: &UiFontAssetCache,
        auto_router: &mut AutoTextRasterRouter,
        shaping_changed: bool,
        font_collection: &Arc<FontCollectionService>,
    ) -> Arc<ScreenSpaceUiTextFrameProduct> {
        let render_segments = prepared.render_segments();
        if self.frame_matches(prepared.generation(), viewport_size, font_revision) {
            let product = self
                .frame_product
                .as_ref()
                .expect("matching text frame cache key must retain its product");
            record_segment_cache_profile(
                true,
                render_segments.len(),
                0,
                0,
                product.active_native_glyph_dependency_count,
                product.native_reverse_instance_entry_count,
                product.native_reverse_segment_entry_count,
                product.segment_products.len(),
                product.native_run_count,
                product.sdf_run_count,
                0,
                0,
            );
            return Arc::clone(product);
        }

        let mut segment_product_reuse_count = 0_usize;
        let mut text_batch_visit_count = 0_usize;
        let mut glyph_projection_count = 0_usize;
        let journal = prepared.change_journal();
        let previous_product_generation = self
            .frame_product
            .as_ref()
            .map(|product| product.generation());
        let previous_source_generation = self.frame_generation;
        let previous_segment_count = self
            .frame_product
            .as_ref()
            .map_or(0, |product| product.segment_products.len());
        let previous_viewport_size = self.frame_viewport_size;
        let previous_font_revision = self.frame_font_revision;
        let retained_state_consistent = self.segment_product_entries.len()
            == previous_segment_count
            && self.frame_aggregate_index.leaf_count() == previous_segment_count;
        let pending_full_rebuild_reason = self.pending_full_rebuild_reason.take();
        let can_patch = !journal.is_full_rebuild()
            && journal.base_generation() == self.frame_generation
            && journal.appended_segment_count() == 0
            && journal.truncated_segment_count() == 0
            && self.frame_viewport_size == viewport_size
            && self.frame_font_revision == Some(font_revision)
            && self.frame_product.is_some()
            && self.segment_product_entries.len() == render_segments.len()
            && self.frame_aggregate_index.leaf_count() == render_segments.len()
            && journal
                .changed_segment_indices()
                .iter()
                .all(|&index| index < render_segments.len());
        let mut active_routes_changed = false;
        let mut text_products_changed = false;
        let mut changed_text_segment_indices = Vec::new();
        if can_patch {
            segment_product_reuse_count = render_segments
                .len()
                .saturating_sub(journal.changed_segment_indices().len());
            for &index in journal.changed_segment_indices() {
                let plan = &render_segments[index];
                if segment_product_entry_reused(
                    &self.segment_product_entries[index],
                    plan,
                    viewport_size,
                    font_revision,
                ) {
                    self.segment_product_entries[index].plan = Arc::clone(plan);
                    segment_product_reuse_count = segment_product_reuse_count.saturating_add(1);
                    continue;
                }
                text_products_changed = true;
                changed_text_segment_indices.push(index);
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
                let previous_product = Arc::clone(&self.segment_product_entries[index].product);
                active_routes_changed |=
                    previous_product.auto_routes.as_ref() != product.auto_routes.as_ref();
                remove_native_glyph_aggregate(
                    &mut self.native_glyph_dependency_ref_counts,
                    &mut self.native_reverse_instance_entry_count,
                    &mut self.native_reverse_segment_entry_count,
                    &previous_product,
                );
                add_native_glyph_aggregate(
                    &mut self.native_glyph_dependency_ref_counts,
                    &mut self.native_reverse_instance_entry_count,
                    &mut self.native_reverse_segment_entry_count,
                    &product,
                );
                self.segment_product_entries[index] = ScreenSpaceUiTextSegmentProductEntry {
                    plan: Arc::clone(plan),
                    viewport_size,
                    font_revision,
                    product: Arc::clone(&product),
                };
                let patched = self.frame_aggregate_index.patch(index, &product);
                debug_assert!(
                    patched,
                    "validated text aggregate patch must remain in range"
                );
            }
        } else {
            let previous_entries = std::mem::take(&mut self.segment_product_entries);
            let mut previous_entries = previous_entries.into_iter();
            let mut next_entries = Vec::with_capacity(render_segments.len());
            for (index, plan) in render_segments.iter().enumerate() {
                let previous = previous_entries.next();
                if let Some(mut previous) = previous.filter(|entry| {
                    segment_product_entry_reused(entry, plan, viewport_size, font_revision)
                }) {
                    previous.plan = Arc::clone(plan);
                    segment_product_reuse_count = segment_product_reuse_count.saturating_add(1);
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
                if index < previous_segment_count {
                    changed_text_segment_indices.push(index);
                }
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
                    plan: Arc::clone(plan),
                    viewport_size,
                    font_revision,
                    product,
                });
            }
            self.segment_product_entries = next_entries;
            self.frame_aggregate_index
                .rebuild(&self.segment_product_entries);
            self.native_glyph_dependency_ref_counts.clear();
            self.native_reverse_instance_entry_count = 0;
            self.native_reverse_segment_entry_count = 0;
            for entry in &self.segment_product_entries {
                add_native_glyph_aggregate(
                    &mut self.native_glyph_dependency_ref_counts,
                    &mut self.native_reverse_instance_entry_count,
                    &mut self.native_reverse_segment_entry_count,
                    &entry.product,
                );
            }
            active_routes_changed = true;
            text_products_changed = true;
        }
        if active_routes_changed {
            auto_router.replace_active_routes(
                self.segment_product_entries
                    .iter()
                    .flat_map(|entry| entry.product.auto_routes.iter().cloned()),
            );
        }

        if can_patch && !text_products_changed {
            let product = self.reuse_frame_product_for_source_generation(prepared.generation());
            record_segment_cache_profile(
                true,
                segment_product_reuse_count,
                text_batch_visit_count,
                glyph_projection_count,
                product.active_native_glyph_dependency_count,
                product.native_reverse_instance_entry_count,
                product.native_reverse_segment_entry_count,
                product.segment_products.len(),
                product.native_run_count,
                product.sdf_run_count,
                0,
                0,
            );
            return product;
        }

        let aggregate = self.frame_aggregate_index.root();
        let compatibility_batch_clone_count = 0;
        let compatibility_glyph_run_clone_count = 0;
        let generation = ScreenSpaceUiTextFrameProductGeneration::next(
            &mut self.frame_product_generation_counter,
        );
        let change_journal =
            ScreenSpaceUiTextFrameChangeJournal::publish(ScreenSpaceUiTextFrameJournalInputs {
                source: journal,
                previous_product_generation,
                previous_source_generation,
                previous_segment_count,
                previous_viewport_size,
                previous_font_revision,
                retained_state_consistent,
                current_generation: generation,
                current_segment_count: render_segments.len(),
                current_viewport_size: viewport_size,
                current_font_revision: font_revision,
                changed_text_segment_indices: &changed_text_segment_indices,
                pending_full_rebuild_reason,
            });
        let product = Arc::new(ScreenSpaceUiTextFrameProduct {
            generation,
            change_journal,
            segment_products: Arc::from(
                self.segment_product_entries
                    .iter()
                    .map(|entry| Arc::clone(&entry.product))
                    .collect::<Vec<_>>(),
            ),
            resolved_report: aggregate.resolved_report,
            native_font_ids: aggregate.native_font_ids,
            input_batch_counts: aggregate.input_batch_counts,
            active_native_glyph_dependency_count: self.native_glyph_dependency_ref_counts.len(),
            native_reverse_instance_entry_count: self.native_reverse_instance_entry_count,
            native_reverse_segment_entry_count: self.native_reverse_segment_entry_count,
            native_run_count: aggregate.native_run_count,
            sdf_run_count: aggregate.sdf_run_count,
        });
        self.frame_generation = Some(prepared.generation());
        self.frame_viewport_size = viewport_size;
        self.frame_font_revision = Some(font_revision);
        self.frame_product = Some(Arc::clone(&product));
        record_segment_cache_profile(
            false,
            segment_product_reuse_count,
            text_batch_visit_count,
            glyph_projection_count,
            product.active_native_glyph_dependency_count,
            product.native_reverse_instance_entry_count,
            product.native_reverse_segment_entry_count,
            product.segment_products.len(),
            product.native_run_count,
            product.sdf_run_count,
            compatibility_batch_clone_count,
            compatibility_glyph_run_clone_count,
        );
        product
    }

    pub(super) fn invalidate_frame_product(
        &mut self,
        reason: ScreenSpaceUiTextFrameFullRebuildReason,
    ) {
        self.segment_product_entries.clear();
        self.frame_aggregate_index = ScreenSpaceUiTextFrameAggregateIndex::default();
        self.native_glyph_dependency_ref_counts.clear();
        self.native_reverse_instance_entry_count = 0;
        self.native_reverse_segment_entry_count = 0;
        self.frame_generation = None;
        self.frame_product = None;
        self.pending_full_rebuild_reason = Some(reason);
    }

    fn reuse_frame_product_for_source_generation(
        &mut self,
        source_generation: u64,
    ) -> Arc<ScreenSpaceUiTextFrameProduct> {
        self.frame_generation = Some(source_generation);
        Arc::clone(
            self.frame_product
                .as_ref()
                .expect("a text-stable delta must retain its frame product"),
        )
    }

    fn frame_matches(
        &self,
        generation: u64,
        viewport_size: UVec2,
        font_revision: FontCollectionRevision,
    ) -> bool {
        self.frame_product.is_some()
            && self.frame_generation == Some(generation)
            && self.frame_viewport_size == viewport_size
            && self.frame_font_revision == Some(font_revision)
    }
}

impl ScreenSpaceUiTextFrameProduct {
    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::ui) fn for_test_sdf_segments(
        generation: ScreenSpaceUiTextFrameProductGeneration,
        change_journal: ScreenSpaceUiTextFrameChangeJournal,
        segments: Vec<Vec<ScreenSpaceUiTextBatch>>,
    ) -> Self {
        let segment_products = segments
            .into_iter()
            .map(|sdf_texts| {
                Arc::new(ScreenSpaceUiTextSegmentProduct {
                    resolved_texts: ResolvedScreenSpaceUiTextBatches::from_explicit_batches(
                        &[],
                        &sdf_texts,
                    ),
                    resolved_report: Default::default(),
                    native_glyph_runs: Default::default(),
                    native_glyph_dependencies: Default::default(),
                    native_glyph_dependency_keys: Arc::from([]),
                    input_batch_counts: [0, 0, sdf_texts.len()],
                    auto_routes: Arc::from([]),
                })
            })
            .collect::<Vec<_>>();
        let sdf_run_count = segment_products
            .iter()
            .map(|product| product.sdf_texts().len())
            .sum();
        Self {
            generation,
            change_journal,
            segment_products: Arc::from(segment_products),
            resolved_report: Default::default(),
            native_font_ids: Default::default(),
            input_batch_counts: [0, 0, sdf_run_count],
            active_native_glyph_dependency_count: 0,
            native_reverse_instance_entry_count: 0,
            native_reverse_segment_entry_count: 0,
            native_run_count: 0,
            sdf_run_count,
        }
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn generation(
        &self,
    ) -> ScreenSpaceUiTextFrameProductGeneration {
        self.generation
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn change_journal(
        &self,
    ) -> &ScreenSpaceUiTextFrameChangeJournal {
        &self.change_journal
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn segment_products(
        &self,
    ) -> &[Arc<ScreenSpaceUiTextSegmentProduct>] {
        &self.segment_products
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn native_text_segments(
        &self,
    ) -> impl Clone + Iterator<Item = &[ScreenSpaceUiTextBatch]> {
        self.segment_products
            .iter()
            .map(|product| product.native_texts())
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn sdf_text_segments(
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
        self.native_run_count
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn sdf_run_count(&self) -> usize {
        self.sdf_run_count
    }
}

impl ScreenSpaceUiTextSegmentProduct {
    pub(in crate::graphics::scene::scene_renderer::ui) fn native_texts(
        &self,
    ) -> &[ScreenSpaceUiTextBatch] {
        self.resolved_texts.native_texts()
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn sdf_texts(
        &self,
    ) -> &[ScreenSpaceUiTextBatch] {
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
    let native_glyph_dependency_keys =
        collect_native_glyph_dependency_keys(&native_glyph_runs.glyph_runs);
    let resolved_report = ScreenSpaceUiResolvedTextReport::from_resolved_texts(&resolved_texts);
    ScreenSpaceUiTextSegmentProduct {
        resolved_texts,
        resolved_report,
        native_glyph_runs,
        native_glyph_dependencies,
        native_glyph_dependency_keys,
        input_batch_counts,
        auto_routes: Arc::from(
            auto_texts
                .iter()
                .map(|text| text.route_identity.clone())
                .collect::<Vec<_>>(),
        ),
    }
}

fn collect_native_glyph_dependency_keys(
    glyph_runs: &[NativeBitmapAtlasGlyphRun],
) -> Arc<[GlyphRasterKey]> {
    let mut seen = HashSet::new();
    let mut keys = Vec::new();
    for glyph in glyph_runs.iter().flat_map(|run| &run.glyphs) {
        if seen.insert(glyph.raster_key) {
            keys.push(glyph.raster_key);
        }
    }
    Arc::from(keys)
}

fn add_native_glyph_aggregate(
    dependency_ref_counts: &mut HashMap<GlyphRasterKey, usize>,
    reverse_instance_entry_count: &mut usize,
    reverse_segment_entry_count: &mut usize,
    product: &ScreenSpaceUiTextSegmentProduct,
) {
    for &key in product.native_glyph_dependency_keys.iter() {
        let count = dependency_ref_counts.entry(key).or_default();
        *count = count.saturating_add(1);
    }
    *reverse_instance_entry_count = reverse_instance_entry_count
        .saturating_add(product.native_glyph_dependencies.instance_count());
    *reverse_segment_entry_count =
        reverse_segment_entry_count.saturating_add(product.native_glyph_dependency_keys.len());
}

fn remove_native_glyph_aggregate(
    dependency_ref_counts: &mut HashMap<GlyphRasterKey, usize>,
    reverse_instance_entry_count: &mut usize,
    reverse_segment_entry_count: &mut usize,
    product: &ScreenSpaceUiTextSegmentProduct,
) {
    for &key in product.native_glyph_dependency_keys.iter() {
        let Some(count) = dependency_ref_counts.get_mut(&key) else {
            continue;
        };
        *count = count.saturating_sub(1);
        if *count == 0 {
            dependency_ref_counts.remove(&key);
        }
    }
    *reverse_instance_entry_count = reverse_instance_entry_count
        .saturating_sub(product.native_glyph_dependencies.instance_count());
    *reverse_segment_entry_count =
        reverse_segment_entry_count.saturating_sub(product.native_glyph_dependency_keys.len());
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
    current: Option<&Arc<PlannedScreenSpaceUi>>,
    next: &Arc<PlannedScreenSpaceUi>,
) -> bool {
    current.is_some_and(|current| {
        Arc::ptr_eq(current, next) || current.text_preparation_inputs_match(next)
    })
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
#[path = "tests/segment_cache.rs"]
mod tests;
