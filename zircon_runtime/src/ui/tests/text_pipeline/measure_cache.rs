use crate::core::runtime::tasks::{TaskPool, TaskPoolDescriptor};
use crate::text::layout::measure_line_width;
use crate::ui::text::{
    UiTextLayoutRequest, UiTextMeasureCache, UiTextShapePrewarmRequest, UiWidthBucket,
};
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiResolvedStyle, UiTextWrap, UiTextWritingMode},
};

#[test]
fn ui_text_hot_paths_borrow_existing_text_and_advances() {
    let hit_test = include_str!("../../text/hit_test.rs");
    let resolved_layout = include_str!("../../text/resolved_layout.rs");
    let measure_cache = include_str!("../../text/measure_cache.rs");
    let wrapping = include_str!("../../text/layout_engine/wrapping.rs");

    assert!(
        hit_test.contains("Cow::Borrowed(&line.glyph_advances)"),
        "hit testing should borrow already-resolved glyph advances"
    );
    assert!(
        resolved_layout.contains("pub(crate) fn resolved_text(&self) -> Cow<'_, str>"),
        "plain layout requests should borrow their source text"
    );
    assert!(
        measure_cache.contains("let resolved_text: Arc<str> = Arc::from(resolved_text.as_ref());"),
        "layout caches should share one resolved source allocation after the frame-cache miss"
    );
    assert!(
        wrapping.contains("struct TextSegment<'a>") && wrapping.contains("text: &'a str"),
        "newline segmentation should borrow source slices"
    );
}

#[test]
fn text_measure_cache_hits_same_layout_request() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::Word,
        ..UiResolvedStyle::default()
    };
    let request = UiTextLayoutRequest::new(
        "Alpha Beta",
        &style,
        UiFrame::new(0.0, 0.0, 60.0, 20.0),
        None,
    );
    let mut cache = UiTextMeasureCache::default();

    assert_eq!(cache.resolve_or_shape(&request).layout.lines.len(), 1);
    assert_eq!(cache.resolve_or_shape(&request).layout.lines.len(), 1);

    assert_eq!(cache.frame_shape_count(), 1);
    assert_eq!(cache.frame_layout_dedup_report().miss_count, 1);
    assert_eq!(cache.frame_layout_dedup_report().hit_count, 1);
    assert!(UiWidthBucket::from_request(&request).value() >= 1);
}

#[test]
fn text_measure_cache_separates_layouts_by_writing_mode() {
    let horizontal_style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::Word,
        text_writing_mode: UiTextWritingMode::HorizontalTb,
        ..UiResolvedStyle::default()
    };
    let vertical_style = UiResolvedStyle {
        text_writing_mode: UiTextWritingMode::VerticalRl,
        ..horizontal_style.clone()
    };
    let frame = UiFrame::new(0.0, 0.0, 64.0, 64.0);
    let mut cache = UiTextMeasureCache::default();

    let horizontal = cache.resolve_or_shape(&UiTextLayoutRequest::new(
        "Alpha Beta",
        &horizontal_style,
        frame,
        None,
    ));
    let vertical = cache.resolve_or_shape(&UiTextLayoutRequest::new(
        "Alpha Beta",
        &vertical_style,
        frame,
        None,
    ));

    assert_eq!(
        horizontal.layout.writing_mode,
        UiTextWritingMode::HorizontalTb
    );
    assert_eq!(vertical.layout.writing_mode, UiTextWritingMode::VerticalRl);
    assert_ne!(
        horizontal.layout.lines[0].frame, vertical.layout.lines[0].frame,
        "vertical text must not reuse the prior horizontal absolute layout entry"
    );
    assert_eq!(
        cache.frame_layout_report().miss_count,
        2,
        "writing mode must participate in the persistent layout cache key"
    );
    assert_eq!(
        cache.frame_layout_dedup_report().miss_count,
        2,
        "writing mode must also participate in same-frame layout dedup"
    );
}

