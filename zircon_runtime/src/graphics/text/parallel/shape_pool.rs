//! Parallel paragraph shaping batch helpers.

use std::sync::Arc;

use crate::core::framework::render::{
    ShapedGlyphRun, TextOrientation, TextShapeRequest, VerticalMode,
};
use crate::core::runtime::tasks::{parallel_for, TaskPool};
use crate::graphics::text::cache::{ShapedRunCache, ShapedRunCacheKey};
use crate::graphics::text::shaping::shape_text;
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextDirection, UiTextRange};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextShapeParagraph {
    text: Arc<str>,
    style: UiResolvedStyle,
    base_direction: UiTextDirection,
    source_range: UiTextRange,
    orientation: TextOrientation,
    vertical_mode: VerticalMode,
    include_kerning: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextParallelShapeBatchReport {
    pub(crate) requested_count: usize,
    pub(crate) cache_hit_count: usize,
    pub(crate) cache_miss_count: usize,
    pub(crate) batch_duplicate_count: usize,
    pub(crate) shaped_count: usize,
    pub(crate) inserted_count: usize,
    pub(crate) chunk_size: usize,
    pub(crate) worker_parallelism: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextParallelShapeBatch {
    pub(crate) runs: Vec<Arc<ShapedGlyphRun>>,
    pub(crate) report: TextParallelShapeBatchReport,
}

#[derive(Clone, Debug, PartialEq)]
struct PendingShapeJob {
    key: ShapedRunCacheKey,
    request: TextShapeParagraph,
    output_indices: Vec<usize>,
    run: Option<ShapedGlyphRun>,
}

impl TextShapeParagraph {
    pub(crate) fn horizontal(
        text: impl Into<Arc<str>>,
        style: UiResolvedStyle,
        base_direction: UiTextDirection,
        source_range: UiTextRange,
    ) -> Self {
        Self::horizontal_with_kerning(text, style, base_direction, source_range, true)
    }

    pub(crate) fn horizontal_with_kerning(
        text: impl Into<Arc<str>>,
        style: UiResolvedStyle,
        base_direction: UiTextDirection,
        source_range: UiTextRange,
        include_kerning: bool,
    ) -> Self {
        Self {
            text: text.into(),
            style,
            base_direction,
            source_range,
            orientation: TextOrientation::Horizontal,
            vertical_mode: VerticalMode::Mixed,
            include_kerning,
        }
    }

    fn request(&self) -> TextShapeRequest<'_> {
        TextShapeRequest {
            text: self.text.as_ref(),
            style: &self.style,
            base_direction: self.base_direction,
            source_range: self.source_range,
            orientation: self.orientation,
            vertical_mode: self.vertical_mode,
            include_kerning: self.include_kerning,
        }
    }

    fn text(&self) -> &str {
        self.text.as_ref()
    }

    fn text_arc(&self) -> Arc<str> {
        Arc::clone(&self.text)
    }
}

impl TextParallelShapeBatchReport {
    fn for_requests(requested_count: usize, chunk_size: usize, worker_parallelism: usize) -> Self {
        Self {
            requested_count,
            chunk_size: chunk_size.max(1),
            worker_parallelism,
            ..Self::default()
        }
    }
}

impl PendingShapeJob {
    fn new(key: ShapedRunCacheKey, request: TextShapeParagraph, output_index: usize) -> Self {
        Self {
            key,
            request,
            output_indices: vec![output_index],
            run: None,
        }
    }

    fn matches(&self, key: &ShapedRunCacheKey, text: &str) -> bool {
        &self.key == key && self.request.text() == text
    }

    fn shape(&mut self) {
        self.run = Some(shape_text(self.request.request()));
    }
}

pub(crate) fn shape_paragraphs_with_cache(
    pool: &TaskPool,
    cache: &mut ShapedRunCache,
    requests: &[TextShapeParagraph],
    chunk_size: usize,
) -> TextParallelShapeBatch {
    let chunk_size = chunk_size.max(1);
    let mut report =
        TextParallelShapeBatchReport::for_requests(requests.len(), chunk_size, pool.parallelism());
    let mut runs = vec![None; requests.len()];
    let mut pending: Vec<PendingShapeJob> = Vec::new();

    for (index, request) in requests.iter().enumerate() {
        let borrowed = request.request();
        let key = ShapedRunCacheKey::from_request(&borrowed);
        if let Some(job) = pending
            .iter_mut()
            .find(|job| job.matches(&key, request.text()))
        {
            job.output_indices.push(index);
            report.batch_duplicate_count = report.batch_duplicate_count.saturating_add(1);
            continue;
        }

        if let Some(run) = cache.get(&key, request.text()) {
            runs[index] = Some(run);
            report.cache_hit_count = report.cache_hit_count.saturating_add(1);
        } else {
            pending.push(PendingShapeJob::new(key, request.clone(), index));
            report.cache_miss_count = report.cache_miss_count.saturating_add(1);
        }
    }

    report.shaped_count = pending.len();
    if !pending.is_empty() {
        parallel_for(pool, pending.as_mut_slice(), chunk_size, |jobs| {
            for job in jobs {
                job.shape();
            }
        });
    }

    for mut job in pending {
        let Some(run) = job.run.take() else {
            continue;
        };
        let run = cache.insert(job.key, job.request.text_arc(), run);
        report.inserted_count = report.inserted_count.saturating_add(1);
        for output_index in job.output_indices {
            runs[output_index] = Some(Arc::clone(&run));
        }
    }

    let ordered_runs = runs.into_iter().flatten().collect::<Vec<_>>();
    debug_assert_eq!(ordered_runs.len(), requests.len());

    TextParallelShapeBatch {
        runs: ordered_runs,
        report,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{shape_paragraphs_with_cache, TextShapeParagraph};
    use crate::core::runtime::tasks::{TaskPool, TaskPoolDescriptor};
    use crate::graphics::text::cache::ShapedRunCache;
    use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextDirection, UiTextRange};

    #[test]
    fn render_perf_text_parallel_shape_count() {
        let style = compact_editor_label_style();
        let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));
        let mut cache = ShapedRunCache::with_capacity(16);
        let first_requests = paragraphs(&[
            "editor base.zui",
            "folder-open-outline.svg",
            "workbench_panel.rs",
        ]);

        cache.begin_frame(1);
        let first = shape_paragraphs_with_cache(&pool, &mut cache, &first_requests, 1);
        let first_cache_report = cache.report();

        assert_eq!(first.runs.len(), first_requests.len());
        assert_eq!(first.report.cache_hit_count, 0);
        assert_eq!(first.report.cache_miss_count, first_requests.len());
        assert_eq!(first.report.shaped_count, first_requests.len());
        assert_eq!(first.report.inserted_count, first_requests.len());
        assert_eq!(first_cache_report.miss_count, first_requests.len() as u64);
        assert_eq!(first_cache_report.insert_count, first_requests.len() as u64);

        cache.begin_frame(2);
        let second_requests = vec![
            paragraph("folder-open-outline.svg", &style),
            paragraph("workbench_panel.rs", &style),
            paragraph("retained_text_metrics.rs", &style),
        ];
        let second = shape_paragraphs_with_cache(&pool, &mut cache, &second_requests, 1);
        let second_cache_report = cache.report();

        assert_eq!(second.runs.len(), second_requests.len());
        assert_eq!(second.report.cache_hit_count, 2);
        assert_eq!(second.report.cache_miss_count, 1);
        assert_eq!(second.report.shaped_count, 1);
        assert_eq!(second.report.inserted_count, 1);
        assert_eq!(
            second_cache_report.hit_count, 2,
            "cached paragraphs should not enter the parallel shape work set"
        );
        assert_eq!(
            second_cache_report.miss_count, 1,
            "only the newly visible paragraph should miss the shaped-run cache"
        );
        assert_eq!(second_cache_report.insert_count, 1);
        assert_eq!(second.runs[0].source_text, "folder-open-outline.svg");
        assert_eq!(second.runs[1].source_text, "workbench_panel.rs");
        assert_eq!(second.runs[2].source_text, "retained_text_metrics.rs");
    }

    #[test]
    fn text_parallel_shape_batch_deduplicates_same_frame_misses() {
        let style = compact_editor_label_style();
        let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));
        let mut cache = ShapedRunCache::with_capacity(16);
        let requests = vec![
            paragraph("editor base.zui", &style),
            paragraph("folder-open-outline.svg", &style),
            paragraph("editor base.zui", &style),
            paragraph("folder-open-outline.svg", &style),
        ];

        cache.begin_frame(1);
        let batch = shape_paragraphs_with_cache(&pool, &mut cache, &requests, 1);
        let cache_report = cache.report();

        assert_eq!(batch.runs.len(), requests.len());
        assert_eq!(batch.report.cache_miss_count, 2);
        assert_eq!(batch.report.batch_duplicate_count, 2);
        assert_eq!(batch.report.shaped_count, 2);
        assert_eq!(batch.report.inserted_count, 2);
        assert_eq!(cache_report.miss_count, 2);
        assert_eq!(cache_report.insert_count, 2);
        assert!(Arc::ptr_eq(&batch.runs[0], &batch.runs[2]));
        assert!(Arc::ptr_eq(&batch.runs[1], &batch.runs[3]));
    }

    fn paragraphs(texts: &[&str]) -> Vec<TextShapeParagraph> {
        let style = compact_editor_label_style();
        texts.iter().map(|text| paragraph(text, &style)).collect()
    }

    fn paragraph(text: &str, style: &UiResolvedStyle) -> TextShapeParagraph {
        TextShapeParagraph::horizontal(
            Arc::<str>::from(text),
            style.clone(),
            UiTextDirection::LeftToRight,
            UiTextRange {
                start: 0,
                end: text.len(),
            },
        )
    }

    fn compact_editor_label_style() -> UiResolvedStyle {
        UiResolvedStyle {
            font_size: 10.0,
            line_height: 12.0,
            ..UiResolvedStyle::default()
        }
    }
}
