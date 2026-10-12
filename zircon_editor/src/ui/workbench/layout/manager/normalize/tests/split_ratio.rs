use super::*;
use crate::ui::workbench::layout::{SplitAxis, TabStackLayout};
use crate::ui::workbench::view::{ViewInstanceId, ViewRegistry};

fn tabs(id: &str) -> DocumentNode {
    let id = ViewInstanceId::new(id);
    DocumentNode::tabs(TabStackLayout {
        tabs: vec![id.clone()],
        active_tab: Some(id),
    })
}

fn nested_layout(outer_ratio: f32, inner_ratio: f32) -> WorkbenchLayout {
    let mut layout = WorkbenchLayout::default();
    *layout.ensure_workbench_content_workspace() = DocumentNode::SplitNode {
        node_id: Default::default(),
        axis: SplitAxis::Horizontal,
        ratio: outer_ratio,
        first: Box::new(tabs("editor.scene#left")),
        second: Box::new(DocumentNode::SplitNode {
            node_id: Default::default(),
            axis: SplitAxis::Vertical,
            ratio: inner_ratio,
            first: Box::new(tabs("editor.game#top")),
            second: Box::new(tabs("editor.material#bottom")),
        }),
    };
    layout
}

#[test]
fn astra_editor_layout_valid_split_tree_and_ratios_survive_normalization_exactly() {
    let saved = serde_json::to_string(&nested_layout(0.3, 0.8)).expect("layout serializes");
    let mut layout: WorkbenchLayout = serde_json::from_str(&saved).expect("saved layout loads");
    let expected = layout.clone();

    LayoutManager.normalize(&mut layout, &ViewRegistry::default());

    assert_eq!(layout, expected);
}

#[test]
fn astra_editor_layout_invalid_saved_ratios_are_repaired_recursively() {
    let mut layout = nested_layout(f32::NAN, 4.0);

    LayoutManager.normalize(&mut layout, &ViewRegistry::default());

    let DocumentNode::SplitNode {
        axis,
        ratio,
        first,
        second,
        ..
    } = layout.ensure_workbench_content_workspace()
    else {
        panic!("outer split topology must survive normalization");
    };
    assert_eq!(*axis, SplitAxis::Horizontal);
    assert_eq!(*ratio, DEFAULT_SPLIT_RATIO);
    assert!(matches!(first.as_ref(), DocumentNode::Tabs(_)));
    let DocumentNode::SplitNode {
        axis: inner_axis,
        ratio: inner_ratio,
        first: inner_first,
        second: inner_second,
        ..
    } = second.as_ref()
    else {
        panic!("nested split topology must survive normalization");
    };
    assert_eq!(*inner_axis, SplitAxis::Vertical);
    assert_eq!(*inner_ratio, MAX_SPLIT_RATIO);
    assert!(matches!(inner_first.as_ref(), DocumentNode::Tabs(_)));
    assert!(matches!(inner_second.as_ref(), DocumentNode::Tabs(_)));
}

#[test]
fn astra_editor_layout_negative_saved_ratio_clamps_without_reordering_leaves() {
    let mut layout = nested_layout(-2.0, 0.75);

    LayoutManager.normalize(&mut layout, &ViewRegistry::default());

    let DocumentNode::SplitNode {
        ratio,
        first,
        second,
        ..
    } = layout.ensure_workbench_content_workspace()
    else {
        panic!("outer split topology must survive normalization");
    };
    assert_eq!(*ratio, MIN_SPLIT_RATIO);
    assert!(
        matches!(first.as_ref(), DocumentNode::Tabs(stack) if stack.tabs[0].0 == "editor.scene#left")
    );
    assert!(matches!(second.as_ref(), DocumentNode::SplitNode { ratio, .. } if *ratio == 0.75));
}
