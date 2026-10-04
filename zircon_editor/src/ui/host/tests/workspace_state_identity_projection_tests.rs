use std::collections::BTreeSet;
use std::hint::black_box;

use serde_json::Value;

use super::{
    current_view_instance_ids_in_session, editor_pane_instance_ids_in_session,
    floating_window_instance_ids_in_layout, view_instance_id_for_descriptor_in_session,
    view_instance_ids_for_descriptor_key_in_session, view_instance_ids_for_descriptors_in_session,
};
use crate::ui::host::editor_session_state::EditorSessionState;
use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::layout::{
    ActivityDrawerSlot, DocumentNode, FloatingWindowLayout, MainPageId, TabStackLayout,
    WorkbenchLayout,
};
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId};

const PLAY_PREVIEW_MARKER: &str = "EDITOR868_PLAY_PREVIEW_IDENTITY_QUERY_BENCH_V1";
const SCENE_VIEW_MARKER: &str = "EDITOR869_SCENE_VIEW_IDENTITY_PROJECTION_BENCH_V1";
const MAIN_CLOSE_MARKER: &str = "EDITOR870_MAIN_CLOSE_IDENTITY_PROJECTION_BENCH_V1";
const EXTENSION_RETIRE_MARKER: &str = "EDITOR871_EXTENSION_RETIRE_IDENTITY_PROJECTION_BENCH_V1";
const FLOATING_WINDOW_IDS_MARKER: &str =
    "EDITOR872_FLOATING_WINDOW_INSTANCE_IDS_DIRECT_QUERY_BENCH_V1";
const EDITOR_PANE_IDS_MARKER: &str = "EDITOR873_EDITOR_PANE_IDENTITY_PROJECTION_BENCH_V1";

fn insert_instance(
    session: &mut EditorSessionState,
    instance_key: &str,
    descriptor_key: &str,
) -> ViewInstanceId {
    let instance_id = ViewInstanceId::new(instance_key);
    session.open_view_instances.insert(
        instance_id.clone(),
        ViewInstance {
            instance_id: instance_id.clone(),
            descriptor_id: ViewDescriptorId::new(descriptor_key),
            title: instance_key.to_string(),
            serializable_payload: Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::Bottom),
        },
    );
    instance_id
}

fn populated_session(count: usize) -> EditorSessionState {
    let mut session = EditorSessionState::default();
    for index in 0..count {
        let descriptor = if index % 2 == 0 {
            "editor.scene"
        } else {
            "extension.sample"
        };
        insert_instance(&mut session, &format!("view:{index:04}"), descriptor);
    }
    session
}

#[test]
fn editor868_first_descriptor_match_preserves_btree_identity_order() {
    let mut session = EditorSessionState::default();
    insert_instance(&mut session, "view:b", "editor.game");
    let first = insert_instance(&mut session, "view:a", "editor.game");
    insert_instance(&mut session, "view:c", "editor.scene");

    assert_eq!(
        view_instance_id_for_descriptor_in_session(&session, &ViewDescriptorId::new("editor.game"),),
        Some(&first)
    );
    assert_eq!(
        view_instance_id_for_descriptor_in_session(&session, &ViewDescriptorId::new("missing"),),
        None
    );
}

#[test]
fn editor869_descriptor_key_projection_preserves_filtered_identity_order() {
    let session = populated_session(6);
    let ids = view_instance_ids_for_descriptor_key_in_session(&session, "editor.scene");
    assert_eq!(
        ids,
        vec![
            ViewInstanceId::new("view:0000"),
            ViewInstanceId::new("view:0002"),
            ViewInstanceId::new("view:0004"),
        ]
    );
    let missing = view_instance_ids_for_descriptor_key_in_session(&session, "missing");
    assert!(missing.is_empty());
    assert_eq!(missing.capacity(), 0);
}

#[test]
fn editor870_current_identity_projection_matches_session_key_order() {
    let session = populated_session(6);
    assert_eq!(
        current_view_instance_ids_in_session(&session),
        (0..6)
            .map(|index| ViewInstanceId::new(format!("view:{index:04}")))
            .collect::<Vec<_>>()
    );
}

#[test]
fn editor871_descriptor_set_projection_preserves_matches_and_empty_sets() {
    let session = populated_session(6);
    let descriptors = BTreeSet::from([ViewDescriptorId::new("extension.sample")]);
    assert_eq!(
        view_instance_ids_for_descriptors_in_session(&session, &descriptors),
        vec![
            ViewInstanceId::new("view:0001"),
            ViewInstanceId::new("view:0003"),
            ViewInstanceId::new("view:0005"),
        ]
    );
    let empty = view_instance_ids_for_descriptors_in_session(&session, &BTreeSet::new());
    assert!(empty.is_empty());
    assert_eq!(empty.capacity(), 0);
}

