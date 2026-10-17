use super::*;

fn normal_model_or_assert_arbitrary_precision_refusal() -> Option<RetainedAccountingModelV1> {
    // This is the compiled normal dependency representation, not a host feature flag.
    if needs_drop::<serde_json::Number>() {
        assert!(matches!(
            verified_std_model_v1(),
            Err(RetentionContractError::UnsupportedConfiguration { .. })
        ));
        return None;
    }
    Some(verified_std_model_v1().expect("admitted ordinary-number Windows model must be available"))
}

#[test]
fn retained_owner_bounds_count_unused_capacity_and_reject_overflow() {
    let Some(model) = normal_model_or_assert_arbitrary_precision_refusal() else {
        return;
    };
    let mut text = String::with_capacity(4096);
    text.push('x');
    let bound = retained_bound_v1(&text, model).unwrap().max_bytes.get();
    assert!(bound >= (size_of::<String>() + text.capacity()) as u64);
    let mut overflow = RetainedBoundBuilderV1::new(model);
    overflow.add(u64::MAX).unwrap();
    assert_eq!(overflow.add(1), Err(RetentionContractError::Overflow));
    assert_eq!(
        align_up_v1(usize::MAX, 8),
        Err(RetentionContractError::Overflow)
    );

    fn nested(levels: usize) -> serde_json::Value {
        let mut value = serde_json::Value::Null;
        for _ in 0..levels {
            value = serde_json::Value::Array(vec![value]);
        }
        value
    }
    let boundary = nested(MAX_RETAINED_DEPTH_V1 - 1);
    assert!(retained_bound_v1(&boundary, model).is_ok());
    let too_deep = nested(MAX_RETAINED_DEPTH_V1);
    let mut meter = RetainedBoundBuilderV1::new(model);
    assert_eq!(
        meter.owned(&too_deep),
        Err(RetentionContractError::DepthLimit {
            limit: MAX_RETAINED_DEPTH_V1
        })
    );
    assert_eq!(meter.depth, 0, "Err must restore nesting before reuse");
    meter.owned(&serde_json::Value::Null).unwrap();

    // Capacity is billed before child traversal; do not first walk a huge tree.
    let values = serde_json::Value::Array(Vec::with_capacity(4096));
    let mut limited = RetainedBoundBuilderV1::with_limit(model, RetainedBytesV1::new(64));
    assert!(matches!(
        limited.owned(&values),
        Err(RetentionContractError::Budget { .. })
    ));
    assert_eq!(limited.depth, 0);
    let enormous_zst = vec![(); MAX_RETAINED_VISITS_V1 + 1];
    assert_eq!(
        retained_bound_v1(&enormous_zst, model),
        Err(RetentionContractError::WorkLimit {
            limit: MAX_RETAINED_VISITS_V1
        })
    );
}

#[test]
fn bounded_shared_state_cannot_grow_past_its_retention_owner() {
    let Some(model) = normal_model_or_assert_arbitrary_precision_refusal() else {
        return;
    };
    let state = Arc::new(
        BoundedRetainedStateV1::try_new(String::from("small"), RetainedBytesV1::new(128), model)
            .unwrap(),
    );
    let before = retained_bound_v1(&state, model).unwrap();
    let alias = Arc::clone(&state);
    assert!(matches!(
        alias.try_replace("x".repeat(4096)),
        Err(RetentionContractError::Budget { .. })
    ));
    assert_eq!(state.with(String::clone), "small");
    let old = alias.try_replace(String::from("next")).unwrap();
    drop(old);
    assert_eq!(retained_bound_v1(&state, model).unwrap(), before);
}
