use crate::ui::workbench::autolayout::{
    compact_bottom_height_limit, compact_side_width_limit, compute_workbench_shell_geometry,
    compute_workbench_shell_geometry_with_region_defaults, workbench_layout_defaults,
    workbench_layout_tier_for_logical_width, workbench_layout_tier_for_physical_width,
    workbench_logical_width_for_scale, EditorRegion, EditorRegionRole, RegionBinding, ShellFrame,
    ShellRegionId, ShellSizePx, WorkbenchChromeMetrics, WorkbenchConstraintTokenName,
    WorkbenchLayoutTier, WorkbenchShellRegionsAsset, WorkbenchShellRegionsAssetError,
    WorkbenchSkeleton, WORKBENCH_SHELL_REGIONS_ASSET_ID, WORKBENCH_SHELL_REGIONS_ASSET_KIND,
    WORKBENCH_SHELL_REGIONS_ASSET_VERSION,
};
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::layout::{
    ActivityDrawerMode, ActivityDrawerSlot, DocumentNode, LayoutCommand, LayoutCommandError,
    LayoutManager, MainHostPageLayout, MainPageId, SplitAxis, SplitPlacement, WorkbenchLayout,
    WorkspaceTarget,
};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::view::{ViewHost, ViewInstanceId};
use crate::ui::workbench::{
    FloatingLayer, FloatingWindow, FloatingWindowKind, LayoutPreset, LayoutPresetName,
    PageLayoutTemplate,
};
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

const SHELL_REGIONS_ASSET: &str =
    include_str!("../../../../assets/ui/editor/layout/shell_regions.toml");

#[test]
fn region_bindings_map_semantic_regions_to_existing_drawer_slots() {
    let binding = RegionBinding::new(
        EditorRegion::LeftBottom,
        EditorRegionRole::ProjectTree,
        "res://ui/editor/host/asset_surface_controls.zui",
        Some(WorkbenchConstraintTokenName::new("--left-drawer-width")),
    )
    .expect("project tree belongs in left-bottom");

    assert_eq!(binding.drawer_slot(), Some(ActivityDrawerSlot::LeftBottom));
    assert_eq!(binding.shell_region(), ShellRegionId::Left);
    assert_eq!(
        binding.size_token.as_ref().unwrap().as_str(),
        "--left-drawer-width"
    );

    let error = RegionBinding::new(
        EditorRegion::LeftBottom,
        EditorRegionRole::DetailInspector,
        "res://ui/editor/host/inspector_body.zui",
        None,
    )
    .expect_err("inspector panels must not enter the project-tree slot");
    assert_eq!(error.region(), EditorRegion::LeftBottom);
    assert_eq!(error.expected_role(), EditorRegionRole::ProjectTree);
    assert_eq!(error.actual_role(), EditorRegionRole::DetailInspector);
}

#[test]
fn jetbrains_workbench_skeleton_exposes_fixed_layout_regions() {
    let skeleton = WorkbenchSkeleton::jetbrains_default();

    assert_eq!(skeleton.regions.len(), 6);
    assert_eq!(
        skeleton.region(EditorRegion::RightBottom).unwrap().role,
        EditorRegionRole::DetailInspector
    );
    assert_eq!(
        skeleton.region(EditorRegion::Bottom).unwrap().drawer_slot(),
        Some(ActivityDrawerSlot::Bottom)
    );
    assert_eq!(
        skeleton.default_drawer_mode(EditorRegion::Bottom),
        Some(ActivityDrawerMode::Pinned)
    );
    assert!(skeleton
        .region(EditorRegion::Center)
        .unwrap()
        .panel_asset
        .ends_with("workbench_main_band.zui"));
}

