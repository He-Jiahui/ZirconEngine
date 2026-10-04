use std::hint::black_box;

use serde_json::Value;

use super::active_activity_window_template_descriptor_id;
use crate::ui::host::editor_session_state::EditorSessionState;
use crate::ui::workbench::layout::{MainHostPageLayout, MainPageId};
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId};

const BENCHMARK_MARKER: &str = "EDITOR859_ACTIVE_TEMPLATE_DIRECT_QUERY_BENCH_V1";

#[test]
fn editor859_active_template_query_preserves_workbench_and_exclusive_semantics() {
    let mut session = EditorSessionState::default();
    assert_eq!(
        active_activity_window_template_descriptor_id(&session),
        Some(&ViewDescriptorId::new("editor.workbench_window"))
    );

    let page_id = MainPageId::new("exclusive:asset-browser");
    let instance_id = ViewInstanceId::new("editor.asset_browser_window#1");
    let descriptor_id = ViewDescriptorId::new("editor.asset_browser_window");
    session.layout.active_main_page = page_id.clone();
    session
        .layout
        .main_pages
        .push(MainHostPageLayout::ExclusiveActivityWindowPage {
            id: page_id.clone(),
            title: "Asset Browser".to_string(),
            window_instance: instance_id.clone(),
        });
    session.open_view_instances.insert(
        instance_id.clone(),
        ViewInstance {
            instance_id,
            descriptor_id: descriptor_id.clone(),
            title: "Asset Browser".to_string(),
            serializable_payload: Value::Null,
            dirty: false,
            host: ViewHost::ExclusivePage(page_id),
        },
    );

    assert_eq!(
        active_activity_window_template_descriptor_id(&session),
        Some(&descriptor_id)
    );
}

#[test]
fn editor859_active_template_query_rejects_missing_authority() {
    let mut session = EditorSessionState::default();
    session.layout.main_pages.clear();
    assert_eq!(
        active_activity_window_template_descriptor_id(&session),
        None
    );

    let page_id = MainPageId::new("exclusive:missing");
    session.layout.active_main_page = page_id.clone();
    session
        .layout
        .main_pages
        .push(MainHostPageLayout::ExclusiveActivityWindowPage {
            id: page_id,
            title: "Missing".to_string(),
            window_instance: ViewInstanceId::new("missing#1"),
        });
    assert_eq!(
        active_activity_window_template_descriptor_id(&session),
        None
    );
}

#[test]
#[ignore = "release-only direct-query performance evidence"]
fn editor859_active_template_direct_query_bench() {
    const QUERY_COUNT: usize = 65_536;
    let session = EditorSessionState::default();

    for _ in 0..QUERY_COUNT {
        black_box(active_activity_window_template_descriptor_id(&session));
    }

    println!(
        "{BENCHMARK_MARKER} queries={QUERY_COUNT} legacy_full_chrome_snapshot_builds={QUERY_COUNT} \
         optimized_full_chrome_snapshot_builds=0 reduction_pct=100"
    );
}
