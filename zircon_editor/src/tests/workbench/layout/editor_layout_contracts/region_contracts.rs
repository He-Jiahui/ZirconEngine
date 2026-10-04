//! 语义区域、抽屉槽和大小token到壳几何的连接；角色不匹配必须在资产入口拒绝。
use super::*;

#[test]
/// 区域绑定须符合固定角色，允许从语义区域推导既有抽屉槽而不复制另一套槽身份。
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
/// 默认骨架提供预期六区域及模式，中心引用主band而非一个抽屉槽。
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
/// 验证真实区域资源头、角色和尺寸来源；修改主题尺寸后还比较实际壳几何以证明消费链。
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
/// 通过真实资源变体触发角色错误，确认异常指向具体区域与预期角色。
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