#[test]
fn shell_regions_asset_loads_verified_workbench_skeleton_regions() {
    let asset =
        WorkbenchShellRegionsAsset::from_toml_str(SHELL_REGIONS_ASSET).expect("asset parses");

    assert_eq!(asset.header.kind, WORKBENCH_SHELL_REGIONS_ASSET_KIND);
    assert_eq!(asset.header.id, WORKBENCH_SHELL_REGIONS_ASSET_ID);
    assert_eq!(asset.header.version, WORKBENCH_SHELL_REGIONS_ASSET_VERSION);

    let skeleton = WorkbenchSkeleton::from_shell_regions_asset(asset);

    assert_eq!(skeleton.regions.len(), EditorRegion::ALL.len());
    assert_eq!(
        skeleton
            .region(EditorRegion::LeftBottom)
            .unwrap()
            .panel_asset,
        "res://ui/editor/asset_browser.zui"
    );
    assert_eq!(
        skeleton.region(EditorRegion::Center).unwrap().panel_asset,
        "res://ui/editor/components/workbench/shell/workbench_main_band.zui"
    );
    assert_eq!(
        skeleton
            .region(EditorRegion::Bottom)
            .unwrap()
            .size_token
            .as_ref()
            .unwrap()
            .as_str(),
        "--bottom-output-height"
    );
    assert_eq!(
        skeleton.default_drawer_mode(EditorRegion::RightBottom),
        Some(ActivityDrawerMode::Pinned)
    );

    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.density.left_drawer_width = 512.0;
    tokens.density.right_drawer_width = 544.0;
    tokens.density.bottom_output_height = 288.0;
    let extents = skeleton.preferred_region_extents_from_tokens(&tokens);

    assert_eq!(extents.get(&ShellRegionId::Left), Some(&512.0));
    assert_eq!(extents.get(&ShellRegionId::Right), Some(&544.0));
    assert_eq!(extents.get(&ShellRegionId::Bottom), Some(&288.0));

    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    let shell_size = ShellSizePx::new(2400.0, 1200.0);
    let baseline = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        shell_size,
        1.0,
        &metrics,
        None,
    );
    let from_asset = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        shell_size,
        1.0,
        &metrics,
        Some(&extents),
    );

    assert!(
        from_asset.region_frame(ShellRegionId::Left).width
            > baseline.region_frame(ShellRegionId::Left).width
    );
    assert!(
        from_asset.region_frame(ShellRegionId::Right).width
            > baseline.region_frame(ShellRegionId::Right).width
    );
    assert!(
        from_asset.region_frame(ShellRegionId::Bottom).height
            > baseline.region_frame(ShellRegionId::Bottom).height
    );
}

#[test]
fn shell_regions_asset_rejects_region_role_mismatches() {
    let mismatched_source =
        SHELL_REGIONS_ASSET.replace("role = \"project_tree\"", "role = \"detail_inspector\"");
    let error = WorkbenchSkeleton::from_shell_regions_asset_str(&mismatched_source)
        .expect_err("asset role mismatch must fail");

    match error {
        WorkbenchShellRegionsAssetError::RoleMismatch {
            region,
            expected_role,
            actual_role,
        } => {
            assert_eq!(region, EditorRegion::LeftBottom);
            assert_eq!(expected_role, EditorRegionRole::ProjectTree);
            assert_eq!(actual_role, EditorRegionRole::DetailInspector);
        }
        other => panic!("expected role mismatch, got {other:?}"),
    }
}

