use zircon_runtime_interface::ui::layout::{AxisConstraint, StretchMode};

use super::{solve_axis_constraints, solve_axis_constraints_into};

#[test]
fn reusable_solver_workspace_matches_owned_results_and_preserves_capacity() {
    let constraints = [constraint(2, 1.0), constraint(1, 2.0), constraint(1, 1.0)];
    let mut resolved = Vec::new();
    let mut priorities = Vec::new();
    let mut active_indices = Vec::new();

    for available in [75.0, 12.0] {
        let expected = solve_axis_constraints(available, &constraints);
        solve_axis_constraints_into(
            available,
            &constraints,
            &mut resolved,
            &mut priorities,
            &mut active_indices,
        );
        assert_eq!(resolved, expected);
    }
    let first_capacity = (
        resolved.capacity(),
        priorities.capacity(),
        active_indices.capacity(),
    );

    for available in [75.0, 12.0] {
        solve_axis_constraints_into(
            available,
            &constraints,
            &mut resolved,
            &mut priorities,
            &mut active_indices,
        );
    }

    assert_eq!(
        (
            resolved.capacity(),
            priorities.capacity(),
            active_indices.capacity(),
        ),
        first_capacity
    );
}

#[test]
fn solver_branch_workspaces_reserve_constraint_upper_bound() {
    let source = include_str!("../constraints.rs");
    let implementation = source
        .split("mod tests {")
        .next()
        .expect("constraints implementation before tests");

    assert_eq!(
        implementation
            .matches("priorities.reserve(resolved.len());")
            .count(),
        2,
        "both priority-ordering branches should reserve their constraint bound"
    );
    assert_eq!(
        implementation
            .matches("active_indices.reserve(resolved.len());")
            .count(),
        2,
        "both distribution branches should reserve their constraint bound"
    );
}

#[test]
fn reusable_solver_growth_respects_priority_and_max_saturation() {
    let constraints = [
        axis(0.0, 15.0, 10.0, 2, 1.0, StretchMode::Stretch),
        axis(0.0, 100.0, 10.0, 1, 1.0, StretchMode::Stretch),
    ];

    assert_eq!(solve_reused(40.0, &constraints), vec![15.0, 25.0]);
}

#[test]
fn reusable_solver_growth_with_zero_weights_shares_evenly() {
    let constraints = [
        axis(0.0, 100.0, 10.0, 0, 0.0, StretchMode::Stretch),
        axis(0.0, 100.0, 10.0, 0, 0.0, StretchMode::Stretch),
    ];

    assert_eq!(solve_reused(30.0, &constraints), vec![15.0, 15.0]);
}

#[test]
fn reusable_solver_shrink_respects_ascending_priority() {
    let constraints = [
        axis(5.0, 100.0, 20.0, 0, 1.0, StretchMode::Fixed),
        axis(5.0, 100.0, 20.0, 1, 1.0, StretchMode::Fixed),
    ];

    assert_eq!(solve_reused(25.0, &constraints), vec![5.0, 20.0]);
}

#[test]
fn reusable_solver_exact_fit_and_minimum_floor_are_explicit() {
    let exact = [
        axis(0.0, 100.0, 10.0, 0, 1.0, StretchMode::Fixed),
        axis(0.0, 100.0, 20.0, 0, 1.0, StretchMode::Fixed),
    ];
    let minimum_floor = [
        axis(8.0, 100.0, 8.0, 0, 1.0, StretchMode::Fixed),
        axis(8.0, 100.0, 8.0, 0, 1.0, StretchMode::Fixed),
    ];

    assert_eq!(solve_reused(30.0, &exact), vec![10.0, 20.0]);
    assert_eq!(solve_reused(5.0, &minimum_floor), vec![8.0, 8.0]);
}

fn constraint(priority: i32, weight: f32) -> AxisConstraint {
    axis(1.0, 30.0, 10.0, priority, weight, StretchMode::Stretch)
}

fn solve_reused(available: f32, constraints: &[AxisConstraint]) -> Vec<f32> {
    let mut resolved = Vec::new();
    solve_axis_constraints_into(
        available,
        constraints,
        &mut resolved,
        &mut Vec::new(),
        &mut Vec::new(),
    );
    resolved.into_iter().map(|axis| axis.resolved).collect()
}

fn axis(
    min: f32,
    max: f32,
    preferred: f32,
    priority: i32,
    weight: f32,
    stretch_mode: StretchMode,
) -> AxisConstraint {
    AxisConstraint {
        min,
        max,
        preferred,
        priority,
        weight,
        stretch_mode,
    }
}
