use super::*;

const SATURATED_OBSERVER_BUDGET: usize = 32;

fn observer() -> TerminalObserver {
    Box::new(|_| {})
}

#[test]
fn observer_queue_reservation_preserves_small_fan_out_and_jumps_to_budget() {
    let mut observers = Vec::new();
    reserve_remaining_observer_budget_if_full(&mut observers, SATURATED_OBSERVER_BUDGET);
    assert_eq!(observers.capacity(), 0);

    observers.push(observer());
    while observers.len() < observers.capacity() {
        reserve_remaining_observer_budget_if_full(&mut observers, SATURATED_OBSERVER_BUDGET);
        observers.push(observer());
    }
    assert!(observers.capacity() < SATURATED_OBSERVER_BUDGET);

    reserve_remaining_observer_budget_if_full(&mut observers, SATURATED_OBSERVER_BUDGET);
    assert!(observers.capacity() >= SATURATED_OBSERVER_BUDGET);
}