#[test]
fn jetbrains_docking_state_commands_drive_drawer_split_and_active_contracts(
) -> Result<(), LayoutCommandError> {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let project = ViewInstanceId::new("editor.project#jetbrains-contract");
    let scene = ViewInstanceId::new("editor.scene#jetbrains-contract");
    let material = ViewInstanceId::new("editor.material#jetbrains-contract");

    manager.apply(
        &mut layout,
        LayoutCommand::AttachView {
            instance_id: project.clone(),
            target: ViewHost::Drawer(ActivityDrawerSlot::LeftBottom),
            anchor: None,
        },
    )?;
    manager.apply(
        &mut layout,
        LayoutCommand::SetDrawerMode {
            slot: ActivityDrawerSlot::LeftBottom,
            mode: ActivityDrawerMode::Collapsed,
        },
    )?;

    let Some(collapsed_drawer) = layout.drawers.get(&ActivityDrawerSlot::LeftBottom) else {
        panic!("expected left-bottom drawer");
    };
    assert_eq!(collapsed_drawer.mode, ActivityDrawerMode::Collapsed);
    assert_eq!(collapsed_drawer.tab_stack.tabs, vec![project.clone()]);
    assert_eq!(collapsed_drawer.active_view, None);

    manager.apply(
        &mut layout,
        LayoutCommand::ActivateDrawerTab {
            slot: ActivityDrawerSlot::LeftBottom,
            instance_id: project.clone(),
        },
    )?;
    let Some(active_drawer) = layout.drawers.get(&ActivityDrawerSlot::LeftBottom) else {
        panic!("expected active left-bottom drawer");
    };
    assert_eq!(active_drawer.mode, ActivityDrawerMode::Pinned);
    assert_eq!(active_drawer.tab_stack.active_tab.as_ref(), Some(&project));
    assert_eq!(active_drawer.active_view.as_ref(), Some(&project));

    manager.apply(
        &mut layout,
        LayoutCommand::OpenView {
            instance_id: scene.clone(),
            target: ViewHost::Document(MainPageId::workbench(), vec![]),
        },
    )?;
    manager.apply(
        &mut layout,
        LayoutCommand::CreateSplit {
            workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
            path: vec![],
            axis: SplitAxis::Horizontal,
            placement: SplitPlacement::After,
            new_instance: material.clone(),
        },
    )?;

    let MainHostPageLayout::WorkbenchPage {
        document_workspace, ..
    } = &layout.main_pages[0]
    else {
        panic!("expected workbench page");
    };
    let DocumentNode::SplitNode {
        axis,
        ratio,
        first,
        second,
    } = document_workspace
    else {
        panic!("expected split document root");
    };
    assert_eq!(*axis, SplitAxis::Horizontal);
    assert_eq!(*ratio, 0.5);

    let DocumentNode::Tabs(first_tabs) = first.as_ref() else {
        panic!("expected first split tab stack");
    };
    let DocumentNode::Tabs(second_tabs) = second.as_ref() else {
        panic!("expected second split tab stack");
    };
    assert_eq!(first_tabs.tabs, vec![scene.clone()]);
    assert_eq!(second_tabs.tabs, vec![material.clone()]);

    manager.apply(
        &mut layout,
        LayoutCommand::FocusView {
            instance_id: scene.clone(),
        },
    )?;
    let MainHostPageLayout::WorkbenchPage {
        document_workspace, ..
    } = &layout.main_pages[0]
    else {
        panic!("expected workbench page");
    };
    let DocumentNode::SplitNode { first, .. } = document_workspace else {
        panic!("expected split document root");
    };
    let DocumentNode::Tabs(first_tabs) = first.as_ref() else {
        panic!("expected first split tab stack");
    };
    assert_eq!(first_tabs.active_tab.as_ref(), Some(&scene));

    Ok(())
}

#[test]
fn layout_command_failures_are_typed_for_docking_contract_errors() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let missing = ViewInstanceId::new("editor.missing#typed-error");

    let missing_tab = match manager.apply(
        &mut layout,
        LayoutCommand::ActivateDrawerTab {
            slot: ActivityDrawerSlot::LeftBottom,
            instance_id: missing.clone(),
        },
    ) {
        Ok(_) => panic!("activating a drawer tab outside the drawer must fail"),
        Err(error) => error,
    };
    assert_eq!(
        missing_tab,
        LayoutCommandError::DrawerMissingTab {
            slot: ActivityDrawerSlot::LeftBottom,
            instance_id: missing
        }
    );

    let non_split = match manager.apply(
        &mut layout,
        LayoutCommand::ResizeSplit {
            workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
            path: vec![],
            ratio: 0.7,
        },
    ) {
        Ok(_) => panic!("resizing a tab node must fail"),
        Err(error) => error,
    };
    assert_eq!(
        non_split,
        LayoutCommandError::TargetPathIsNotSplitNode {
            workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
            path: Vec::new()
        }
    );
}

#[test]
fn built_in_layout_presets_match_authoring_review_focus_debug_contracts() {
    let presets = LayoutPreset::builtin_presets();

    assert_eq!(
        presets.iter().map(|preset| preset.name).collect::<Vec<_>>(),
        vec![
            LayoutPresetName::Authoring,
            LayoutPresetName::Review,
            LayoutPresetName::Focus,
            LayoutPresetName::Debug,
        ]
    );
    assert!(presets
        .iter()
        .find(|preset| preset.name == LayoutPresetName::Focus)
        .unwrap()
        .drawer_states
        .iter()
        .all(|state| state.mode == ActivityDrawerMode::Collapsed));
    assert!(presets
        .iter()
        .find(|preset| preset.name == LayoutPresetName::Debug)
        .unwrap()
        .size_overrides
        .iter()
        .any(|override_value| override_value.token.as_str() == "--bottom-output-height"));
}

