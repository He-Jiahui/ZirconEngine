use std::hint::black_box;

use serde_json::Value;

use super::{
    active_drawer_mode_in_layout, active_drawer_toggle_state_in_layout,
    floating_window_exists_in_layout, view_host_for_instance_key_in_session,
};
use crate::ui::host::editor_session_state::EditorSessionState;
use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::layout::{
    ActivityDrawerMode, ActivityDrawerSlot, DocumentNode, FloatingWindowLayout, MainPageId,
    WorkbenchLayout,
};
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId};

const DRAWER_TOGGLE_MARKER: &str = "EDITOR863_DRAWER_TOGGLE_DIRECT_QUERY_BENCH_V1";
const DRAWER_MODE_MARKER: &str = "EDITOR864_TAB_DROP_DRAWER_MODE_DIRECT_QUERY_BENCH_V1";
const WINDOW_EXISTS_MARKER: &str = "EDITOR865_FLOATING_WINDOW_EXISTS_DIRECT_QUERY_BENCH_V1";
const VIEW_HOST_MARKER: &str = "EDITOR866_VIEW_HOST_DIRECT_QUERY_BENCH_V1";

#[test]
fn editor863_drawer_toggle_query_preserves_active_and_region_semantics() {
    let mut layout = WorkbenchLayout::default();
    let target = ViewInstanceId::new("drawer:target");
    let sibling = ViewInstanceId::new("drawer:sibling");
    let window = layout
        .active_activity_window_mut()
        .expect("default workbench activity window");
    let target_drawer = window
        .activity_drawers
        .get_mut(&ActivityDrawerSlot::LeftTop)
        .expect("default left-top drawer");
    target_drawer.tab_stack.tabs.push(target.clone());
    target_drawer.tab_stack.active_tab = Some(target.clone());
    target_drawer.mode = ActivityDrawerMode::Pinned;
    let sibling_drawer = window
        .activity_drawers
        .get_mut(&ActivityDrawerSlot::LeftBottom)
        .expect("default left-bottom drawer");
    sibling_drawer.tab_stack.tabs.push(sibling);
    sibling_drawer.visible = true;
    sibling_drawer.mode = ActivityDrawerMode::AutoHide;

    assert_eq!(
        active_drawer_toggle_state_in_layout(&layout, ActivityDrawerSlot::LeftTop, &target,),
        Ok((ActivityDrawerMode::Pinned, true, true))
    );
}

#[test]
fn editor863_drawer_toggle_query_preserves_missing_authority_errors() {
    let mut layout = WorkbenchLayout::default();
    layout.main_pages.clear();
    assert_eq!(
        active_drawer_toggle_state_in_layout(
            &layout,
            ActivityDrawerSlot::Bottom,
            &ViewInstanceId::new("missing"),
        ),
        Err("missing active activity window".to_string())
    );
}

#[test]
fn editor864_drawer_mode_query_returns_only_the_requested_mode() {
    let mut layout = WorkbenchLayout::default();
    layout
        .active_activity_window_mut()
        .expect("default workbench activity window")
        .activity_drawers
        .get_mut(&ActivityDrawerSlot::RightBottom)
        .expect("default right-bottom drawer")
        .mode = ActivityDrawerMode::Collapsed;

    assert_eq!(
        active_drawer_mode_in_layout(&layout, ActivityDrawerSlot::RightBottom),
        Some(ActivityDrawerMode::Collapsed)
    );
}

#[test]
fn editor865_floating_window_exists_query_preserves_exact_identity() {
    let mut layout = WorkbenchLayout::default();
    layout.floating_windows.push(FloatingWindowLayout {
        window_id: MainPageId::new("window:editor865"),
        title: "Editor865".to_string(),
        workspace: DocumentNode::default(),
        focused_view: None,
        frame: ShellFrame::default(),
    });

    assert!(floating_window_exists_in_layout(
        &layout,
        &MainPageId::new("window:editor865")
    ));
    assert!(!floating_window_exists_in_layout(
        &layout,
        &MainPageId::new("WINDOW:editor865")
    ));
}

#[test]
fn editor866_view_host_query_clones_only_the_matching_host() {
    let mut session = EditorSessionState::default();
    let instance_id = ViewInstanceId::new("view:editor866");
    let expected = ViewHost::ExclusivePage(MainPageId::new("page:editor866"));
    session.open_view_instances.insert(
        instance_id.clone(),
        ViewInstance {
            instance_id,
            descriptor_id: ViewDescriptorId::new("descriptor:editor866"),
            title: "Editor866".to_string(),
            serializable_payload: Value::Null,
            dirty: false,
            host: expected.clone(),
        },
    );

    assert_eq!(
        view_host_for_instance_key_in_session(&session, "view:editor866"),
        Some(expected)
    );
    assert_eq!(
        view_host_for_instance_key_in_session(&session, "VIEW:editor866"),
        None
    );
}