#[test]
fn text_measure_cache_reuses_layout_across_native_and_sdf_modes() {
    let native_style = UiResolvedStyle {
        font_size: 14.0,
        line_height: 18.0,
        wrap: UiTextWrap::Glyph,
        text_render_mode: zircon_runtime_interface::ui::surface::UiTextRenderMode::Native,
        ..UiResolvedStyle::default()
    };
    let sdf_style = UiResolvedStyle {
        text_render_mode: zircon_runtime_interface::ui::surface::UiTextRenderMode::Sdf,
        ..native_style.clone()
    };
    let frame = UiFrame::new(8.0, 12.0, 72.0, 96.0);
    let mut cache = UiTextMeasureCache::default();

    let native = cache.resolve_or_shape(&UiTextLayoutRequest::new(
        "Alpha世界Beta",
        &native_style,
        frame,
        None,
    ));
    let sdf = cache.resolve_or_shape(&UiTextLayoutRequest::new(
        "Alpha世界Beta",
        &sdf_style,
        frame,
        None,
    ));

    assert_eq!(native.layout, sdf.layout);
    assert_eq!(cache.frame_layout_report().miss_count, 1);
    assert_eq!(cache.frame_layout_dedup_report().hit_count, 1);
}

#[test]
fn render_perf_text_measure_then_layout_shapes_once() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::None,
        ..UiResolvedStyle::default()
    };
    let text = "editor base.zui";
    let request = UiTextLayoutRequest::new(text, &style, UiFrame::new(0.0, 0.0, 180.0, 20.0), None);
    let mut cache = UiTextMeasureCache::default();

    cache.begin_frame();
    let metrics_warmup = cache.measure_text_size("Hg", &style);
    let warmed_shape_report = cache.frame_shaped_run_report();
    let measured = cache.measure_text_size(text, &style);
    let resolution = cache.resolve_or_shape(&request);
    let shape_report = cache.frame_shaped_run_report();

    assert!(metrics_warmup.width > 0.0);
    assert!(measured.width > 0.0);
    assert_eq!(resolution.layout.lines.len(), 1);
    assert_eq!(
        shape_report
            .miss_count
            .saturating_sub(warmed_shape_report.miss_count),
        1,
        "measure + layout for one non-tab label should shape the source run once after style metrics are warm"
    );
    assert_eq!(
        shape_report
            .insert_count
            .saturating_sub(warmed_shape_report.insert_count),
        1,
        "the source label should be the only post-warmup shaped-run insertion"
    );
    assert!(
        shape_report.hit_count >= 2,
        "layout should reuse the run populated by measurement for metrics and advances"
    );
}

#[test]
fn render_perf_text_scroll_list_reuses_cache() {
    const VIEWPORT_ROWS: usize = 5;
    const SCROLL_ROWS: usize = 3;

    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::None,
        ..UiResolvedStyle::default()
    };
    let rows = [
        "editor base.zui",
        "folder-open-outline.svg",
        "workbench_panel.rs",
        "selected_component.zui",
        "scene_tree_row.zui",
        "asset_preview_panel.zui",
        "command_palette_entry.zui",
        "retained_text_metrics.rs",
    ];
    let mut cache = UiTextMeasureCache::default();

    cache.begin_frame();
    let metrics_warmup = cache.measure_text_size("Hg", &style);
    let warmed_shape_report = cache.frame_shaped_run_report();
    render_text_scroll_window(&mut cache, &style, &rows, 0, VIEWPORT_ROWS);
    let first_shape_report = cache.frame_shaped_run_report();
    let first_layout_report = cache.frame_layout_report();

    assert!(metrics_warmup.width > 0.0);
    assert_eq!(
        first_shape_report
            .miss_count
            .saturating_sub(warmed_shape_report.miss_count),
        VIEWPORT_ROWS as u64,
        "first visible window should shape each newly visible row once after metrics warmup"
    );
    assert_eq!(
        first_shape_report
            .insert_count
            .saturating_sub(warmed_shape_report.insert_count),
        VIEWPORT_ROWS as u64,
        "first visible window should insert one shaped run per newly visible row"
    );
    assert_eq!(
        first_layout_report.miss_count, VIEWPORT_ROWS as u64,
        "each first-window row needs an absolute layout entry"
    );
    cache.finish_frame();

    cache.begin_frame();
    render_text_scroll_window(&mut cache, &style, &rows, SCROLL_ROWS, VIEWPORT_ROWS);
    let scrolled_shape_report = cache.frame_shaped_run_report();
    let scrolled_layout_report = cache.frame_layout_report();
    let new_rows_after_scroll = SCROLL_ROWS.min(VIEWPORT_ROWS);
    let reused_rows_after_scroll = VIEWPORT_ROWS - new_rows_after_scroll;

    assert_eq!(
        scrolled_shape_report.miss_count, new_rows_after_scroll as u64,
        "scrolling should only shape rows that enter the viewport"
    );
    assert_eq!(
        scrolled_shape_report.insert_count, new_rows_after_scroll as u64,
        "scrolling should only insert shaped runs for newly visible rows"
    );
    assert!(
        scrolled_shape_report.hit_count >= reused_rows_after_scroll as u64,
        "overlapping rows should hit the shaped-run cache after scroll"
    );
    assert_eq!(
        scrolled_layout_report.miss_count, VIEWPORT_ROWS as u64,
        "absolute layout still changes with row y positions after scroll"
    );
    assert_eq!(
        scrolled_layout_report.hit_count, 0,
        "layout cache must not reuse text geometry from a different absolute frame"
    );
}