#[test]
fn page_templates_bind_core_pages_to_the_shared_skeleton_regions() {
    let scene = PageLayoutTemplate::scene();
    let material = PageLayoutTemplate::material();
    let inspector = PageLayoutTemplate::inspector();

    assert_eq!(scene.default_preset, LayoutPresetName::Authoring);
    assert!(scene.has_region_role(EditorRegion::RightTop, EditorRegionRole::HierarchyStructure));
    assert!(scene.has_region_role(EditorRegion::RightBottom, EditorRegionRole::DetailInspector));
    assert!(material.has_region_role(EditorRegion::Center, EditorRegionRole::CenterDocument));
    assert!(inspector.has_region_role(EditorRegion::Center, EditorRegionRole::CenterDocument));
}

#[test]
fn floating_window_declarations_preserve_modal_and_layer_contracts() {
    let command_palette = FloatingWindow::command_palette();
    let preferences = FloatingWindow::preferences();

    assert_eq!(command_palette.kind, FloatingWindowKind::CommandPalette);
    assert_eq!(command_palette.layer, FloatingLayer::TopOverlay);
    assert!(!command_palette.modal);
    assert_eq!(preferences.kind, FloatingWindowKind::Preferences);
    assert!(preferences.modal);
    assert!(preferences
        .content_asset
        .ends_with("workbench_preferences.zui"));
}

#[test]
fn region_size_tokens_feed_shell_autolayout_preferred_extents() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.density.left_drawer_width = 520.0;
    tokens.density.right_drawer_width = 560.0;
    tokens.density.bottom_output_height = 320.0;

    let skeleton = WorkbenchSkeleton::jetbrains_default();
    let extents = skeleton.preferred_region_extents_from_tokens(&tokens);

    assert_eq!(extents.get(&ShellRegionId::Left), Some(&520.0));
    assert_eq!(extents.get(&ShellRegionId::Right), Some(&560.0));
    assert_eq!(extents.get(&ShellRegionId::Bottom), Some(&320.0));
    assert!(!extents.contains_key(&ShellRegionId::Document));

    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    let shell_size = ShellSizePx::new(2400.0, 1200.0);
    let baseline = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        shell_size,
        1.0,
        &metrics,
        None,
    );
    let token_default = compute_workbench_shell_geometry_with_region_defaults(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        shell_size,
        1.0,
        &metrics,
        None,
        Some(&extents),
    );

    assert_eq!(
        token_default.region_frame(ShellRegionId::Left).width,
        baseline.region_frame(ShellRegionId::Left).width
    );
    assert_eq!(
        token_default.region_frame(ShellRegionId::Right).width,
        baseline.region_frame(ShellRegionId::Right).width
    );
    assert_eq!(
        token_default.region_frame(ShellRegionId::Bottom).height,
        baseline.region_frame(ShellRegionId::Bottom).height
    );

    let mut without_persisted_drawers = model.clone();
    without_persisted_drawers.drawer_ring.drawers.clear();
    let token_fallback = compute_workbench_shell_geometry_with_region_defaults(
        &without_persisted_drawers,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        shell_size,
        1.0,
        &metrics,
        None,
        Some(&extents),
    );

    assert!(
        token_fallback.region_frame(ShellRegionId::Left).width
            > baseline.region_frame(ShellRegionId::Left).width
    );
    assert!(
        token_fallback.region_frame(ShellRegionId::Right).width
            > baseline.region_frame(ShellRegionId::Right).width
    );
    assert!(
        token_fallback.region_frame(ShellRegionId::Bottom).height
            > baseline.region_frame(ShellRegionId::Bottom).height
    );

    let transient = [
        (ShellRegionId::Left, 640.0),
        (ShellRegionId::Right, 680.0),
        (ShellRegionId::Bottom, 360.0),
    ]
    .into_iter()
    .collect();
    let active_drag = compute_workbench_shell_geometry_with_region_defaults(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        shell_size,
        1.0,
        &metrics,
        Some(&transient),
        Some(&extents),
    );

    assert!(
        active_drag.region_frame(ShellRegionId::Left).width
            > baseline.region_frame(ShellRegionId::Left).width
    );
    assert!(
        active_drag.region_frame(ShellRegionId::Right).width
            > baseline.region_frame(ShellRegionId::Right).width
    );
    assert!(
        active_drag.region_frame(ShellRegionId::Bottom).height
            > baseline.region_frame(ShellRegionId::Bottom).height
    );
}

#[test]
fn workbench_layout_tiers_classify_reference_capture_widths() {
    assert_eq!(
        workbench_layout_tier_for_logical_width(420.0),
        WorkbenchLayoutTier::Ultra
    );
    assert_eq!(
        workbench_layout_tier_for_logical_width(640.0),
        WorkbenchLayoutTier::Narrow
    );
    assert_eq!(
        workbench_layout_tier_for_logical_width(900.0),
        WorkbenchLayoutTier::Regular
    );
    assert_eq!(
        workbench_layout_tier_for_logical_width(1260.0),
        WorkbenchLayoutTier::Wide
    );
}

