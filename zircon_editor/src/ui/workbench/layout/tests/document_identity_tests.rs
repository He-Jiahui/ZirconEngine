use super::*;
use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::view::{ViewInstanceId, ViewRegistry};

#[test]
fn astra_nested_workspace_reopens_exact_node_ids_ratios_and_content() {
    let mut layout = WorkbenchLayout::default();
    *layout.ensure_workbench_content_workspace() = split(
        0.63,
        leaf("scene"),
        split(0.27, leaf("material"), leaf("console")),
    );
    let encoded = serde_json::to_string(&layout).unwrap();
    let restored: WorkbenchLayout = serde_json::from_str(&encoded).unwrap();
    assert_eq!(layout, restored);
    assert_eq!(encoded, serde_json::to_string(&restored).unwrap());
}

#[test]
fn astra_legacy_nested_workspace_migrates_once_without_repartitioning() {
    let mut layout = WorkbenchLayout::default();
    *layout.ensure_workbench_content_workspace() = split(0.63, leaf("scene"), leaf("material"));
    let mut saved = serde_json::to_value(&layout).unwrap();
    remove_ids(&mut saved);
    let migrated: WorkbenchLayout = serde_json::from_value(saved.clone()).unwrap();
    let migrated_again: WorkbenchLayout = serde_json::from_value(saved).unwrap();
    assert_eq!(migrated, migrated_again);
    let mut ids = std::collections::HashSet::new();
    let root = migrated
        .content_workspace_for_page(&MainPageId::workbench())
        .unwrap();
    collect_ids(root, &mut ids);
    assert_eq!(ids.len(), 3);
    let DocumentNode::SplitNode {
        ratio,
        first,
        second,
        ..
    } = root
    else {
        unreachable!()
    };
    assert_eq!(*ratio, 0.63);
    assert!(first.contains(&ViewInstanceId::new("scene")));
    assert!(second.contains(&ViewInstanceId::new("material")));
    let saved = serde_json::to_string(&migrated).unwrap();
    assert_eq!(
        migrated,
        serde_json::from_str::<WorkbenchLayout>(&saved).unwrap()
    );
}

#[test]
fn astra_duplicate_saved_node_ids_repair_only_later_owners() {
    let original = leaf("scene");
    let original_id = original.node_id();
    let mut layout = WorkbenchLayout::default();
    *layout.ensure_workbench_content_workspace() = split(0.63, original.clone(), original);
    let saved = serde_json::to_value(&layout).unwrap();
    let restored: WorkbenchLayout = serde_json::from_value(saved.clone()).unwrap();
    let restored_again: WorkbenchLayout = serde_json::from_value(saved).unwrap();
    assert_eq!(restored, restored_again);
    let root = restored
        .content_workspace_for_page(&MainPageId::workbench())
        .unwrap();
    let DocumentNode::SplitNode {
        first,
        second,
        ratio,
        ..
    } = root
    else {
        unreachable!()
    };
    assert_eq!(first.node_id(), original_id);
    assert_ne!(second.node_id(), original_id);
    assert_eq!(*ratio, 0.63);
    let saved = serde_json::to_string(&restored).unwrap();
    assert_eq!(
        restored,
        serde_json::from_str::<WorkbenchLayout>(&saved).unwrap()
    );
}

#[test]
fn astra_legacy_repair_preserves_later_saved_id_across_floating_windows() {
    const FIRST_MIGRATED_ID: &str = "9d7e4bcc-2cd7-8d6d-8000-000000000001";
    let saved_id: DocumentNodeId =
        serde_json::from_value(serde_json::json!(FIRST_MIGRATED_ID)).unwrap();
    let mut floating_leaf = leaf("floating");
    let DocumentNode::Tabs(floating_tabs) = &mut floating_leaf else {
        unreachable!()
    };
    floating_tabs.node_id = saved_id;

    let mut layout = WorkbenchLayout::default();
    *layout.ensure_workbench_content_workspace() =
        split(0.37, leaf("main-left"), leaf("main-right"));
    let floating_window = FloatingWindowLayout {
        window_id: MainPageId::new("floating:legacy-id"),
        title: "Floating".to_string(),
        workspace: floating_leaf,
        focused_view: None,
        frame: ShellFrame::default(),
    };
    layout.floating_windows.push(floating_window.clone());
    layout.floating_windows.push(FloatingWindowLayout {
        window_id: MainPageId::new("floating:duplicate-id"),
        ..floating_window
    });
    let mut saved = serde_json::to_value(&layout).unwrap();
    remove_ids_except(&mut saved, Some(FIRST_MIGRATED_ID));

    let restored: WorkbenchLayout = serde_json::from_value(saved.clone()).unwrap();
    let restored_again: WorkbenchLayout = serde_json::from_value(saved).unwrap();
    assert_eq!(restored, restored_again);
    assert_eq!(restored.floating_windows[0].workspace.node_id(), saved_id);
    assert_ne!(restored.floating_windows[1].workspace.node_id(), saved_id);

    let mut ids = std::collections::HashSet::new();
    collect_ids(
        restored
            .content_workspace_for_page(&MainPageId::workbench())
            .unwrap(),
        &mut ids,
    );
    for window in &restored.floating_windows {
        collect_ids(&window.workspace, &mut ids);
    }
    assert_eq!(ids.len(), 5);
    assert_eq!(
        restored,
        serde_json::from_str::<WorkbenchLayout>(&serde_json::to_string(&restored).unwrap())
            .unwrap()
    );
}

