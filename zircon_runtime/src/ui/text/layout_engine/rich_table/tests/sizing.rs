use crate::text::{RichTableColumn, TextLayoutAxisConstraint, TextLayoutGeometryBudget};

use super::{fit_columns_to_available_width, resolve_column_extents};

fn shrinkable() -> RichTableColumn {
    RichTableColumn::default()
}

fn fixed() -> RichTableColumn {
    RichTableColumn {
        shrink: false,
        ..RichTableColumn::default()
    }
}

fn budget() -> TextLayoutGeometryBudget {
    TextLayoutGeometryBudget::new(1_000.0, 4_000.0).expect("valid test budget")
}

#[test]
fn shrink_budget_accounts_for_columns_that_reach_the_minimum() {
    let mut widths = [20.0, 190.0];
    fit_columns_to_available_width(
        &mut widths,
        &[shrinkable(), shrinkable()],
        TextLayoutAxisConstraint::Bounded(100.0),
        20.0,
        budget(),
    )
    .expect("valid geometry");

    assert!((widths[0] - 20.0).abs() < 0.001);
    assert!((widths[1] - 80.0).abs() < 0.001);
    assert!((widths.iter().sum::<f32>() - 100.0).abs() < 0.001);
}

#[test]
fn shrink_budget_excludes_fixed_columns_before_solving_lower_bounds() {
    let mut widths = [70.0, 40.0, 160.0];
    fit_columns_to_available_width(
        &mut widths,
        &[fixed(), shrinkable(), shrinkable()],
        TextLayoutAxisConstraint::Bounded(150.0),
        20.0,
        budget(),
    )
    .expect("valid geometry");

    assert!((widths[0] - 70.0).abs() < 0.001);
    assert!((widths[1] - 20.0).abs() < 0.001);
    assert!((widths[2] - 60.0).abs() < 0.001);
    assert!((widths.iter().sum::<f32>() - 150.0).abs() < 0.001);
}

#[test]
fn minimum_width_wins_when_the_available_budget_is_infeasible() {
    let mut widths = [60.0, 80.0, 120.0];
    fit_columns_to_available_width(
        &mut widths,
        &[fixed(), shrinkable(), shrinkable()],
        TextLayoutAxisConstraint::Bounded(80.0),
        20.0,
        budget(),
    )
    .expect("valid geometry");

    assert_eq!(widths, [60.0, 20.0, 20.0]);
}

#[test]
fn unbounded_available_width_retains_natural_column_extents() {
    let widths = resolve_column_extents(
        &[shrinkable(), shrinkable()],
        &[],
        TextLayoutAxisConstraint::Unbounded,
        4.0,
        20.0,
        budget(),
    )
    .expect("valid geometry");

    assert_eq!(widths, vec![20.0, 20.0]);
}
