use super::*;

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
fn shell_geometry_honors_an_explicit_constant_pixel_root_policy() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    let constant_physical = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(1800.0, 1240.0),
        2.0,
        &metrics,
        None,
    );
    let constant_pixel = compute_workbench_shell_geometry_with_scale_mode(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(1800.0, 1240.0),
        2.0,
        ResolutionScaleMode::ConstantPixel,
        &metrics,
        None,
    );

    assert_eq!(
        constant_physical.status_bar_frame.height,
        metrics.status_bar_height * 2.0
    );
    assert_eq!(
        constant_pixel.status_bar_frame.height,
        metrics.status_bar_height
    );
    assert!(constant_pixel.center_band_frame.height > constant_physical.center_band_frame.height);
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
    let region_frames =
        include_str!("../../../../ui/workbench/autolayout/geometry/region_frames.rs");
    let vertical_bands =
        include_str!("../../../../ui/workbench/autolayout/geometry/vertical_bands.rs");

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

#[test]
fn scene_window_minimum_reaches_ultra_floor_after_narrow_bridge_at_each_dpi() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    let defaults = workbench_layout_defaults();

    for scale in [1.0_f32, 1.25, 2.0] {
        let narrow = compute_workbench_shell_geometry(
            &model,
            &chrome,
            &fixture.layout,
            &fixture.descriptors,
            ShellSizePx::new(640.0 * scale, 620.0 * scale),
            scale,
            &metrics,
            None,
        );
        let ultra = compute_workbench_shell_geometry(
            &model,
            &chrome,
            &fixture.layout,
            &fixture.descriptors,
            ShellSizePx::new(480.0 * scale, 620.0 * scale),
            scale,
            &metrics,
            None,
        );

        assert!(
            (narrow.window_min_width - defaults.breakpoints.ultra_max_width * scale).abs() < 0.001,
            "narrow minimum should bridge to the Ultra breakpoint at scale {scale}: {}",
            narrow.window_min_width,
        );
        assert!(
            (ultra.window_min_width - defaults.window_minimums.ultra_min_width * scale).abs()
                < 0.001,
            "Ultra minimum should reach the compact floor at scale {scale}: {}",
            ultra.window_min_width,
        );
    }
}

#[test]
fn persisted_extreme_drawer_extents_remain_finite_and_inside_the_host() {
    let mut fixture = default_preview_fixture();
    for drawer in fixture
        .layout
        .active_activity_window_mut()
        .unwrap()
        .activity_drawers
        .values_mut()
    {
        drawer.extent = f32::MAX;
    }
    let encoded = crate::ui::workbench::layout_persistence_document::encode_default_layout_value(
        fixture.layout.clone(),
    )
    .expect("extreme finite extents should serialize");
    let restored =
        crate::ui::workbench::layout_persistence_document::decode_default_layout_value(encoded)
            .expect("extreme finite extents should deserialize");
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let geometry = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &restored,
        &fixture.descriptors,
        ShellSizePx::new(640.0, 420.0),
        1.0,
        &WorkbenchChromeMetrics::default(),
        None,
    );

    assert_geometry_is_contained(&geometry, ShellSizePx::new(640.0, 420.0));
}

#[test]
fn tiny_shell_geometry_stays_contained_for_width_height_and_scale_matrix() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    for width in [
        0.0, 1.0, 7.0, 32.0, 120.0, 319.0, 320.0, 479.0, 480.0, 640.0,
    ] {
        for height in [0.0, 1.0, 7.0, 32.0, 120.0, 360.0, 420.0] {
            for scale in [0.5, 1.0, 1.25, 2.0, f32::NAN, f32::INFINITY] {
                let geometry = compute_workbench_shell_geometry(
                    &model,
                    &chrome,
                    &fixture.layout,
                    &fixture.descriptors,
                    ShellSizePx::new(width, height),
                    scale,
                    &metrics,
                    None,
                );
                assert_geometry_is_contained(&geometry, ShellSizePx::new(width, height));
            }
        }
    }
}

fn assert_geometry_is_contained(
    geometry: &crate::ui::workbench::autolayout::WorkbenchShellGeometry,
    size: ShellSizePx,
) {
    let epsilon = 0.01;
    let check = |frame: ShellFrame| {
        assert!(frame.x.is_finite() && frame.y.is_finite());
        assert!(frame.width.is_finite() && frame.height.is_finite());
        assert!(frame.width >= -epsilon && frame.height >= -epsilon);
        assert!(frame.x >= -epsilon && frame.y >= -epsilon);
        assert!(frame.right() <= size.width + epsilon);
        assert!(frame.bottom() <= size.height + epsilon);
    };
    check(geometry.center_band_frame);
    check(geometry.status_bar_frame);
    check(geometry.viewport_content_frame);
    for frame in geometry
        .region_frames
        .values()
        .chain(geometry.splitter_frames.values())
    {
        check(*frame);
    }
    for frame in geometry.floating_window_frames.values() {
        check(*frame);
    }
}
