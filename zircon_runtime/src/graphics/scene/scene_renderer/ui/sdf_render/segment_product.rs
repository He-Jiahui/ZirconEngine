use std::ops::Range;

use crate::core::math::UVec2;
use crate::text::font::TextDecorationMetrics;
use crate::text::sdf::{SdfAtlasBake, SdfRunCpuPreparation};

use super::super::sdf_atlas::SdfAtlasPlan;
use super::super::text::{
    ScreenSpaceUiTextFrameProduct, ScreenSpaceUiTextFrameProductGeneration,
    ScreenSpaceUiTextSegmentProduct,
};
use super::decorations::build_text_decoration_vertices_iter;
use super::material::{
    SdfTextMaterial, SdfTextMaterialDraw, SdfTextMaterialDrawPlan, SDF_TEXT_EFFECT_GLOW,
    SDF_TEXT_EFFECT_OUTLINE, SDF_TEXT_EFFECT_SHADOW,
};
use super::vertices::{build_sdf_vertex_plan_with_runs_iter, ScreenSpaceUiSdfVertex};

#[derive(Default)]
pub(super) struct SdfCompiledTextSegmentIndex {
    retained_frame_generation: Option<ScreenSpaceUiTextFrameProductGeneration>,
    viewport_size: UVec2,
    sdf_run_ranges: Vec<Range<usize>>,
    native_metric_ranges: Vec<Range<usize>>,
    native_vertex_ranges: Vec<Range<usize>>,
    sdf_decoration_vertex_ranges: Vec<Range<usize>>,
    glyph_vertex_ranges: Vec<Range<usize>>,
    material_ranges: Vec<Range<usize>>,
    draw_ranges: Vec<Range<usize>>,
    decoration_vertex_count: u32,
    effect_batch_counts: [usize; 3],
}

#[derive(Default)]
pub(super) struct SdfCompiledTextPrepareReport {
    pub(super) reused: bool,
    pub(super) full_rebuild: bool,
    pub(super) changed_vertex_ranges: Vec<Range<usize>>,
    pub(super) changed_material_ranges: Vec<Range<usize>>,
    pub(super) segment_visit_count: usize,
    pub(super) vertex_visit_count: usize,
    pub(super) material_visit_count: usize,
}

struct SdfCompiledTextSegmentProduct {
    native_decoration_vertices: Vec<ScreenSpaceUiSdfVertex>,
    sdf_decoration_vertices: Vec<ScreenSpaceUiSdfVertex>,
    glyph_vertices: Vec<ScreenSpaceUiSdfVertex>,
    draw_plan: SdfTextMaterialDrawPlan,
}