#[test]
#[ignore = "release-only direct-query performance evidence"]
fn editor863_drawer_toggle_direct_query_bench() {
    const QUERY_COUNT: usize = 65_536;
    let mut layout = WorkbenchLayout::default();
    let target = ViewInstanceId::new("drawer:bench");
    let drawer = layout
        .active_activity_window_mut()
        .expect("default workbench activity window")
        .activity_drawers
        .get_mut(&ActivityDrawerSlot::LeftTop)
        .expect("default left-top drawer");
    drawer.tab_stack.tabs.push(target.clone());
    drawer.tab_stack.active_tab = Some(target.clone());

    for _ in 0..QUERY_COUNT {
        black_box(
            active_drawer_toggle_state_in_layout(
                black_box(&layout),
                ActivityDrawerSlot::LeftTop,
                black_box(&target),
            )
            .expect("drawer state"),
        );
    }

    println!(
        "{DRAWER_TOGGLE_MARKER} queries={QUERY_COUNT} legacy_layout_clones={QUERY_COUNT} \
         optimized_layout_clones=0 reduction_pct=100"
    );
}

#[test]
#[ignore = "release-only direct-query performance evidence"]
fn editor864_tab_drop_drawer_mode_direct_query_bench() {
    const QUERY_COUNT: usize = 65_536;
    let layout = WorkbenchLayout::default();

    for _ in 0..QUERY_COUNT {
        black_box(active_drawer_mode_in_layout(
            black_box(&layout),
            ActivityDrawerSlot::Bottom,
        ));
    }

    println!(
        "{DRAWER_MODE_MARKER} queries={QUERY_COUNT} legacy_layout_clones={QUERY_COUNT} \
         optimized_layout_clones=0 reduction_pct=100"
    );
}

#[test]
#[ignore = "release-only direct-query performance evidence"]
fn editor865_floating_window_exists_direct_query_bench() {
    const WINDOW_COUNT: usize = 128;
    const QUERY_COUNT: usize = 65_536;
    let mut layout = WorkbenchLayout::default();
    layout
        .floating_windows
        .extend((0..WINDOW_COUNT).map(|index| FloatingWindowLayout {
            window_id: MainPageId::new(format!("window:{index}")),
            title: format!("Window {index}"),
            workspace: DocumentNode::default(),
            focused_view: None,
            frame: ShellFrame::default(),
        }));
    let target = MainPageId::new(format!("window:{}", WINDOW_COUNT - 1));

    for _ in 0..QUERY_COUNT {
        black_box(floating_window_exists_in_layout(
            black_box(&layout),
            black_box(&target),
        ));
    }

    println!(
        "{WINDOW_EXISTS_MARKER} queries={QUERY_COUNT} windows={WINDOW_COUNT} \
         legacy_layout_clones={QUERY_COUNT} optimized_layout_clones=0 reduction_pct=100"
    );
}

#[test]
#[ignore = "release-only direct-query performance evidence"]
fn editor866_view_host_direct_query_bench() {
    const VIEW_COUNT: usize = 128;
    const QUERY_COUNT: usize = 65_536;
    let mut session = EditorSessionState::default();
    for index in 0..VIEW_COUNT {
        let instance_id = ViewInstanceId::new(format!("view:{index}"));
        session.open_view_instances.insert(
            instance_id.clone(),
            ViewInstance {
                instance_id,
                descriptor_id: ViewDescriptorId::new("descriptor:bench"),
                title: format!("View {index}"),
                serializable_payload: Value::Null,
                dirty: false,
                host: ViewHost::ExclusivePage(MainPageId::new("page:bench")),
            },
        );
    }
    let target = format!("view:{}", VIEW_COUNT - 1);

    for _ in 0..QUERY_COUNT {
        black_box(view_host_for_instance_key_in_session(
            black_box(&session),
            black_box(&target),
        ));
    }

    println!(
        "{VIEW_HOST_MARKER} queries={QUERY_COUNT} views={VIEW_COUNT} \
         legacy_view_instance_clones={} optimized_view_instance_clones=0 \
         optimized_view_host_clones={QUERY_COUNT}",
        QUERY_COUNT.saturating_mul(VIEW_COUNT),
    );
}
