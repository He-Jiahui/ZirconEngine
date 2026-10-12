use std::cell::Cell;

use super::*;

#[test]
fn binding_expression_evaluator_resolves_values_and_preserves_short_circuiting() {
    let expression = UiBindingExpression::And(
        Box::new(UiBindingExpression::Equals(
            Box::new(UiBindingExpression::ParamRef("expected".to_string())),
            Box::new(UiBindingExpression::PropRef("value".to_string())),
        )),
        Box::new(UiBindingExpression::Or(
            Box::new(UiBindingExpression::Literal(UiValue::Bool(true))),
            Box::new(UiBindingExpression::ControlPropRef {
                control_id: "NeverRead".to_string(),
                property: "value".to_string(),
            }),
        )),
    );
    let control_reads = Cell::new(0usize);

    let value = expression
        .evaluate_with(
            |name| (name == "expected").then_some(UiValue::Int(7)),
            |name| (name == "value").then_some(UiValue::Int(7)),
            |_, _| {
                control_reads.set(control_reads.get() + 1);
                None
            },
        )
        .unwrap();

    assert_eq!(value, UiValue::Bool(true));
    assert_eq!(control_reads.get(), 0);
}

#[test]
fn binding_expression_evaluator_rejects_over_depth_programs_without_recursing() {
    let mut expression = UiBindingExpression::Literal(UiValue::Bool(true));
    for _ in 0..UI_BINDING_EXPRESSION_MAX_DEPTH {
        expression = UiBindingExpression::Not(Box::new(expression));
    }

    assert_eq!(
        expression.evaluate_with(|_| None, |_| None, |_, _| None),
        Err(UiBindingExpressionEvaluationError::BudgetExceeded {
            budget: "depth",
            limit: UI_BINDING_EXPRESSION_MAX_DEPTH,
        })
    );
}