#[test]
fn render_perf_text_parallel_shape_pool_prewarms_ui_measure_cache() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::None,
        ..UiResolvedStyle::default()
    };
    let rows = [
        "editor base.zui",
        "folder-open-outline.svg",
        "workbench_panel.rs",
        "editor base.zui",
        "retained_text_metrics.rs",
    ];
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));
    let mut cache = UiTextMeasureCache::default();

    cache.begin_frame();
    let metrics_warmup = cache.measure_text_size("Hg", &style);
    let warmed_shape_report = cache.frame_shaped_run_report();
    let requests = rows
        .iter()
        .map(|text| UiTextShapePrewarmRequest::horizontal(*text, style.clone()))
        .collect::<Vec<_>>();
    let prewarm_report = cache.prewarm_horizontal_paragraphs(&pool, &requests, 1);
    let prewarmed_shape_report = cache.frame_shaped_run_report();

    assert!(metrics_warmup.width > 0.0);
    assert_eq!(prewarm_report.requested_count, rows.len());
    assert_eq!(prewarm_report.cache_hit_count, 0);
    assert_eq!(prewarm_report.cache_miss_count, 4);
    assert_eq!(prewarm_report.batch_duplicate_count, 1);
    assert_eq!(prewarm_report.shaped_count, 4);
    assert_eq!(prewarm_report.inserted_count, 4);
    assert_eq!(
        prewarmed_shape_report
            .miss_count
            .saturating_sub(warmed_shape_report.miss_count),
        4,
        "parallel UI prewarm should shape each unique visible editor row once"
    );
    assert_eq!(
        prewarmed_shape_report
            .insert_count
            .saturating_sub(warmed_shape_report.insert_count),
        4,
        "parallel UI prewarm should insert the visible editor rows into the shared shaped-run cache"
    );

    render_text_scroll_window(&mut cache, &style, &rows, 0, rows.len());
    let after_layout_shape_report = cache.frame_shaped_run_report();
    let layout_report = cache.frame_layout_report();

    assert_eq!(
        after_layout_shape_report.miss_count, prewarmed_shape_report.miss_count,
        "layout should not shape prewarmed editor rows again"
    );
    assert_eq!(
        after_layout_shape_report.insert_count, prewarmed_shape_report.insert_count,
        "layout should not insert duplicate shaped runs after prewarm"
    );
    assert!(
        after_layout_shape_report.hit_count > prewarmed_shape_report.hit_count,
        "layout should consume the runs populated by the parallel prewarm"
    );
    assert_eq!(
        layout_report.miss_count,
        rows.len() as u64,
        "prewarming shaped runs does not bypass absolute frame layout entries"
    );
}

