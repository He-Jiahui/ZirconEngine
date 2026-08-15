use super::*;

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