impl SdfCompiledTextSegmentIndex {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn prepare(
        &mut self,
        frame: &ScreenSpaceUiTextFrameProduct,
        viewport_size: UVec2,
        sdf_cpu_runs: &[SdfRunCpuPreparation],
        native_decoration_metrics: &[TextDecorationMetrics],
        atlas_plan: &SdfAtlasPlan,
        atlas_bake: &SdfAtlasBake,
        full_rebuild_required: bool,
        vertices: &mut Vec<ScreenSpaceUiSdfVertex>,
        draw_plan: &mut SdfTextMaterialDrawPlan,
    ) -> SdfCompiledTextPrepareReport {
        if !full_rebuild_required
            && self.retained_frame_generation == Some(frame.generation())
            && self.viewport_size == viewport_size
        {
            return SdfCompiledTextPrepareReport {
                reused: true,
                ..Default::default()
            };
        }

        if !full_rebuild_required {
            if let Some(report) = self.patch_retained_frame(
                frame,
                viewport_size,
                sdf_cpu_runs,
                native_decoration_metrics,
                atlas_plan,
                atlas_bake,
                vertices,
                draw_plan,
            ) {
                return report;
            }
        }

        self.rebuild(
            frame,
            viewport_size,
            sdf_cpu_runs,
            native_decoration_metrics,
            atlas_plan,
            atlas_bake,
            vertices,
            draw_plan,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn patch_retained_frame(
        &mut self,
        frame: &ScreenSpaceUiTextFrameProduct,
        viewport_size: UVec2,
        sdf_cpu_runs: &[SdfRunCpuPreparation],
        native_decoration_metrics: &[TextDecorationMetrics],
        atlas_plan: &SdfAtlasPlan,
        atlas_bake: &SdfAtlasBake,
        vertices: &mut Vec<ScreenSpaceUiSdfVertex>,
        draw_plan: &mut SdfTextMaterialDrawPlan,
    ) -> Option<SdfCompiledTextPrepareReport> {
        let journal = frame.change_journal();
        let segment_count = frame.segment_products().len();
        if journal.is_full_rebuild()
            || journal.base_generation() != self.retained_frame_generation
            || journal.current_generation() != frame.generation()
            || journal.appended_segment_count() != 0
            || journal.truncated_segment_count() != 0
            || self.viewport_size != viewport_size
            || !self.directories_match(segment_count, vertices, draw_plan)
            || self
                .sdf_run_ranges
                .last()
                .map_or(!sdf_cpu_runs.is_empty(), |range| {
                    range.end != sdf_cpu_runs.len()
                })
            || self
                .native_metric_ranges
                .last()
                .map_or(!native_decoration_metrics.is_empty(), |range| {
                    range.end != native_decoration_metrics.len()
                })
            || atlas_plan.runs.len() != sdf_cpu_runs.len()
        {
            return None;
        }

        let mut replacements = Vec::with_capacity(journal.changed_segment_indices().len());
        for &index in journal.changed_segment_indices() {
            let segment = frame.segment_products().get(index)?;
            let sdf_range = self.sdf_run_ranges.get(index)?.clone();
            let native_range = self.native_metric_ranges.get(index)?.clone();
            if sdf_range.len() != segment.sdf_texts().len()
                || native_range.len() != segment.native_texts().len()
            {
                return None;
            }
            replacements.push((
                index,
                build_segment_product(
                    segment,
                    &sdf_cpu_runs[sdf_range.clone()],
                    &native_decoration_metrics[native_range],
                    atlas_plan,
                    &atlas_plan.runs[sdf_range],
                    atlas_bake,
                    viewport_size,
                ),
            ));
        }

        let mut report = self.apply_replacements(replacements, vertices, draw_plan)?;
        self.retained_frame_generation = Some(frame.generation());
        report.segment_visit_count = journal.changed_segment_indices().len();
        Some(report)
    }

    #[allow(clippy::too_many_arguments)]
    fn rebuild(
        &mut self,
        frame: &ScreenSpaceUiTextFrameProduct,
        viewport_size: UVec2,
        sdf_cpu_runs: &[SdfRunCpuPreparation],
        native_decoration_metrics: &[TextDecorationMetrics],
        atlas_plan: &SdfAtlasPlan,
        atlas_bake: &SdfAtlasBake,
        vertices: &mut Vec<ScreenSpaceUiSdfVertex>,
        draw_plan: &mut SdfTextMaterialDrawPlan,
    ) -> SdfCompiledTextPrepareReport {
        self.invalidate();
        self.viewport_size = viewport_size;

        let mut products = Vec::with_capacity(frame.segment_products().len());
        let mut sdf_cursor = 0_usize;
        let mut native_cursor = 0_usize;
        for segment in frame.segment_products() {
            let sdf_end = sdf_cursor.saturating_add(segment.sdf_texts().len());
            let native_end = native_cursor.saturating_add(segment.native_texts().len());
            self.sdf_run_ranges.push(sdf_cursor..sdf_end);
            self.native_metric_ranges.push(native_cursor..native_end);
            let sdf_slice_end = sdf_end.min(sdf_cpu_runs.len()).min(atlas_plan.runs.len());
            let native_slice_end = native_end.min(native_decoration_metrics.len());
            products.push(build_segment_product(
                segment,
                sdf_cpu_runs
                    .get(sdf_cursor..sdf_slice_end)
                    .unwrap_or_default(),
                native_decoration_metrics
                    .get(native_cursor..native_slice_end)
                    .unwrap_or_default(),
                atlas_plan,
                atlas_plan
                    .runs
                    .get(sdf_cursor..sdf_slice_end)
                    .unwrap_or_default(),
                atlas_bake,
                viewport_size,
            ));
            sdf_cursor = sdf_end;
            native_cursor = native_end;
        }

        let segment_visit_count = products.len();
        let vertex_visit_count = products.iter().fold(0_usize, |total, product| {
            total
                .saturating_add(product.native_decoration_vertices.len())
                .saturating_add(product.sdf_decoration_vertices.len())
                .saturating_add(product.glyph_vertices.len())
        });
        let material_visit_count = products
            .iter()
            .map(|product| product.draw_plan.materials.len())
            .fold(0_usize, usize::saturating_add);
        self.publish_products(products, vertices, draw_plan);
        self.retained_frame_generation = Some(frame.generation());
        SdfCompiledTextPrepareReport {
            full_rebuild: true,
            segment_visit_count,
            vertex_visit_count,
            material_visit_count,
            ..Default::default()
        }
    }

    fn publish_products(
        &mut self,
        products: Vec<SdfCompiledTextSegmentProduct>,
        vertices: &mut Vec<ScreenSpaceUiSdfVertex>,
        draw_plan: &mut SdfTextMaterialDrawPlan,
    ) {
        vertices.clear();
        self.native_vertex_ranges.clear();
        self.sdf_decoration_vertex_ranges.clear();
        self.glyph_vertex_ranges.clear();
        self.material_ranges.clear();
        self.draw_ranges.clear();

        for product in &products {
            self.native_vertex_ranges.push(append_vertices(
                vertices,
                &product.native_decoration_vertices,
            ));
        }
        for product in &products {
            self.sdf_decoration_vertex_ranges
                .push(append_vertices(vertices, &product.sdf_decoration_vertices));
        }
        self.decoration_vertex_count = vertices.len().min(u32::MAX as usize) as u32;
        for product in &products {
            self.glyph_vertex_ranges
                .push(append_vertices(vertices, &product.glyph_vertices));
        }

        draw_plan.materials.clear();
        draw_plan.draws.clear();
        draw_plan.materials.push(SdfTextMaterial::default());
        if self.decoration_vertex_count > 0 {
            draw_plan.draws.push(SdfTextMaterialDraw {
                vertices: 0..self.decoration_vertex_count,
                material_index: 0,
            });
        }
        for (index, product) in products.into_iter().enumerate() {
            let material_start = draw_plan.materials.len();
            draw_plan.materials.extend(product.draw_plan.materials);
            self.material_ranges
                .push(material_start..draw_plan.materials.len());

            let draw_start = draw_plan.draws.len();
            let glyph_start = self.glyph_vertex_ranges[index].start as u32;
            for draw in product.draw_plan.draws {
                draw_plan.draws.push(SdfTextMaterialDraw {
                    vertices: draw.vertices.start.saturating_add(glyph_start)
                        ..draw.vertices.end.saturating_add(glyph_start),
                    material_index: draw.material_index.saturating_add(material_start as u32),
                });
            }
            self.draw_ranges.push(draw_start..draw_plan.draws.len());
        }
        self.effect_batch_counts = material_effect_counts(&draw_plan.materials);
    }

    fn apply_replacements(
        &mut self,
        replacements: Vec<(usize, SdfCompiledTextSegmentProduct)>,
        vertices: &mut [ScreenSpaceUiSdfVertex],
        draw_plan: &mut SdfTextMaterialDrawPlan,
    ) -> Option<SdfCompiledTextPrepareReport> {
        for (index, product) in &replacements {
            if self.native_vertex_ranges.get(*index)?.len()
                != product.native_decoration_vertices.len()
                || self.sdf_decoration_vertex_ranges.get(*index)?.len()
                    != product.sdf_decoration_vertices.len()
                || self.glyph_vertex_ranges.get(*index)?.len() != product.glyph_vertices.len()
                || self.material_ranges.get(*index)?.len() != product.draw_plan.materials.len()
                || self.draw_ranges.get(*index)?.len() != product.draw_plan.draws.len()
            {
                return None;
            }
        }

        let mut report = SdfCompiledTextPrepareReport::default();
        for (index, product) in replacements {
            let native_range = self.native_vertex_ranges[index].clone();
            let sdf_decoration_range = self.sdf_decoration_vertex_ranges[index].clone();
            let glyph_range = self.glyph_vertex_ranges[index].clone();
            let material_range = self.material_ranges[index].clone();
            let draw_range = self.draw_ranges[index].clone();
            let previous_effect_counts =
                material_effect_counts(&draw_plan.materials[material_range.clone()]);
            let replacement_effect_counts = material_effect_counts(&product.draw_plan.materials);

            vertices[native_range.clone()].copy_from_slice(&product.native_decoration_vertices);
            vertices[sdf_decoration_range.clone()]
                .copy_from_slice(&product.sdf_decoration_vertices);
            vertices[glyph_range.clone()].copy_from_slice(&product.glyph_vertices);
            draw_plan.materials[material_range.clone()]
                .clone_from_slice(&product.draw_plan.materials);
            for (output, replacement) in draw_plan.draws[draw_range]
                .iter_mut()
                .zip(product.draw_plan.draws)
            {
                *output = SdfTextMaterialDraw {
                    vertices: replacement
                        .vertices
                        .start
                        .saturating_add(glyph_range.start as u32)
                        ..replacement
                            .vertices
                            .end
                            .saturating_add(glyph_range.start as u32),
                    material_index: replacement
                        .material_index
                        .saturating_add(material_range.start as u32),
                };
            }

            report.vertex_visit_count = report
                .vertex_visit_count
                .saturating_add(native_range.len())
                .saturating_add(sdf_decoration_range.len())
                .saturating_add(glyph_range.len());
            report.material_visit_count = report
                .material_visit_count
                .saturating_add(material_range.len());
            report
                .changed_vertex_ranges
                .extend([native_range, sdf_decoration_range, glyph_range]);
            report.changed_material_ranges.push(material_range);
            for effect_index in 0..self.effect_batch_counts.len() {
                self.effect_batch_counts[effect_index] = self.effect_batch_counts[effect_index]
                    .saturating_sub(previous_effect_counts[effect_index])
                    .saturating_add(replacement_effect_counts[effect_index]);
            }
        }
        Some(report)
    }

    fn directories_match(
        &self,
        segment_count: usize,
        vertices: &[ScreenSpaceUiSdfVertex],
        draw_plan: &SdfTextMaterialDrawPlan,
    ) -> bool {
        self.sdf_run_ranges.len() == segment_count
            && self.native_metric_ranges.len() == segment_count
            && self.native_vertex_ranges.len() == segment_count
            && self.sdf_decoration_vertex_ranges.len() == segment_count
            && self.glyph_vertex_ranges.len() == segment_count
            && self.material_ranges.len() == segment_count
            && self.draw_ranges.len() == segment_count
            && self
                .glyph_vertex_ranges
                .last()
                .map_or(vertices.is_empty(), |range| range.end == vertices.len())
            && self
                .material_ranges
                .last()
                .map_or(draw_plan.materials.len() == 1, |range| {
                    range.end == draw_plan.materials.len()
                })
            && self
                .draw_ranges
                .last()
                .map_or(draw_plan.draws.is_empty(), |range| {
                    range.end == draw_plan.draws.len()
                })
    }

    pub(super) fn invalidate(&mut self) {
        self.retained_frame_generation = None;
        self.sdf_run_ranges.clear();
        self.native_metric_ranges.clear();
        self.native_vertex_ranges.clear();
        self.sdf_decoration_vertex_ranges.clear();
        self.glyph_vertex_ranges.clear();
        self.material_ranges.clear();
        self.draw_ranges.clear();
        self.decoration_vertex_count = 0;
        self.effect_batch_counts = [0; 3];
    }

    pub(super) fn decoration_vertex_count(&self) -> u32 {
        self.decoration_vertex_count
    }

    pub(super) fn effect_batch_counts(&self) -> [usize; 3] {
        self.effect_batch_counts
    }
}

#[allow(clippy::too_many_arguments)]
fn build_segment_product(
    segment: &ScreenSpaceUiTextSegmentProduct,
    sdf_cpu_runs: &[SdfRunCpuPreparation],
    native_decoration_metrics: &[TextDecorationMetrics],
    atlas_plan: &SdfAtlasPlan,
    atlas_runs: &[super::super::sdf_atlas::SdfAtlasRun],
    atlas_bake: &SdfAtlasBake,
    viewport_size: UVec2,
) -> SdfCompiledTextSegmentProduct {
    let mut native_decoration_vertices = Vec::new();
    build_text_decoration_vertices_iter(
        &mut native_decoration_vertices,
        segment.native_texts().iter(),
        native_decoration_metrics.iter().copied(),
        viewport_size,
    );

    let mut sdf_decoration_vertices = Vec::new();
    build_text_decoration_vertices_iter(
        &mut sdf_decoration_vertices,
        segment.sdf_texts().iter(),
        sdf_cpu_runs.iter().map(|run| run.decoration_metrics),
        viewport_size,
    );

    let mut glyph_vertices = Vec::new();
    let mut text_ranges = Vec::new();
    build_sdf_vertex_plan_with_runs_iter(
        &mut glyph_vertices,
        &mut text_ranges,
        segment.sdf_texts().iter(),
        segment.sdf_texts().len(),
        atlas_plan,
        atlas_runs,
        atlas_bake,
        sdf_cpu_runs,
        viewport_size,
    );
    let mut draw_plan = SdfTextMaterialDrawPlan::default();
    draw_plan.rebuild_iter(
        segment.sdf_texts().iter(),
        atlas_plan.atlas_size,
        0,
        &text_ranges,
    );
    if draw_plan.draws.is_empty() {
        draw_plan.materials.clear();
    }
    SdfCompiledTextSegmentProduct {
        native_decoration_vertices,
        sdf_decoration_vertices,
        glyph_vertices,
        draw_plan,
    }
}

fn append_vertices(
    vertices: &mut Vec<ScreenSpaceUiSdfVertex>,
    appended: &[ScreenSpaceUiSdfVertex],
) -> Range<usize> {
    let start = vertices.len();
    vertices.extend_from_slice(appended);
    start..vertices.len()
}

fn material_effect_counts(materials: &[SdfTextMaterial]) -> [usize; 3] {
    let mut counts = [0_usize; 3];
    for material in materials {
        counts[0] = counts[0].saturating_add(usize::from(
            material.effect_flags & SDF_TEXT_EFFECT_OUTLINE != 0,
        ));
        counts[1] = counts[1].saturating_add(usize::from(
            material.effect_flags & SDF_TEXT_EFFECT_SHADOW != 0,
        ));
        counts[2] = counts[2].saturating_add(usize::from(
            material.effect_flags & SDF_TEXT_EFFECT_GLOW != 0,
        ));
    }
    counts
}

#[cfg(test)]
#[path = "tests/segment_product.rs"]
mod tests;