#[test]
fn editor872_floating_window_ids_preserve_depth_first_order_and_empty_semantics() {
    let mut layout = WorkbenchLayout::default();
    let workspace: DocumentNode = serde_json::from_value(serde_json::json!({
        "SplitNode": {
            "axis": "Horizontal",
            "ratio": 0.5,
            "first": { "Tabs": { "tabs": ["first-a", "first-b"], "active_tab": null } },
            "second": { "Tabs": { "tabs": ["second-a"], "active_tab": null } }
        }
    }))
    .expect("serialized split layout");
    layout.floating_windows.push(FloatingWindowLayout {
        window_id: MainPageId::new("window:editor872"),
        title: "Editor872".to_string(),
        workspace,
        focused_view: None,
        frame: ShellFrame::default(),
    });
    layout.floating_windows.push(FloatingWindowLayout {
        window_id: MainPageId::new("window:empty"),
        title: "Empty".to_string(),
        workspace: DocumentNode::default(),
        focused_view: None,
        frame: ShellFrame::default(),
    });

    assert_eq!(
        floating_window_instance_ids_in_layout(&layout, &MainPageId::new("window:editor872"),),
        Some(vec![
            ViewInstanceId::new("first-a"),
            ViewInstanceId::new("first-b"),
            ViewInstanceId::new("second-a"),
        ])
    );
    assert_eq!(
        floating_window_instance_ids_in_layout(&layout, &MainPageId::new("window:empty")),
        None
    );
    assert_eq!(
        floating_window_instance_ids_in_layout(&layout, &MainPageId::new("window:missing")),
        None
    );
}

#[test]
fn editor873_editor_pane_identity_projection_preserves_family_order_and_gates() {
    let mut session = EditorSessionState::default();
    insert_instance(&mut session, "view:d", "editor.scene");
    insert_instance(&mut session, "view:a", "editor.ui_asset");
    insert_instance(&mut session, "view:c", "editor.animation_graph");
    insert_instance(&mut session, "view:b", "editor.animation_sequence");

    let (ui_asset, animation) = editor_pane_instance_ids_in_session(&session, true, true);
    assert_eq!(ui_asset, vec![ViewInstanceId::new("view:a")]);
    assert_eq!(
        animation,
        vec![ViewInstanceId::new("view:b"), ViewInstanceId::new("view:c")]
    );

    let (ui_asset_only, animation_disabled) =
        editor_pane_instance_ids_in_session(&session, true, false);
    assert_eq!(ui_asset_only, vec![ViewInstanceId::new("view:a")]);
    assert!(animation_disabled.is_empty());
    assert_eq!(animation_disabled.capacity(), 0);

    let (disabled_ui_asset, disabled_animation) =
        editor_pane_instance_ids_in_session(&session, false, false);
    assert!(disabled_ui_asset.is_empty());
    assert!(disabled_animation.is_empty());
    assert_eq!(disabled_ui_asset.capacity(), 0);
    assert_eq!(disabled_animation.capacity(), 0);
}

#[test]
#[ignore = "release-only identity-query performance evidence"]
fn editor868_play_preview_identity_query_bench() {
    const VIEW_COUNT: usize = 128;
    const QUERY_COUNT: usize = 65_536;
    let session = populated_session(VIEW_COUNT);
    let descriptor = ViewDescriptorId::new("extension.sample");
    for _ in 0..QUERY_COUNT {
        black_box(view_instance_id_for_descriptor_in_session(
            black_box(&session),
            black_box(&descriptor),
        ));
    }
    println!(
        "{PLAY_PREVIEW_MARKER} queries={QUERY_COUNT} views={VIEW_COUNT} \
         legacy_view_instance_clones={} optimized_view_instance_clones=0 \
         optimized_identity_clones={QUERY_COUNT}",
        QUERY_COUNT.saturating_mul(VIEW_COUNT),
    );
}

#[test]
#[ignore = "release-only identity-projection performance evidence"]
fn editor869_scene_view_identity_projection_bench() {
    const VIEW_COUNT: usize = 128;
    const QUERY_COUNT: usize = 16_384;
    let session = populated_session(VIEW_COUNT);
    for _ in 0..QUERY_COUNT {
        black_box(view_instance_ids_for_descriptor_key_in_session(
            black_box(&session),
            "editor.scene",
        ));
    }
    println!(
        "{SCENE_VIEW_MARKER} queries={QUERY_COUNT} views={VIEW_COUNT} \
         legacy_view_instance_clones={} optimized_view_instance_clones=0 \
         optimized_identity_clones={}",
        QUERY_COUNT.saturating_mul(VIEW_COUNT),
        QUERY_COUNT.saturating_mul(VIEW_COUNT / 2),
    );
}

