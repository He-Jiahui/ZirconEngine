//! 直接 AST 求值的行为契约测试；不经过 Runtime 的 compiled binding program。

use std::cell::Cell;

use super::*;

/// 参数和属性 resolver 共同提供相等比较值后，真值 `Or` 分支不得读取右侧 control 引用。
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

/// 由公开 AST 变体直接构造的超深表达式必须返回深度预算错误，而不能依赖递归求值。
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