#[test]
fn astra_migrated_subtree_merge_preserves_saved_id_and_stays_stable_on_normalize() {
    const FIRST_MIGRATED_ID: &str = "9d7e4bcc-2cd7-8d6d-8000-000000000001";
    let saved_id: DocumentNodeId =
        serde_json::from_value(serde_json::json!(FIRST_MIGRATED_ID)).unwrap();
    let migrated: DocumentNode = serde_json::from_value(serde_json::json!({
        "Tabs": {"tabs": ["legacy"], "active_tab": "legacy"}
    }))
    .unwrap();
    assert_eq!(migrated.node_id(), saved_id);

    let mut source = WorkbenchLayout::default();
    *source.ensure_workbench_content_workspace() = migrated;
    let mut destination = WorkbenchLayout::default();
    *destination.ensure_workbench_content_workspace() = source
        .content_workspace_for_page(&MainPageId::workbench())
        .unwrap()
        .clone();
    let mut saved_leaf = leaf("saved-floating");
    let DocumentNode::Tabs(saved_tabs) = &mut saved_leaf else {
        unreachable!()
    };
    saved_tabs.node_id = saved_id;
    destination.floating_windows.push(FloatingWindowLayout {
        window_id: MainPageId::new("floating:saved-id"),
        title: "Saved".to_string(),
        workspace: saved_leaf,
        focused_view: None,
        frame: ShellFrame::default(),
    });

    LayoutManager.normalize(&mut destination, &ViewRegistry::default());
    let migrated_id = destination
        .content_workspace_for_page(&MainPageId::workbench())
        .unwrap()
        .node_id();
    assert_ne!(migrated_id, saved_id);
    assert_eq!(
        destination.floating_windows[0].workspace.node_id(),
        saved_id
    );
    let normalized_once = destination.clone();
    LayoutManager.normalize(&mut destination, &ViewRegistry::default());
    assert_eq!(destination, normalized_once);
    assert_eq!(
        destination
            .content_workspace_for_page(&MainPageId::workbench())
            .unwrap()
            .node_id(),
        migrated_id
    );
}

fn leaf(name: &str) -> DocumentNode {
    DocumentNode::tabs(TabStackLayout {
        tabs: vec![ViewInstanceId::new(name)],
        active_tab: Some(ViewInstanceId::new(name)),
    })
}

fn split(ratio: f32, first: DocumentNode, second: DocumentNode) -> DocumentNode {
    DocumentNode::SplitNode {
        node_id: DocumentNodeId::default(),
        axis: SplitAxis::Horizontal,
        ratio,
        first: Box::new(first),
        second: Box::new(second),
    }
}

fn collect_ids(node: &DocumentNode, ids: &mut std::collections::HashSet<DocumentNodeId>) {
    assert!(!node.node_id().is_nil());
    assert!(ids.insert(node.node_id()));
    if let DocumentNode::SplitNode { first, second, .. } = node {
        collect_ids(first, ids);
        collect_ids(second, ids);
    }
}

fn remove_ids(value: &mut serde_json::Value) {
    remove_ids_except(value, None);
}

fn remove_ids_except(value: &mut serde_json::Value, keep: Option<&str>) {
    match value {
        serde_json::Value::Object(fields) => {
            if fields.get("node_id").and_then(serde_json::Value::as_str) != keep {
                fields.remove("node_id");
            }
            for child in fields.values_mut() {
                remove_ids_except(child, keep);
            }
        }
        serde_json::Value::Array(values) => {
            for child in values {
                remove_ids_except(child, keep);
            }
        }
        _ => {}
    }
}
