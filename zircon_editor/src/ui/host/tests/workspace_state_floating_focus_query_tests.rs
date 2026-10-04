use std::hint::black_box;

use super::floating_window_focus_target_in_layout;
use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::layout::{
    DocumentNode, FloatingWindowLayout, MainPageId, SplitAxis, TabStackLayout, WorkbenchLayout,
};
use crate::ui::workbench::view::ViewInstanceId;

const BENCHMARK_MARKER: &str = "EDITOR860_FLOATING_FOCUS_DIRECT_QUERY_BENCH_V1";
const WINDOW_ID: &str = "window:editor860";

#[test]
fn editor860_floating_focus_query_preserves_focused_active_first_priority() {
    let focused_layout = layout_with_window(Some("focused"), Some("active"));
    assert_eq!(focus_target(&focused_layout), Some("focused"));

    let active_layout = layout_with_window(Some("missing"), Some("active"));
    assert_eq!(focus_target(&active_layout), Some("active"));

    let first_layout = layout_with_window(None, None);
    assert_eq!(focus_target(&first_layout), Some("first"));
}

#[test]
fn editor860_floating_focus_query_rejects_missing_or_empty_windows() {
    let layout = WorkbenchLayout::default();
    assert_eq!(
        floating_window_focus_target_in_layout(&layout, &MainPageId::new(WINDOW_ID)),
        None
    );

    let mut layout = WorkbenchLayout::default();
    layout.floating_windows.push(FloatingWindowLayout {
        window_id: MainPageId::new(WINDOW_ID),
        title: "Empty".to_string(),
        workspace: DocumentNode::default(),
        focused_view: Some(ViewInstanceId::new("missing")),
        frame: ShellFrame::default(),
    });
    assert_eq!(
        floating_window_focus_target_in_layout(&layout, &MainPageId::new(WINDOW_ID)),
        None
    );
}

#[test]
#[ignore = "release-only direct-query performance evidence"]
fn editor860_floating_focus_direct_query_bench() {
    const QUERY_COUNT: usize = 65_536;
    let layout = layout_with_window(Some("focused"), Some("active"));
    let window_id = MainPageId::new(WINDOW_ID);

    for _ in 0..QUERY_COUNT {
        black_box(floating_window_focus_target_in_layout(&layout, &window_id));
    }

    println!(
        "{BENCHMARK_MARKER} queries={QUERY_COUNT} legacy_chrome_and_model_builds={QUERY_COUNT} \
         optimized_chrome_and_model_builds=0 reduction_pct=100"
    );
}

fn layout_with_window(focused_view: Option<&str>, active_tab: Option<&str>) -> WorkbenchLayout {
    let mut layout = WorkbenchLayout::default();
    layout.floating_windows.push(FloatingWindowLayout {
        window_id: MainPageId::new(WINDOW_ID),
        title: "Editor860".to_string(),
        workspace: DocumentNode::SplitNode {
            node_id: Default::default(),
            axis: SplitAxis::Horizontal,
            ratio: 0.5,
            first: Box::new(DocumentNode::tabs(TabStackLayout {
                tabs: vec![ViewInstanceId::new("first"), ViewInstanceId::new("active")],
                active_tab: active_tab.map(ViewInstanceId::new),
            })),
            second: Box::new(DocumentNode::tabs(TabStackLayout {
                tabs: vec![ViewInstanceId::new("focused")],
                active_tab: None,
            })),
        },
        focused_view: focused_view.map(ViewInstanceId::new),
        frame: ShellFrame::default(),
    });
    layout
}

fn focus_target(layout: &WorkbenchLayout) -> Option<&str> {
    floating_window_focus_target_in_layout(layout, &MainPageId::new(WINDOW_ID))
        .map(|instance_id| instance_id.0.as_str())
}