#[test]
fn tier_uses_logical_width_consistent_across_scale() {
    assert_eq!(workbench_logical_width_for_scale(3840.0, 2.0), 1920.0);
    assert_eq!(
        workbench_layout_tier_for_physical_width(3840.0, 2.0),
        workbench_layout_tier_for_physical_width(1920.0, 1.0)
    );
    assert_eq!(
        workbench_layout_tier_for_physical_width(1280.0, 2.0),
        WorkbenchLayoutTier::Narrow
    );
    assert_eq!(
        workbench_layout_tier_for_physical_width(1800.0, 2.0),
        WorkbenchLayoutTier::Regular
    );
}

#[test]
fn workbench_breakpoint_defaults_are_sourced_from_design_tokens() {
    let tokens = EditorDesignTokens::workbench_dark();
    let defaults = workbench_layout_defaults();

    assert_eq!(
        defaults.breakpoints.ultra_max_width,
        tokens.density.breakpoint_ultra_width
    );
    assert_eq!(
        defaults.breakpoints.narrow_max_width,
        tokens.density.breakpoint_narrow_width
    );
    assert_eq!(
        defaults.breakpoints.wide_min_width,
        tokens.density.breakpoint_wide_width
    );
    assert_eq!(
        defaults.compact_side.available_width,
        tokens.density.compact_side_width
    );
    assert_eq!(
        defaults.compact_bottom.available_height,
        tokens.density.compact_bottom_available_height
    );
    assert_eq!(
        defaults.window_minimums.min_width,
        tokens.density.minimum_window_width
    );
    assert_eq!(
        defaults.minimum_document_width_fraction,
        tokens.density.minimum_document_width_fraction
    );
}

#[test]
fn compact_region_limits_follow_breakpoint_density_defaults() {
    let defaults = workbench_layout_defaults();

    assert_eq!(
        compact_side_width_limit(
            ShellRegionId::Left,
            defaults.compact_side.ultra_available_width
        ),
        Some(defaults.compact_side.ultra_left_max_width)
    );
    assert_eq!(
        compact_side_width_limit(ShellRegionId::Right, defaults.compact_side.available_width),
        Some(
            defaults
                .compact_side
                .right_max_width
                .max(defaults.compact_side.side_min_width)
        )
    );
    assert_eq!(
        compact_bottom_height_limit(defaults.compact_bottom.ultra_available_height),
        Some(
            (defaults.compact_bottom.ultra_available_height
                * defaults.compact_bottom.ultra_max_available_fraction)
                .min(defaults.compact_bottom.ultra_max_height)
                .max(defaults.compact_bottom.ultra_min_height)
        )
    );
    assert_eq!(
        compact_bottom_height_limit(defaults.compact_bottom.available_height),
        Some(defaults.compact_bottom.max_height)
    );
}

#[test]
fn compact_bottom_limit_preserves_fractional_logical_layout_units() {
    let limit = compact_bottom_height_limit(419.9)
        .expect("the ultra-compact breakpoint should supply a bottom limit");

    assert!((limit - 83.98).abs() < 0.001);
}

#[test]
fn narrow_workbench_geometry_collapses_right_drawer_to_rail() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    let narrow = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(640.0, 420.0),
        1.0,
        &metrics,
        None,
    );
    let regular = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(900.0, 620.0),
        1.0,
        &metrics,
        None,
    );

    assert_eq!(
        narrow.region_frame(ShellRegionId::Right).width,
        metrics.rail_width
    );
    assert_eq!(narrow.splitter_frame(ShellRegionId::Right).width, 0.0);
    assert!(regular.region_frame(ShellRegionId::Right).width > metrics.rail_width);
    assert!(regular.splitter_frame(ShellRegionId::Right).width > 0.0);
    assert!(
        narrow.region_frame(ShellRegionId::Document).width
            > regular.region_frame(ShellRegionId::Document).width - 260.0
    );
}

