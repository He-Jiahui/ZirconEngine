use super::{
    finite_f32_or_geometry, finite_geometry, finite_sum, FiniteGeometryAccumulator,
    TextLayoutAxisConstraint, TextLayoutGeometryBudget, DEFAULT_MAX_EXACT_LAYOUT_EXTENT,
};

#[test]
fn finite_geometry_helpers_preserve_normal_f32_and_clamp_overflow() {
    assert_eq!(finite_f32_or_geometry(0.1_f32 + 0.2, 0.3), 0.1_f32 + 0.2);
    assert_eq!(finite_geometry(f64::NAN), 0.0);
    assert_eq!(finite_geometry(f64::INFINITY), f32::MAX);
    assert_eq!(finite_geometry(f64::NEG_INFINITY), -f32::MAX);
    assert_eq!(finite_sum([f32::MAX, f32::MAX]), f32::MAX);
}

#[test]
fn finite_accumulator_preserves_exact_history_after_overflow_recovery() {
    let mut sum = FiniteGeometryAccumulator::default();

    assert_eq!(sum.add(f32::MAX), f32::MAX);
    assert_eq!(sum.add(f32::MAX), f32::MAX);
    assert_eq!(sum.add(-f32::MAX), f32::MAX);
    assert_eq!(sum.add(-f32::MAX), 0.0);
    assert_eq!(sum.value(), 0.0);
    assert_eq!(sum.exact(), 0.0);
    assert_eq!(finite_sum([f32::MAX, f32::MAX, -f32::MAX]), f32::MAX);
}

#[test]
fn default_budget_uses_the_f32_exact_integer_boundary() {
    let budget = TextLayoutGeometryBudget::default();

    assert_eq!(budget.max_axis_extent(), DEFAULT_MAX_EXACT_LAYOUT_EXTENT);
    assert_eq!(
        budget.max_accumulated_extent(),
        DEFAULT_MAX_EXACT_LAYOUT_EXTENT
    );
}

#[test]
fn budget_rejects_non_finite_negative_and_oversized_axis_extents() {
    let budget = TextLayoutGeometryBudget::new(100.0, 200.0).expect("valid budget");

    assert_eq!(budget.admit_axis_extent(100.0), Ok(100.0));
    assert!(budget.admit_axis_extent(f32::NAN).is_err());
    assert!(budget.admit_axis_extent(f32::INFINITY).is_err());
    assert!(budget.admit_axis_extent(-1.0).is_err());
    assert!(budget.admit_axis_extent(100.5).is_err());
}

#[test]
fn accumulated_arithmetic_rejects_invalid_operands_and_budget_overflow() {
    let budget = TextLayoutGeometryBudget::new(100.0, 200.0).expect("valid budget");

    assert_eq!(budget.checked_add_accumulated(80.0, 120.0), Ok(200.0));
    assert!(budget.checked_add_accumulated(-10.0, 20.0).is_err());
    assert!(budget.checked_add_accumulated(120.0, 81.0).is_err());
    assert_eq!(budget.checked_scale_accumulated(20.0, 10), Ok(200.0));
    assert!(budget.checked_scale_accumulated(20.0, 11).is_err());
    assert!(budget.checked_scale_accumulated(f32::INFINITY, 0).is_err());
}

#[test]
fn constructor_requires_a_positive_finite_ordered_budget() {
    assert!(TextLayoutGeometryBudget::new(0.0, 1.0).is_none());
    assert!(TextLayoutGeometryBudget::new(2.0, 1.0).is_none());
    assert!(TextLayoutGeometryBudget::new(1.0, f32::INFINITY).is_none());
    assert!(TextLayoutGeometryBudget::new(f32::NAN, 1.0).is_none());
}

#[test]
fn positive_infinity_is_only_admitted_as_request_metadata() {
    let budget = TextLayoutGeometryBudget::new(100.0, 200.0).expect("valid budget");

    assert_eq!(
        TextLayoutAxisConstraint::from_request_extent(f32::INFINITY, budget),
        Ok(TextLayoutAxisConstraint::Unbounded)
    );
    assert_eq!(
        TextLayoutAxisConstraint::from_request_extent(75.0, budget),
        Ok(TextLayoutAxisConstraint::Bounded(75.0))
    );
    assert!(TextLayoutAxisConstraint::from_request_extent(f32::NEG_INFINITY, budget).is_err());
    assert!(TextLayoutAxisConstraint::from_request_extent(f32::NAN, budget).is_err());
    assert!(budget.admit_axis_extent(f32::INFINITY).is_err());
}
