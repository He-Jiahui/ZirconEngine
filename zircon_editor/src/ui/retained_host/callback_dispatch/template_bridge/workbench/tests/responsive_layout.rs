use super::super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::*;

#[test]
fn responsive_min_tier_order_matches_workbench_breakpoint_order() {
    assert!(tier_rank(WorkbenchLayoutTier::Ultra) < tier_rank(WorkbenchLayoutTier::Narrow));
    assert!(tier_rank(WorkbenchLayoutTier::Narrow) < tier_rank(WorkbenchLayoutTier::Regular));
    assert!(tier_rank(WorkbenchLayoutTier::Regular) < tier_rank(WorkbenchLayoutTier::Wide));
    assert_eq!(
        parse_layout_tier(" regular "),
        Some(WorkbenchLayoutTier::Regular)
    );
    assert_eq!(parse_layout_tier("unsupported"), None);
}

#[test]
fn responsive_tier_parsing_avoids_per_node_lowercase_allocation() {
    let source = include_str!("../responsive_layout.rs");
    let forbidden = ["to_ascii", "_lowercase"].concat();

    assert!(!source.contains(&forbidden));
}

#[test]
fn responsive_visibility_honors_both_tier_bounds() {
    let visible = |tier| {
        responsive_node_visible(
            tier,
            Some(WorkbenchLayoutTier::Narrow),
            Some(WorkbenchLayoutTier::Regular),
            false,
            false,
        )
    };

    assert!(!visible(WorkbenchLayoutTier::Ultra));
    assert!(visible(WorkbenchLayoutTier::Narrow));
    assert!(visible(WorkbenchLayoutTier::Regular));
    assert!(!visible(WorkbenchLayoutTier::Wide));
}

#[test]
fn compact_details_drawer_overrides_wide_minimum_only_when_open() {
    let visible = |tier, open| {
        responsive_node_visible(tier, Some(WorkbenchLayoutTier::Wide), None, true, open)
    };

    assert!(visible(WorkbenchLayoutTier::Wide, false));
    assert!(!visible(WorkbenchLayoutTier::Regular, false));
    assert!(visible(WorkbenchLayoutTier::Regular, true));
    assert!(visible(WorkbenchLayoutTier::Narrow, true));
    assert!(!visible(WorkbenchLayoutTier::Ultra, true));
}

#[test]
fn compact_module_details_drawer_is_reachable_without_reducing_regular_center_budget() {
    let mut regular = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("regular workbench should build");
    assert!(regular
        .control_frame("WorkbenchModuleDetailsDrawerToggle")
        .is_some());
    assert!(regular.control_frame("WorkbenchEffectRightPanel").is_none());
    let center_before_drawer = regular
        .control_frame("WorkbenchEffectCenterPanel")
        .expect("regular effect center should remain visible");

    regular
        .dispatch_binding_state_for_control(
            "WorkbenchModuleDetailsDrawerToggle",
            "Workbench/ToggleModuleDetailsDrawer",
        )
        .expect("details drawer should open");
    assert!(regular.control_frame("WorkbenchEffectRightPanel").is_some());
    assert_eq!(
        regular.control_frame("WorkbenchEffectCenterPanel"),
        Some(center_before_drawer),
        "overlay details must not consume the regular center budget"
    );

    regular
        .dispatch_binding_state_for_control(
            "WorkbenchModuleDetailsDrawerToggle",
            "Workbench/ToggleModuleDetailsDrawer",
        )
        .expect("details drawer should close");
    assert!(regular.control_frame("WorkbenchEffectRightPanel").is_none());

    regular
        .dispatch_binding_state_for_control("WorkbenchModuleScene", "WorkbenchModule/Scene")
        .expect("scene workspace should activate");
    assert!(regular
        .control_frame("WorkbenchModuleDetailsDrawerToggle")
        .is_none());
    regular
        .dispatch_binding_state_for_control("WorkbenchModuleEffect", "WorkbenchModule/Effect")
        .expect("effect workspace should reactivate");
    assert!(regular
        .control_frame("WorkbenchModuleDetailsDrawerToggle")
        .is_some());

    let wide = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("wide workbench should build");
    assert!(wide.control_frame("WorkbenchEffectRightPanel").is_some());
    assert!(wide
        .control_frame("WorkbenchModuleDetailsDrawerToggle")
        .is_none());

    let ultra = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(420.0, 520.0))
        .expect("ultra workbench should build");
    assert!(ultra.control_frame("WorkbenchEffectRightPanel").is_none());
    assert!(ultra
        .control_frame("WorkbenchModuleDetailsDrawerToggle")
        .is_none());
}
