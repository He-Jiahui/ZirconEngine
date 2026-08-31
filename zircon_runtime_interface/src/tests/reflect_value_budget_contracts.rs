use std::collections::BTreeMap;

use crate::reflect::{
    ReflectValueBudget, ReflectValueFloatKind, ReflectValueValidationError, ReflectedValue,
};

fn budget(
    max_depth: usize,
    max_nodes: usize,
    max_string_bytes: usize,
    max_container_entries: usize,
) -> ReflectValueBudget {
    ReflectValueBudget::new(
        max_depth,
        max_nodes,
        max_string_bytes,
        max_container_entries,
    )
}

#[test]
fn reflected_value_budget_exposes_caller_owned_limits() {
    let budget = budget(7, 11, 13, 17);

    assert_eq!(budget.max_depth(), 7);
    assert_eq!(budget.max_nodes(), 11);
    assert_eq!(budget.max_string_bytes(), 13);
    assert_eq!(budget.max_container_entries(), 17);
}

#[test]
fn reflected_value_budget_counts_reflected_and_embedded_json_graphs() {
    let value = ReflectedValue::Map(BTreeMap::from([
        (
            "json".to_string(),
            ReflectedValue::Json(serde_json::json!({ "label": ["ok"] })),
        ),
        (
            "list".to_string(),
            ReflectedValue::List(vec![ReflectedValue::Unsigned(3)]),
        ),
    ]));

    assert_eq!(value.validate_with_budget(budget(6, 9, 19, 2)), Ok(()));
}

#[test]
fn reflected_value_budget_rejects_each_bounded_dimension() {
    let depth = ReflectedValue::List(vec![ReflectedValue::List(vec![ReflectedValue::Null])]);
    assert_eq!(
        depth.validate_with_budget(budget(2, 8, 8, 8)),
        Err(ReflectValueValidationError::DepthExceeded {
            actual: 3,
            maximum: 2,
        })
    );

    let nodes = ReflectedValue::List(vec![ReflectedValue::Null, ReflectedValue::Null]);
    assert_eq!(
        nodes.validate_with_budget(budget(4, 2, 8, 8)),
        Err(ReflectValueValidationError::NodeBudgetExceeded {
            actual: 3,
            maximum: 2,
        })
    );

    let strings = ReflectedValue::Map(BTreeMap::from([(
        "aa".to_string(),
        ReflectedValue::String("bbb".to_string()),
    )]));
    assert_eq!(
        strings.validate_with_budget(budget(4, 8, 4, 8)),
        Err(ReflectValueValidationError::StringBudgetExceeded {
            actual: 5,
            maximum: 4,
        })
    );

    let container = ReflectedValue::List(vec![ReflectedValue::Null, ReflectedValue::Null]);
    assert_eq!(
        container.validate_with_budget(budget(4, 8, 8, 1)),
        Err(ReflectValueValidationError::ContainerEntriesExceeded {
            actual: 2,
            maximum: 1,
        })
    );
}

#[test]
fn reflected_value_budget_rejects_non_finite_vector_components() {
    let value = ReflectedValue::Quaternion([0.0, f32::NAN, 0.0, 1.0]);

    assert_eq!(
        value.validate_with_budget(budget(2, 2, 0, 0)),
        Err(ReflectValueValidationError::NonFiniteFloat {
            kind: ReflectValueFloatKind::Quaternion,
            component: 1,
        })
    );
}

#[test]
fn embedded_json_root_is_a_child_of_the_reflected_json_wrapper() {
    let value = ReflectedValue::Json(serde_json::json!([null]));

    assert_eq!(
        value.validate_with_budget(budget(2, 8, 0, 8)),
        Err(ReflectValueValidationError::DepthExceeded {
            actual: 3,
            maximum: 2,
        })
    );
}
