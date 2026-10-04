use super::*;

fn limit(max_encoded_bytes: usize, max_nesting_depth: usize) -> ZrRuntimePayloadLimitV1 {
    ZrRuntimePayloadLimitV1 {
        max_encoded_bytes,
        max_items: 16,
        max_nesting_depth,
        max_processing_time_micros: 100_000,
        allow_empty: false,
    }
}

#[test]
fn build_budget_accumulates_serialized_bytes_before_retention() {
    let mut budget = AccessibilityBuildBudget::new(limit(8, 8));
    budget.observe_value("1234", 2).unwrap();

    let error = budget
        .observe_value("a", 2)
        .expect_err("the second value must exceed the cumulative byte budget");

    assert_eq!(
        error,
        AccessibilitySnapshotBudgetError::EncodedBytes {
            observed: 9,
            limit: 8
        }
    );
}

#[test]
fn build_budget_accounts_for_the_snapshot_nesting_offset() {
    let mut budget = AccessibilityBuildBudget::new(limit(1024, 2));

    let error = budget
        .observe_value(&serde_json::json!({"value": 1}), 2)
        .expect_err("the value object must exceed the remaining nesting budget");

    assert_eq!(
        error,
        AccessibilitySnapshotBudgetError::NestingDepth {
            observed: 3,
            limit: 2
        }
    );
}
