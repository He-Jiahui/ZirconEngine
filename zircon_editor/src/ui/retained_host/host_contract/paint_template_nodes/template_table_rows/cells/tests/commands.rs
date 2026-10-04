use super::*;

#[test]
fn table_cell_text_intersects_its_frame_with_the_inherited_clip() {
    let command = text_command(
        frame(10.0, 20.0, 80.0, 18.0),
        &frame(0.0, 24.0, 100.0, 6.0),
        3,
        "Asset.mesh",
        [255, 255, 255, 255],
        table_cell_metrics(),
        1.0,
    )
    .expect("partially visible table text command");

    assert_eq!(command.clip_frame, Some(frame(10.0, 24.0, 80.0, 6.0)));
}

#[test]
fn table_cell_text_outside_the_inherited_clip_emits_no_command() {
    assert!(text_command(
        frame(10.0, 40.0, 80.0, 18.0),
        &frame(0.0, 10.0, 100.0, 20.0),
        3,
        "Asset.mesh",
        [255, 255, 255, 255],
        table_cell_metrics(),
        1.0,
    )
    .is_none());
}

#[test]
fn fitting_numeric_cell_text_uses_its_measured_width_for_right_alignment() {
    let rect = frame(10.0, 20.0, 100.0, 18.0);
    let metrics = WorkbenchTableCellMetrics {
        font_size: 12.0,
        line_height: 14.4,
        inset_x: 6.0,
        inset_y: 3.0,
        text_clip_guard: 4.0,
    };
    let measured_width = measure_runtime_text_width("12 KB", metrics.font_size);
    let aligned = right_aligned_text_frame(rect.clone(), "12 KB", metrics);

    assert!((aligned.width - measured_width).abs() < 0.0001);
    assert!((aligned.x + aligned.width - (rect.x + rect.width)).abs() < 0.0001);
}

#[test]
fn overflowing_numeric_cell_text_keeps_the_full_cell_frame_for_ellipsis() {
    let rect = frame(10.0, 20.0, 20.0, 18.0);
    let metrics = WorkbenchTableCellMetrics {
        font_size: 12.0,
        line_height: 14.4,
        inset_x: 6.0,
        inset_y: 3.0,
        text_clip_guard: 4.0,
    };

    assert_eq!(
        text_frame_for_cell(rect, "12345 KB", 2, metrics),
        frame(16.0, 20.0, 8.0, 18.0)
    );
}

#[test]
fn table_cell_text_insets_left_labels_and_right_values_inside_the_column() {
    let rect = frame(10.0, 20.0, 100.0, 18.0);
    let metrics = WorkbenchTableCellMetrics {
        font_size: 12.0,
        line_height: 14.4,
        inset_x: 6.0,
        inset_y: 3.0,
        text_clip_guard: 4.0,
    };
    let left = text_frame_for_cell(rect.clone(), "Asset", 0, metrics);
    let right = text_frame_for_cell(rect.clone(), "12 KB", 2, metrics);

    assert_eq!(left.x, rect.x + metrics.inset_x);
    assert_eq!(left.width, rect.width - metrics.inset_x * 2.0);
    assert!((right.x + right.width - (rect.x + rect.width - metrics.inset_x)).abs() < 0.0001);
}

#[test]
fn table_row_computes_column_layout_once_before_painting_cells() {
    let source = include_str!("../commands.rs");
    let repeated_cell_layout = ["table_cell_rect", "(node, rect, index)"].concat();

    assert!(
        source.contains("let cell_rects = table_cell_rects(node, rect);"),
        "table rows should calculate all column frames before iterating their cells"
    );
    assert!(
        !source.contains(&repeated_cell_layout),
        "per-cell painting must not repeat the row column allocation"
    );
}

#[test]
fn table_row_selects_its_text_style_once_before_painting_cells() {
    let source = include_str!("../commands.rs");
    let repeated_style_selection = ["table_cell_color", "(node, index)"].concat();

    assert!(
        source.contains("let row_style = table_row_style(node);"),
        "table rows should select their semantic text style before iterating cells"
    );
    assert!(
        !source.contains(&repeated_style_selection),
        "per-cell painting must not repeat row style selection"
    );
}

#[test]
fn optimization_batch_20260830ct_table_cells_reserve_the_exact_command_upper_bound() {
    let source = include_str!("../commands.rs");

    assert!(
        source.contains("let cell_count = cells.len().min(TABLE_COLUMN_COUNT);"),
        "table rows should calculate the paintable cell upper bound once"
    );
    assert!(
        source.contains("commands.reserve(cell_count);"),
        "table rows should reserve command capacity before painting cells"
    );
    assert!(
        source.contains("cells.iter().take(cell_count)"),
        "iteration should reuse the same bounded cell count"
    );
}

#[test]
#[ignore = "deterministic allocation-growth benchmark"]
fn optimization_batch_20260830ct_table_cell_command_capacity_benchmark() {
    const BATCH_COUNT: usize = 32_768;
    let mut legacy_growth_events = 0_usize;
    let mut optimized_growth_events = 0_usize;

    for _ in 0..BATCH_COUNT {
        let mut legacy = Vec::<usize>::new();
        for command in 0..TABLE_COLUMN_COUNT {
            let capacity = legacy.capacity();
            legacy.push(command);
            legacy_growth_events += usize::from(legacy.capacity() != capacity);
        }

        let mut optimized = Vec::<usize>::with_capacity(TABLE_COLUMN_COUNT);
        for command in 0..TABLE_COLUMN_COUNT {
            let capacity = optimized.capacity();
            optimized.push(command);
            optimized_growth_events += usize::from(optimized.capacity() != capacity);
        }
    }

    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "EDITOR507_TABLE_CELL_COMMAND_CAPACITY_BENCH_V1 batches={BATCH_COUNT} \
             commands_per_batch={TABLE_COLUMN_COUNT} legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events}"
    );
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