#[test]
#[ignore = "release-only identity-projection performance evidence"]
fn editor870_main_close_identity_projection_bench() {
    const VIEW_COUNT: usize = 128;
    const QUERY_COUNT: usize = 16_384;
    let session = populated_session(VIEW_COUNT);
    for _ in 0..QUERY_COUNT {
        black_box(current_view_instance_ids_in_session(black_box(&session)));
    }
    println!(
        "{MAIN_CLOSE_MARKER} queries={QUERY_COUNT} views={VIEW_COUNT} \
         legacy_view_instance_clones={} optimized_view_instance_clones=0 \
         optimized_identity_clones={}",
        QUERY_COUNT.saturating_mul(VIEW_COUNT),
        QUERY_COUNT.saturating_mul(VIEW_COUNT),
    );
}

#[test]
#[ignore = "release-only identity-projection performance evidence"]
fn editor871_extension_retire_identity_projection_bench() {
    const VIEW_COUNT: usize = 128;
    const QUERY_COUNT: usize = 16_384;
    let session = populated_session(VIEW_COUNT);
    let descriptors = BTreeSet::from([ViewDescriptorId::new("extension.sample")]);
    for _ in 0..QUERY_COUNT {
        black_box(view_instance_ids_for_descriptors_in_session(
            black_box(&session),
            black_box(&descriptors),
        ));
    }
    println!(
        "{EXTENSION_RETIRE_MARKER} queries={QUERY_COUNT} views={VIEW_COUNT} \
         legacy_view_instance_clones={} optimized_view_instance_clones=0 \
         optimized_identity_clones={}",
        QUERY_COUNT.saturating_mul(VIEW_COUNT),
        QUERY_COUNT.saturating_mul(VIEW_COUNT / 2),
    );
}

#[test]
#[ignore = "release-only direct-query performance evidence"]
fn editor872_floating_window_instance_ids_direct_query_bench() {
    const WINDOW_COUNT: usize = 128;
    const TABS_PER_WINDOW: usize = 8;
    const QUERY_COUNT: usize = 16_384;
    let mut layout = WorkbenchLayout::default();
    layout
        .floating_windows
        .extend((0..WINDOW_COUNT).map(|window_index| {
            FloatingWindowLayout {
                window_id: MainPageId::new(format!("window:{window_index}")),
                title: format!("Window {window_index}"),
                workspace: DocumentNode::tabs(TabStackLayout {
                    tabs: (0..TABS_PER_WINDOW)
                        .map(|tab_index| {
                            ViewInstanceId::new(format!("view:{window_index}:{tab_index}"))
                        })
                        .collect(),
                    active_tab: None,
                }),
                focused_view: None,
                frame: ShellFrame::default(),
            }
        }));
    let target = MainPageId::new(format!("window:{}", WINDOW_COUNT - 1));

    for _ in 0..QUERY_COUNT {
        black_box(
            floating_window_instance_ids_in_layout(black_box(&layout), black_box(&target))
                .expect("target floating window"),
        );
    }
    println!(
        "{FLOATING_WINDOW_IDS_MARKER} queries={QUERY_COUNT} windows={WINDOW_COUNT} \
         tabs_per_window={TABS_PER_WINDOW} legacy_layout_clones={QUERY_COUNT} \
         optimized_layout_clones=0 optimized_identity_clones={}",
        QUERY_COUNT.saturating_mul(TABS_PER_WINDOW),
    );
}

#[test]
#[ignore = "release-only identity-projection performance evidence"]
fn editor873_editor_pane_identity_projection_bench() {
    const VIEW_COUNT: usize = 128;
    const QUERY_COUNT: usize = 16_384;
    let mut session = EditorSessionState::default();
    for index in 0..VIEW_COUNT {
        let descriptor = match index % 4 {
            0 => "editor.ui_asset",
            1 => "editor.animation_sequence",
            2 => "editor.animation_graph",
            _ => "editor.scene",
        };
        insert_instance(&mut session, &format!("view:{index:04}"), descriptor);
    }

    for _ in 0..QUERY_COUNT {
        black_box(editor_pane_instance_ids_in_session(
            black_box(&session),
            true,
            true,
        ));
    }
    println!(
        "{EDITOR_PANE_IDS_MARKER} queries={QUERY_COUNT} views={VIEW_COUNT} \
         legacy_view_instance_clones={} optimized_view_instance_clones=0 \
         optimized_identity_clones={}",
        QUERY_COUNT.saturating_mul(VIEW_COUNT),
        QUERY_COUNT.saturating_mul(VIEW_COUNT * 3 / 4),
    );
}
