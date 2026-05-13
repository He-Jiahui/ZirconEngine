use crate::ui::{layout::compute_layout_tree, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{
        AxisConstraint, BoxConstraints, StretchMode, UiContainerKind, UiFrame, UiGridBoxConfig,
        UiLinearBoxConfig, UiSize, UiSizeBoxConfig, UiWrapBoxConfig,
    },
    tree::{UiTemplateNodeMetadata, UiTree, UiTreeNode},
};

#[test]
fn layout_pass_routes_supported_containers_through_taffy_arrange() {
    let arrange = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/layout/pass/arrange.rs"),
    )
    .expect("read arrange pass");
    let taffy_arrange = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/ui/layout/pass/taffy_arrange.rs"),
    )
    .expect("read taffy arrange pass");

    assert!(arrange.contains("try_arrange_taffy_owned_children("));
    assert!(taffy_arrange.contains("UiContainerKind::HorizontalBox(_)"));
    assert!(taffy_arrange.contains("UiContainerKind::VerticalBox(_)"));
    assert!(taffy_arrange.contains("UiContainerKind::WrapBox(_)"));
    assert!(taffy_arrange.contains("UiContainerKind::GridBox(_)"));
    assert!(!taffy_arrange.contains("template_metadata.is_some()"));
    assert!(!taffy_arrange.contains("UiContainerKind::Overlay"));
    assert!(!taffy_arrange.contains("UiContainerKind::ScrollableBox"));
    assert!(!taffy_arrange.contains("UiContainerKind::SizeBox"));
}

#[test]
fn taffy_layout_pass_arranges_linear_wrap_and_grid_containers() {
    let mut linear = tree_with_root(
        1,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 5.0 }),
    );
    insert_child(&mut linear, 1, fixed_node(2, Some(50.0), None));
    insert_child(&mut linear, 1, node(3));
    insert_child(&mut linear, 1, fixed_node(4, Some(25.0), None));
    compute_layout_tree(&mut linear, UiSize::new(200.0, 40.0)).unwrap();
    assert_eq!(frame(&linear, 2), UiFrame::new(0.0, 0.0, 50.0, 40.0));
    assert_eq!(frame(&linear, 3), UiFrame::new(55.0, 0.0, 115.0, 40.0));
    assert_eq!(frame(&linear, 4), UiFrame::new(175.0, 0.0, 25.0, 40.0));

    let mut wrap = tree_with_root(
        10,
        UiContainerKind::WrapBox(UiWrapBoxConfig {
            horizontal_gap: 4.0,
            vertical_gap: 5.0,
            item_min_width: 30.0,
        }),
    );
    insert_child(&mut wrap, 10, fixed_node(11, Some(40.0), Some(10.0)));
    insert_child(&mut wrap, 10, fixed_node(12, Some(40.0), Some(10.0)));
    insert_child(&mut wrap, 10, fixed_node(13, Some(40.0), Some(10.0)));
    compute_layout_tree(&mut wrap, UiSize::new(90.0, 40.0)).unwrap();
    assert_eq!(frame(&wrap, 11), UiFrame::new(0.0, 0.0, 40.0, 10.0));
    assert_eq!(frame(&wrap, 12), UiFrame::new(44.0, 0.0, 40.0, 10.0));
    assert_eq!(frame(&wrap, 13), UiFrame::new(0.0, 15.0, 40.0, 10.0));

    let mut grid = tree_with_root(
        20,
        UiContainerKind::GridBox(UiGridBoxConfig {
            columns: 2,
            rows: 1,
            column_gap: 4.0,
            row_gap: 0.0,
        }),
    );
    insert_child(&mut grid, 20, node(21));
    insert_child(&mut grid, 20, node(22));
    compute_layout_tree(&mut grid, UiSize::new(104.0, 20.0)).unwrap();
    assert_eq!(frame(&grid, 21), UiFrame::new(0.0, 0.0, 50.0, 20.0));
    assert_eq!(frame(&grid, 22), UiFrame::new(54.0, 0.0, 50.0, 20.0));
}

#[test]
fn taffy_layout_pass_accepts_template_metadata_from_v2_assets() {
    let mut tree = tree_with_root(
        100,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 4.0 }),
    );
    insert_child(
        &mut tree,
        100,
        fixed_node(101, Some(44.0), None).with_template_metadata(template_metadata("Button")),
    );
    insert_child(
        &mut tree,
        100,
        node(102).with_template_metadata(template_metadata("Label")),
    );

    compute_layout_tree(&mut tree, UiSize::new(160.0, 32.0)).unwrap();

    assert_eq!(frame(&tree, 101), UiFrame::new(0.0, 0.0, 44.0, 32.0));
    assert_eq!(frame(&tree, 102), UiFrame::new(48.0, 0.0, 112.0, 32.0));
}

#[test]
fn size_box_contain_aspect_ratio_stays_zircon_owned() {
    let mut tree = tree_with_root(
        200,
        UiContainerKind::SizeBox(UiSizeBoxConfig { aspect_ratio: 2.0 }),
    );
    insert_child(&mut tree, 200, node(201));

    compute_layout_tree(&mut tree, UiSize::new(100.0, 100.0)).unwrap();

    assert_eq!(frame(&tree, 200), UiFrame::new(0.0, 0.0, 100.0, 100.0));
    assert_eq!(frame(&tree, 201), UiFrame::new(0.0, 25.0, 100.0, 50.0));
}

fn tree_with_root(root_id: u64, container: UiContainerKind) -> UiTree {
    let mut tree = UiTree::new(UiTreeId::new(format!("taffy.layout.{root_id}")));
    tree.insert_root(node(root_id).with_container(container));
    tree
}

fn insert_child(tree: &mut UiTree, parent_id: u64, child: UiTreeNode) {
    tree.insert_child(UiNodeId::new(parent_id), child)
        .expect("insert child");
}

fn node(id: u64) -> UiTreeNode {
    UiTreeNode::new(UiNodeId::new(id), UiNodePath::new(format!("node.{id}")))
}

fn fixed_node(id: u64, width: Option<f32>, height: Option<f32>) -> UiTreeNode {
    let mut constraints = BoxConstraints::default();
    if let Some(width) = width {
        constraints.width = fixed_axis(width);
    }
    if let Some(height) = height {
        constraints.height = fixed_axis(height);
    }
    node(id).with_constraints(constraints)
}

fn fixed_axis(value: f32) -> AxisConstraint {
    AxisConstraint {
        min: 0.0,
        max: value,
        preferred: value,
        priority: 0,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

fn frame(tree: &UiTree, id: u64) -> UiFrame {
    tree.node(UiNodeId::new(id))
        .expect("node")
        .layout_cache
        .frame
}

fn template_metadata(component: &str) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: component.to_string(),
        ..UiTemplateNodeMetadata::default()
    }
}