#[test]
fn text_measure_cache_reuses_shaped_runs_between_measure_and_layout() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::None,
        ..UiResolvedStyle::default()
    };
    let text = "editor base.zui";
    let request = UiTextLayoutRequest::new(text, &style, UiFrame::new(0.0, 0.0, 180.0, 20.0), None);
    let mut cache = UiTextMeasureCache::default();

    cache.begin_frame();
    let measured = cache.measure_text_size(text, &style);
    let after_measure = cache.frame_shaped_run_report();
    let resolution = cache.resolve_or_shape(&request);
    let after_layout = cache.frame_shaped_run_report();

    assert!(measured.width > 0.0);
    assert_eq!(resolution.layout.lines.len(), 1);
    assert_eq!(
        after_measure.miss_count, 2,
        "measuring an unwrapped line should shape metrics and the source text once each"
    );
    assert_eq!(
        after_layout.miss_count, after_measure.miss_count,
        "layout should reuse the shaped metric and source text runs populated by measurement"
    );
    assert!(
        after_layout.hit_count >= after_measure.hit_count + 2,
        "layout should hit both shared shaped runs instead of shaping new copies"
    );
}

#[test]
fn text_measure_cache_reshapes_when_frame_origin_changes() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::Word,
        ..UiResolvedStyle::default()
    };
    let first = UiTextLayoutRequest::new(
        "Alpha Beta",
        &style,
        UiFrame::new(8.0, 0.0, 60.0, 20.0),
        None,
    );
    let shifted = UiTextLayoutRequest::new(
        "Alpha Beta",
        &style,
        UiFrame::new(24.0, 0.0, 60.0, 20.0),
        None,
    );
    let mut cache = UiTextMeasureCache::default();

    assert_eq!(cache.resolve_or_shape(&first).layout.lines[0].frame.x, 8.0);
    assert_eq!(
        cache.resolve_or_shape(&shifted).layout.lines[0].frame.x,
        24.0
    );

    assert_eq!(cache.frame_shape_count(), 2);
}

#[test]
fn text_measure_cache_reshapes_when_wrap_bucket_changes() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::Word,
        ..UiResolvedStyle::default()
    };
    let neutral_style = crate::ui::text::text_style(&style);
    let alpha_width = measure_line_width("Alpha", &neutral_style);
    let beta_width = measure_line_width("Beta", &neutral_style);
    let full_width = measure_line_width("Alpha Beta", &neutral_style);
    let narrow_width = alpha_width.max(beta_width) + 0.5;
    let wide_width = full_width + 0.5;
    assert!(
        narrow_width < full_width,
        "test fixture must fit each word while forcing the phrase to wrap"
    );
    let narrow = UiTextLayoutRequest::new(
        "Alpha Beta",
        &style,
        UiFrame::new(0.0, 0.0, narrow_width, 40.0),
        None,
    );
    let wide = UiTextLayoutRequest::new(
        "Alpha Beta",
        &style,
        UiFrame::new(0.0, 0.0, wide_width, 20.0),
        None,
    );
    let mut cache = UiTextMeasureCache::default();

    assert_eq!(cache.resolve_or_shape(&narrow).layout.lines.len(), 2);
    assert_eq!(cache.resolve_or_shape(&wide).layout.lines.len(), 1);

    assert_eq!(cache.frame_shape_count(), 2);
}

fn render_text_scroll_window(
    cache: &mut UiTextMeasureCache,
    style: &UiResolvedStyle,
    rows: &[&str],
    start_row: usize,
    visible_rows: usize,
) {
    const ROW_HEIGHT: f32 = 16.0;

    for (slot, text) in rows[start_row..start_row + visible_rows].iter().enumerate() {
        let frame = UiFrame::new(0.0, slot as f32 * ROW_HEIGHT, 220.0, ROW_HEIGHT);
        let request = UiTextLayoutRequest::new(text, style, frame, None);
        let resolution = cache.resolve_or_shape(&request);

        assert_eq!(resolution.layout.lines.len(), 1);
        assert!(resolution.size.width > 0.0);
    }
}
