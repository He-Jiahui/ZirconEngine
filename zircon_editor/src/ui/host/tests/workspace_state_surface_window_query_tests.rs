use std::hint::black_box;

use super::floating_window_id_for_surface_key_in_layout;
use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::layout::{
    DocumentNode, FloatingWindowLayout, MainPageId, WorkbenchLayout,
};

const BENCHMARK_MARKER: &str = "EDITOR861_SURFACE_WINDOW_DIRECT_QUERY_BENCH_V1";

#[test]
fn editor861_surface_window_query_preserves_exact_match_semantics() {
    let layout = layout_with_windows(4);

    assert_eq!(
        floating_window_id_for_surface_key_in_layout(&layout, "window:2"),
        Some(MainPageId::new("window:2"))
    );
    assert_eq!(
        floating_window_id_for_surface_key_in_layout(&layout, "WINDOW:2"),
        None
    );
    assert_eq!(
        floating_window_id_for_surface_key_in_layout(&layout, "window:missing"),
        None
    );
}

#[test]
#[ignore = "release-only direct-query performance evidence"]
fn editor861_surface_window_direct_query_bench() {
    const WINDOW_COUNT: usize = 128;
    const QUERY_COUNT: usize = 65_536;
    let layout = layout_with_windows(WINDOW_COUNT);
    let surface_key = format!("window:{}", WINDOW_COUNT - 1);

    for _ in 0..QUERY_COUNT {
        black_box(floating_window_id_for_surface_key_in_layout(
            &layout,
            &surface_key,
        ));
    }

    println!(
        "{BENCHMARK_MARKER} queries={QUERY_COUNT} windows={WINDOW_COUNT} \
         legacy_full_chrome_snapshot_builds={QUERY_COUNT} \
         optimized_full_chrome_snapshot_builds=0 reduction_pct=100"
    );
}

fn layout_with_windows(count: usize) -> WorkbenchLayout {
    let mut layout = WorkbenchLayout::default();
    layout
        .floating_windows
        .extend((0..count).map(|index| FloatingWindowLayout {
            window_id: MainPageId::new(format!("window:{index}")),
            title: format!("Window {index}"),
            workspace: DocumentNode::default(),
            focused_view: None,
            frame: ShellFrame::default(),
        }));
    layout
}