#[test]
fn regular_workbench_geometry_reserves_token_backed_document_width() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    let shell_size = ShellSizePx::new(900.0, 620.0);
    let geometry = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        shell_size,
        1.0,
        &metrics,
        None,
    );
    let document = geometry.region_frame(ShellRegionId::Document);

    assert!(geometry.region_frame(ShellRegionId::Left).width > metrics.rail_width);
    assert!(geometry.region_frame(ShellRegionId::Right).width > metrics.rail_width);
    assert!(
        document.width
            >= shell_size.width * workbench_layout_defaults().minimum_document_width_fraction,
        "regular shell geometry should reserve token-backed document width: {document:?}"
    );
}

#[test]
fn scaled_workbench_geometry_uses_logical_width_for_right_drawer_collapse() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    let scaled_narrow = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(1280.0, 840.0),
        2.0,
        &metrics,
        None,
    );
    let scaled_regular = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(1800.0, 1240.0),
        2.0,
        &metrics,
        None,
    );

    assert_eq!(
        scaled_narrow.region_frame(ShellRegionId::Right).width,
        metrics.rail_width * 2.0
    );
    assert!(scaled_regular.region_frame(ShellRegionId::Right).width > metrics.rail_width * 2.0);
}

#[test]
fn equivalent_logical_workbenches_scale_all_shell_geometry_at_the_dpi_boundary() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    let logical = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(900.0, 620.0),
        1.0,
        &metrics,
        None,
    );
    let high_dpi = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(1800.0, 1240.0),
        2.0,
        &metrics,
        None,
    );

    let assert_scaled = |logical: ShellFrame, physical: ShellFrame| {
        assert!((physical.x - logical.x * 2.0).abs() < 0.001);
        assert!((physical.y - logical.y * 2.0).abs() < 0.001);
        assert!((physical.width - logical.width * 2.0).abs() < 0.001);
        assert!((physical.height - logical.height * 2.0).abs() < 0.001);
    };

    assert_scaled(logical.center_band_frame, high_dpi.center_band_frame);
    assert_scaled(logical.status_bar_frame, high_dpi.status_bar_frame);
    for region in [
        ShellRegionId::Left,
        ShellRegionId::Document,
        ShellRegionId::Right,
        ShellRegionId::Bottom,
    ] {
        assert_scaled(logical.region_frame(region), high_dpi.region_frame(region));
        assert_scaled(
            logical.splitter_frame(region),
            high_dpi.splitter_frame(region),
        );
    }
    assert_scaled(
        logical.viewport_content_frame,
        high_dpi.viewport_content_frame,
    );
    assert!((high_dpi.window_min_width - logical.window_min_width * 2.0).abs() < 0.001);
    assert!((high_dpi.window_min_height - logical.window_min_height * 2.0).abs() < 0.001);
}

#[test]
fn physical_drawer_resize_preferences_are_converted_before_logical_layout() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    let logical_preferred = std::collections::BTreeMap::from([(ShellRegionId::Left, 360.0)]);
    let physical_preferred = std::collections::BTreeMap::from([(ShellRegionId::Left, 720.0)]);
    let logical = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(900.0, 620.0),
        1.0,
        &metrics,
        Some(&logical_preferred),
    );
    let high_dpi = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(1800.0, 1240.0),
        2.0,
        &metrics,
        Some(&physical_preferred),
    );

    assert!(
        (high_dpi.region_frame(ShellRegionId::Left).width
            - logical.region_frame(ShellRegionId::Left).width * 2.0)
            .abs()
            < 0.001
    );
}

#[test]
fn workbench_shell_geometry_vertical_layout_uses_a_flex_band_solver_instead_of_pixel_sums() {
    let region_frames = include_str!("../../../ui/workbench/autolayout/geometry/region_frames.rs");
    let vertical_bands =
        include_str!("../../../ui/workbench/autolayout/geometry/vertical_bands.rs");

    assert!(region_frames.contains("resolve_vertical_flex_bands"));
    assert!(!region_frames.contains("fixed_vertical"));
    assert!(!region_frames.contains("let mut y"));
    assert!(vertical_bands.contains("solve_axis_constraints(available_height"));
    assert!(vertical_bands.contains("VerticalFlexBandStack"));
}

#[test]
fn workbench_window_minimums_allow_reference_capture_sizes() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    let narrow = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(640.0, 420.0),
        1.0,
        &metrics,
        None,
    );
    let regular = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(900.0, 620.0),
        1.0,
        &metrics,
        None,
    );

    assert!(narrow.window_min_width <= 640.0);
    assert!(narrow.window_min_height <= 420.0);
    assert!(regular.window_min_width <= 640.0);
    assert!(regular.window_min_height <= 420.0);
}
