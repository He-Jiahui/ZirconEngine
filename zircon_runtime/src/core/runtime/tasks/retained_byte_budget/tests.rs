use super::*;

#[test]
fn cloned_lease_releases_bytes_only_after_the_last_owner_drops() {
    let budget = RetainedByteBudget::new(8);
    let lease = budget.try_reserve(8).unwrap();
    let clone = lease.clone();

    assert_eq!(budget.diagnostics().retained_bytes, 8);
    assert!(matches!(
        budget.try_reserve(1),
        Err(RetainedByteBudgetError::CapacityExceeded {
            requested_bytes: 1,
            remaining_bytes: 0,
        })
    ));

    drop(lease);
    assert_eq!(budget.diagnostics().retained_bytes, 8);
    drop(clone);
    assert_eq!(budget.diagnostics().retained_bytes, 0);
    assert_eq!(budget.diagnostics().active_leases, 0);
}

#[test]
fn closed_budget_rejects_new_reservations_without_revoking_live_leases() {
    let budget = RetainedByteBudget::new(16);
    let lease = budget.try_reserve(7).unwrap();

    budget.close();

    assert!(matches!(
        budget.try_reserve(1),
        Err(RetainedByteBudgetError::Closed)
    ));
    assert_eq!(budget.diagnostics().retained_bytes, 7);
    assert!(budget.diagnostics().closed);

    drop(lease);
    assert_eq!(budget.diagnostics().retained_bytes, 0);
}

#[test]
fn oversized_reservation_reports_the_exact_remaining_capacity() {
    let budget = RetainedByteBudget::new(10);
    let _lease = budget.try_reserve(4).unwrap();

    assert!(matches!(
        budget.try_reserve(7),
        Err(RetainedByteBudgetError::CapacityExceeded {
            requested_bytes: 7,
            remaining_bytes: 6,
        })
    ));
}

#[test]
fn lease_capacity_bounds_many_small_retained_results() {
    let budget = RetainedByteBudget::with_max_leases(1024, 1);
    let _lease = budget.try_reserve(1).unwrap();

    assert!(matches!(
        budget.try_reserve(1),
        Err(RetainedByteBudgetError::LeaseCapacityExceeded { maximum_leases: 1 })
    ));
    assert_eq!(budget.diagnostics().capacity_leases, 1);
}
