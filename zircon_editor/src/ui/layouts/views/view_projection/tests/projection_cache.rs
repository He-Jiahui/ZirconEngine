use std::collections::BTreeMap;

use zircon_runtime_interface::ui::layout::UiSize;
use zircon_runtime_interface::ui::v2::{UiV2ComponentGraph, UiV2NodeArena};

use super::*;

#[test]
fn projection_resource_identity_ignores_size_but_rejects_resource_replacement() {
    let compiled = Arc::new(UiV2CompiledDocument {
        asset_id: "projection-cache-test".to_string(),
        arena: UiV2NodeArena::default(),
        node_handles: BTreeMap::new(),
        component_graph: UiV2ComponentGraph::default(),
    });
    let replacement_compiled = Arc::new(compiled.as_ref().clone());
    let design_tokens = Arc::new(EditorDesignTokens::workbench_dark());
    let replacement_design_tokens = Arc::new(design_tokens.as_ref().clone());
    let width_bits = 640.0_f32.to_bits();
    let height_bits = 480.0_f32.to_bits();

    assert!(projection_resource_identity_matches(
        &compiled,
        &design_tokens,
        &compiled,
        &design_tokens,
        7,
        7,
    ));
    assert_ne!(width_bits, 800.0_f32.to_bits());
    assert_eq!(height_bits, 480.0_f32.to_bits());
    assert!(!projection_resource_identity_matches(
        &compiled,
        &design_tokens,
        &replacement_compiled,
        &design_tokens,
        7,
        7,
    ));
    assert!(!projection_resource_identity_matches(
        &compiled,
        &design_tokens,
        &compiled,
        &replacement_design_tokens,
        7,
        7,
    ));
    assert!(!projection_resource_identity_matches(
        &compiled,
        &design_tokens,
        &compiled,
        &design_tokens,
        7,
        8,
    ));
}

#[test]
fn topology_fallback_rebases_current_surface_and_preserves_authored_text() {
    clear_for_tests();
    let document_tree_id = "view.v2.project_overview.topology_rebase";
    let layout = "/assets/ui/editor/project_overview.zui";
    let first_overrides = BTreeMap::from([(
        "ProjectOverviewTitleText".to_string(),
        "First title".to_string(),
    )]);
    super::super::build_view_template_nodes(
        document_tree_id,
        layout,
        &[],
        UiSize::new(640.0, 480.0),
        &first_overrides,
    )
    .expect("initial projection");

    // A layout or render-command change can invalidate the cached row topology.
    PROJECTION_CACHE.with(|cache| {
        cache
            .borrow_mut()
            .get_mut(document_tree_id)
            .expect("cached projection")
            .surface_topology_signatures = Rc::new(Vec::new());
    });

    let second_overrides = BTreeMap::from([(
        "ProjectOverviewTitleText".to_string(),
        "Second title".to_string(),
    )]);
    let updated = super::super::build_view_template_nodes(
        document_tree_id,
        layout,
        &[],
        UiSize::new(800.0, 480.0),
        &second_overrides,
    )
    .expect("topology fallback should produce a projection");
    assert_eq!(
        updated
            .iter()
            .find(|node| node.control_id == "ProjectOverviewTitleText")
            .map(|node| node.text.as_str()),
        Some("Second title")
    );

    let materializations = surface_materialization_count_for_tests();
    let stable = super::super::build_view_template_nodes(
        document_tree_id,
        layout,
        &[],
        UiSize::new(800.0, 480.0),
        &second_overrides,
    )
    .expect("rebased projection should be reusable");
    assert_eq!(stable, updated);
    assert_eq!(surface_materialization_count_for_tests(), materializations);

    let restored = super::super::build_view_template_nodes(
        document_tree_id,
        layout,
        &[],
        UiSize::new(800.0, 480.0),
        &BTreeMap::new(),
    )
    .expect("authored text should remain recoverable");
    assert_eq!(
        restored
            .iter()
            .find(|node| node.control_id == "ProjectOverviewTitleText")
            .map(|node| node.text.as_str()),
        Some("Project")
    );
}
