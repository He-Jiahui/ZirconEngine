use crate::ui::workbench::autolayout::{
    compact_bottom_height_limit, compact_side_width_limit, compute_workbench_shell_geometry,
    workbench_layout_defaults, workbench_layout_tier_for_width, EditorRegion, EditorRegionRole,
    RegionBinding, ShellRegionId, ShellSizePx, WorkbenchChromeMetrics,
    WorkbenchConstraintTokenName, WorkbenchLayoutTier, WorkbenchShellRegionsAsset,
    WorkbenchShellRegionsAssetError, WorkbenchSkeleton, WORKBENCH_SHELL_REGIONS_ASSET_ID,
    WORKBENCH_SHELL_REGIONS_ASSET_KIND, WORKBENCH_SHELL_REGIONS_ASSET_VERSION,
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

const EDITOR_TOKENS_ASSET: &str =
    include_str!("../../../../assets/ui/editor/theme/editor_tokens.zui");
const WORKBENCH_SKELETON_ASSET: &str =
    include_str!("../../../../assets/ui/editor/components/workbench/shell/workbench_skeleton.zui");
const WORKBENCH_MAIN_BAND_ASSET: &str =
    include_str!("../../../../assets/ui/editor/components/workbench/shell/workbench_main_band.zui");
const WORKBENCH_SCENE_TREE_PANEL_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui"
);
const WORKBENCH_INSPECTOR_PANEL_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui"
);
const COMMAND_PALETTE_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/floating/workbench_command_palette.zui"
);
const PREFERENCES_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/floating/workbench_preferences.zui"
);
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
    let model = WorkbenchViewModel::build(&chrome);
    let metrics = WorkbenchChromeMetrics::default();
    let shell_size = ShellSizePx::new(2400.0, 1200.0);
    let baseline = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        shell_size,
        &metrics,
        None,
    );
    let from_asset = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        shell_size,
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
fn layout_skeleton_and_floating_assets_reference_editor_tokens_instead_of_hex_colors() {
    assert!(EDITOR_TOKENS_ASSET.contains("editor.surface.0"));
    assert!(EDITOR_TOKENS_ASSET.contains("editor.control.height.default"));
    assert!(EDITOR_TOKENS_ASSET.contains("--left-drawer-width"));

    for (asset_name, asset_source) in [
        ("workbench_skeleton.zui", WORKBENCH_SKELETON_ASSET),
        ("workbench_command_palette.zui", COMMAND_PALETTE_ASSET),
        ("workbench_preferences.zui", PREFERENCES_ASSET),
    ] {
        assert!(
            asset_source.contains("res://ui/editor/theme/editor_tokens.zui"),
            "{asset_name} must import the editor token asset"
        );
        assert!(
            asset_source.contains("editor.surface.")
                || asset_source.contains("editor.text.")
                || asset_source.contains("editor.border"),
            "{asset_name} must reference editor token names"
        );
        assert!(
            !contains_hex_color(asset_source),
            "{asset_name} must not reintroduce naked hex colors"
        );
    }
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
    let model = WorkbenchViewModel::build(&chrome);
    let metrics = WorkbenchChromeMetrics::default();
    let shell_size = ShellSizePx::new(2400.0, 1200.0);
    let baseline = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        shell_size,
        &metrics,
        None,
    );
    let tokenized = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        shell_size,
        &metrics,
        Some(&extents),
    );

    assert!(
        tokenized.region_frame(ShellRegionId::Left).width
            > baseline.region_frame(ShellRegionId::Left).width
    );
    assert!(
        tokenized.region_frame(ShellRegionId::Right).width
            > baseline.region_frame(ShellRegionId::Right).width
    );
    assert!(
        tokenized.region_frame(ShellRegionId::Bottom).height
            > baseline.region_frame(ShellRegionId::Bottom).height
    );
}

#[test]
fn workbench_layout_tiers_classify_reference_capture_widths() {
    assert_eq!(
        workbench_layout_tier_for_width(420.0),
        WorkbenchLayoutTier::Ultra
    );
    assert_eq!(
        workbench_layout_tier_for_width(640.0),
        WorkbenchLayoutTier::Narrow
    );
    assert_eq!(
        workbench_layout_tier_for_width(900.0),
        WorkbenchLayoutTier::Regular
    );
    assert_eq!(
        workbench_layout_tier_for_width(1260.0),
        WorkbenchLayoutTier::Wide
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
                .round()
        )
    );
    assert_eq!(
        compact_bottom_height_limit(defaults.compact_bottom.available_height),
        Some(defaults.compact_bottom.max_height)
    );
}

#[test]
fn narrow_workbench_geometry_collapses_right_drawer_to_rail() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let metrics = WorkbenchChromeMetrics::default();
    let narrow = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(640.0, 420.0),
        &metrics,
        None,
    );
    let regular = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(900.0, 620.0),
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
fn workbench_window_minimums_allow_reference_capture_sizes() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let metrics = WorkbenchChromeMetrics::default();
    let narrow = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(640.0, 420.0),
        &metrics,
        None,
    );
    let regular = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(900.0, 620.0),
        &metrics,
        None,
    );

    assert!(narrow.window_min_width <= 640.0);
    assert!(narrow.window_min_height <= 420.0);
    assert!(regular.window_min_width <= 640.0);
    assert!(regular.window_min_height <= 420.0);
}

#[test]
fn shell_drawer_assets_use_constraint_tokens_instead_of_inline_drawer_widths() {
    for (asset_name, asset_source, token_name, removed_width) in [
        (
            "workbench_main_band.zui",
            WORKBENCH_MAIN_BAND_ASSET,
            "$--left-drawer-width",
            "332.0",
        ),
        (
            "workbench_main_band.zui",
            WORKBENCH_MAIN_BAND_ASSET,
            "$--right-drawer-width",
            "404.0",
        ),
        (
            "workbench_scene_tree_panel.zui",
            WORKBENCH_SCENE_TREE_PANEL_ASSET,
            "$--left-drawer-width",
            "332.0",
        ),
        (
            "workbench_inspector_panel.zui",
            WORKBENCH_INSPECTOR_PANEL_ASSET,
            "$--right-drawer-width",
            "404.0",
        ),
    ] {
        assert!(
            asset_source.contains("res://ui/editor/theme/editor_tokens.zui"),
            "{asset_name} must import the editor token asset"
        );
        assert!(
            asset_source.contains(token_name),
            "{asset_name} must reference {token_name}"
        );
        assert!(
            !asset_source.contains(removed_width),
            "{asset_name} must not keep the old inline drawer width {removed_width}"
        );
    }
}

fn contains_hex_color(source: &str) -> bool {
    source
        .as_bytes()
        .windows(7)
        .any(|window| window[0] == b'#' && window[1..].iter().all(u8::is_ascii_hexdigit))
}
