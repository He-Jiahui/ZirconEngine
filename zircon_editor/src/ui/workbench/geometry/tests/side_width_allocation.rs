use super::*;

#[test]
fn larger_side_releases_width_before_balanced_panels_shrink_together() {
    assert_eq!(
        balanced_side_widths_for_budget(278.0, 186.0, 378.0),
        BalancedSideWidths {
            left: 192.0,
            right: 186.0
        }
    );
    assert_eq!(
        balanced_side_widths_for_budget(340.0, 220.0, 448.0),
        BalancedSideWidths {
            left: 228.0,
            right: 220.0
        }
    );
}

#[test]
fn authored_side_widths_survive_when_the_document_budget_already_fits() {
    assert_eq!(
        balanced_side_widths_for_budget(278.0, 274.0, 558.0),
        BalancedSideWidths {
            left: 278.0,
            right: 274.0
        }
    );
}

#[test]
fn maximum_finite_side_demands_stay_inside_the_host_budget() {
    let widths = balanced_side_widths_for_budget(f32::MAX, f32::MAX, 378.0);

    assert!(widths.left.is_finite());
    assert!(widths.right.is_finite());
    assert!(widths.left >= 0.0);
    assert!(widths.right >= 0.0);
    assert!(widths.left + widths.right <= 378.001);
}

#[test]
fn non_finite_side_demands_cannot_poison_the_allocation_boundary() {
    let widths = balanced_side_widths_for_budget(f32::INFINITY, f32::NAN, 120.0);

    assert_eq!(widths.left, 120.0);
    assert_eq!(widths.right, 0.0);
}
