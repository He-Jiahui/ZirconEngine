use zircon_editor::{
    compute_workbench_shell_geometry, default_preview_fixture, solve_axis_constraints,
    AxisConstraint, ShellRegionId, ShellSizePx, StretchMode, WorkbenchChromeMetrics,
    WorkbenchViewModel,
};

fn stretch_constraint(min: f32, preferred: f32, priority: i32, weight: f32) -> AxisConstraint {
    AxisConstraint {
        min,
        max: -1.0,
        preferred,
        priority,
        weight,
        stretch_mode: StretchMode::Stretch,
    }
}

#[test]
fn axis_solver_grows_higher_priority_regions_before_lower_priority_regions() {
    let resolved = solve_axis_constraints(
        900.0,
        &[
            stretch_constraint(200.0, 300.0, 100, 3.0),
            stretch_constraint(180.0, 220.0, 50, 1.0),
            stretch_constraint(180.0, 220.0, 50, 1.0),
        ],
    );

    assert_eq!(resolved.len(), 3);
    assert!(resolved[0].resolved > 300.0);
    assert_eq!(resolved[1].resolved, 220.0);
    assert_eq!(resolved[2].resolved, 220.0);
}

#[test]
fn axis_solver_clamps_to_hard_minimums_when_available_is_below_total_minimum() {
    let resolved = solve_axis_constraints(
        700.0,
        &[
            stretch_constraint(480.0, 540.0, 100, 3.0),
            stretch_constraint(220.0, 300.0, 50, 1.0),
            stretch_constraint(120.0, 180.0, 25, 1.0),
        ],
    );

    assert_eq!(resolved.len(), 3);
    assert_eq!(resolved[0].resolved, 480.0);
    assert_eq!(resolved[1].resolved, 220.0);
    assert_eq!(resolved[2].resolved, 120.0);
}

#[test]
fn default_hybrid_geometry_anchors_right_and_bottom_regions() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let geometry = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(1440.0, 900.0),
        &WorkbenchChromeMetrics::default(),
        None,
    );

    let right = geometry.region_frame(ShellRegionId::Right);
    let bottom = geometry.region_frame(ShellRegionId::Bottom);
    let document = geometry.region_frame(ShellRegionId::Document);

    assert!(right.width > 0.0);
    assert_eq!(right.x + right.width, 1440.0);
    assert!(bottom.height > 0.0);
    assert_eq!(
        bottom.y + bottom.height + geometry.status_bar_frame.height + 1.0,
        900.0
    );
    assert_eq!(document.x + document.width + right.width + 1.0, 1440.0);
}

#[test]
fn geometry_viewport_frame_is_derived_from_shell_layout_not_snapshot_viewport_size() {
    let mut fixture = default_preview_fixture();
    fixture.editor.viewport_size = [32, 32];

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let geometry = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(1600.0, 980.0),
        &WorkbenchChromeMetrics::default(),
        None,
    );

    assert!(geometry.viewport_content_frame.width > 32.0);
    assert!(geometry.viewport_content_frame.height > 32.0);
}
