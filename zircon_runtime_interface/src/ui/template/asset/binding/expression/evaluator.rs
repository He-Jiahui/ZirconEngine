use std::fmt;

use crate::ui::component::UiValue;

use super::{
    UiBindingExpression, UI_BINDING_EXPRESSION_INLINE_STACK_CAPACITY,
    UI_BINDING_EXPRESSION_MAX_DEPTH, UI_BINDING_EXPRESSION_MAX_NODES,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiBindingExpressionEvaluationError {
    UnresolvedParameter(String),
    UnresolvedProperty(String),
    UnresolvedControlProperty {
        control_id: String,
        property: String,
    },
    ExpectedBoolean,
    BudgetExceeded {
        budget: &'static str,
        limit: usize,
    },
    InvalidProgram,
}

impl fmt::Display for UiBindingExpressionEvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnresolvedParameter(name) => {
                write!(
                    f,
                    "binding expression parameter {name} could not be resolved"
                )
            }
            Self::UnresolvedProperty(name) => {
                write!(
                    f,
                    "binding expression property {name} could not be resolved"
                )
            }
            Self::UnresolvedControlProperty {
                control_id,
                property,
            } => write!(
                f,
                "binding expression control property {control_id}.prop.{property} could not be resolved"
            ),
            Self::ExpectedBoolean => {
                f.write_str("binding expression boolean operator requires a boolean value")
            }
            Self::BudgetExceeded { budget, limit } => {
                write!(f, "binding expression exceeds {budget} budget of {limit}")
            }
            Self::InvalidProgram => f.write_str("binding expression program is invalid"),
        }
    }
}

impl std::error::Error for UiBindingExpressionEvaluationError {}

#[derive(Clone, Copy)]
enum BinaryOperator {
    Equals,
    NotEquals,
}

enum EvaluationFrame<'a> {
    Enter(&'a UiBindingExpression, usize),
    BinaryAfterLeft {
        operator: BinaryOperator,
        right: &'a UiBindingExpression,
        depth: usize,
    },
    BinaryAfterRight {
        operator: BinaryOperator,
        left: UiValue,
    },
    AndAfterLeft {
        right: &'a UiBindingExpression,
        depth: usize,
    },
    AndAfterRight,
    OrAfterLeft {
        right: &'a UiBindingExpression,
        depth: usize,
    },
    OrAfterRight,
    NotAfterValue,
}

struct EvaluationStack<T> {
    inline: [Option<T>; UI_BINDING_EXPRESSION_INLINE_STACK_CAPACITY],
    inline_len: usize,
    spill: Vec<T>,
}

impl<T> EvaluationStack<T> {
    fn new() -> Self {
        Self {
            inline: std::array::from_fn(|_| None),
            inline_len: 0,
            spill: Vec::new(),
        }
    }

    fn push(&mut self, value: T) -> Result<(), UiBindingExpressionEvaluationError> {
        if self.len() >= UI_BINDING_EXPRESSION_MAX_DEPTH + 1 {
            return Err(budget_error("depth", UI_BINDING_EXPRESSION_MAX_DEPTH));
        }
        if self.spill.is_empty() && self.inline_len < self.inline.len() {
            self.inline[self.inline_len] = Some(value);
            self.inline_len += 1;
        } else {
            self.spill.push(value);
        }
        Ok(())
    }

    fn pop(&mut self) -> Option<T> {
        if let Some(value) = self.spill.pop() {
            return Some(value);
        }
        self.inline_len = self.inline_len.checked_sub(1)?;
        self.inline[self.inline_len].take()
    }

    fn len(&self) -> usize {
        self.inline_len + self.spill.len()
    }
}

impl UiBindingExpression {
    pub fn evaluate_with<Param, Property, ControlProperty>(
        &self,
        mut resolve_param: Param,
        mut resolve_property: Property,
        mut resolve_control_property: ControlProperty,
    ) -> Result<UiValue, UiBindingExpressionEvaluationError>
    where
        Param: FnMut(&str) -> Option<UiValue>,
        Property: FnMut(&str) -> Option<UiValue>,
        ControlProperty: FnMut(&str, &str) -> Option<UiValue>,
    {
        let mut frames = EvaluationStack::new();
        frames.push(EvaluationFrame::Enter(self, 1))?;
        let mut values = EvaluationStack::new();
        let mut visited_nodes = 0usize;
        while let Some(frame) = frames.pop() {
            match frame {
                EvaluationFrame::Enter(expression, depth) => {
                    visited_nodes += 1;
                    if visited_nodes > UI_BINDING_EXPRESSION_MAX_NODES {
                        return Err(budget_error("nodes", UI_BINDING_EXPRESSION_MAX_NODES));
                    }
                    if depth > UI_BINDING_EXPRESSION_MAX_DEPTH {
                        return Err(budget_error("depth", UI_BINDING_EXPRESSION_MAX_DEPTH));
                    }
                    match expression {
                        UiBindingExpression::Literal(value) => values.push(value.clone())?,
                        UiBindingExpression::ParamRef(name) => {
                            values.push(resolve_param(name).ok_or_else(|| {
                                UiBindingExpressionEvaluationError::UnresolvedParameter(
                                    name.clone(),
                                )
                            })?)?
                        }
                        UiBindingExpression::PropRef(name) => {
                            values.push(resolve_property(name).ok_or_else(|| {
                                UiBindingExpressionEvaluationError::UnresolvedProperty(name.clone())
                            })?)?
                        }
                        UiBindingExpression::ControlPropRef {
                            control_id,
                            property,
                        } => {
                            values
                                .push(resolve_control_property(control_id, property).ok_or_else(
                                || UiBindingExpressionEvaluationError::UnresolvedControlProperty {
                                    control_id: control_id.clone(),
                                    property: property.clone(),
                                },
                            )?)?
                        }
                        UiBindingExpression::Equals(left, right) => {
                            frames.push(EvaluationFrame::BinaryAfterLeft {
                                operator: BinaryOperator::Equals,
                                right,
                                depth: depth + 1,
                            })?;
                            frames.push(EvaluationFrame::Enter(left, depth + 1))?;
                        }
                        UiBindingExpression::NotEquals(left, right) => {
                            frames.push(EvaluationFrame::BinaryAfterLeft {
                                operator: BinaryOperator::NotEquals,
                                right,
                                depth: depth + 1,
                            })?;
                            frames.push(EvaluationFrame::Enter(left, depth + 1))?;
                        }
                        UiBindingExpression::And(left, right) => {
                            frames.push(EvaluationFrame::AndAfterLeft {
                                right,
                                depth: depth + 1,
                            })?;
                            frames.push(EvaluationFrame::Enter(left, depth + 1))?;
                        }
                        UiBindingExpression::Or(left, right) => {
                            frames.push(EvaluationFrame::OrAfterLeft {
                                right,
                                depth: depth + 1,
                            })?;
                            frames.push(EvaluationFrame::Enter(left, depth + 1))?;
                        }
                        UiBindingExpression::Not(value) => {
                            frames.push(EvaluationFrame::NotAfterValue)?;
                            frames.push(EvaluationFrame::Enter(value, depth + 1))?;
                        }
                    }
                }
                EvaluationFrame::BinaryAfterLeft {
                    operator,
                    right,
                    depth,
                } => {
                    let left = pop_value(&mut values)?;
                    frames.push(EvaluationFrame::BinaryAfterRight { operator, left })?;
                    frames.push(EvaluationFrame::Enter(right, depth))?;
                }
                EvaluationFrame::BinaryAfterRight { operator, left } => {
                    let right = pop_value(&mut values)?;
                    values.push(UiValue::Bool(match operator {
                        BinaryOperator::Equals => left == right,
                        BinaryOperator::NotEquals => left != right,
                    }))?;
                }
                EvaluationFrame::AndAfterLeft { right, depth } => {
                    if !bool_value(pop_value(&mut values)?)? {
                        values.push(UiValue::Bool(false))?;
                    } else {
                        frames.push(EvaluationFrame::AndAfterRight)?;
                        frames.push(EvaluationFrame::Enter(right, depth))?;
                    }
                }
                EvaluationFrame::AndAfterRight => {
                    let value = bool_value(pop_value(&mut values)?)?;
                    values.push(UiValue::Bool(value))?;
                }
                EvaluationFrame::OrAfterLeft { right, depth } => {
                    if bool_value(pop_value(&mut values)?)? {
                        values.push(UiValue::Bool(true))?;
                    } else {
                        frames.push(EvaluationFrame::OrAfterRight)?;
                        frames.push(EvaluationFrame::Enter(right, depth))?;
                    }
                }
                EvaluationFrame::OrAfterRight => {
                    let value = bool_value(pop_value(&mut values)?)?;
                    values.push(UiValue::Bool(value))?;
                }
                EvaluationFrame::NotAfterValue => {
                    let value = !bool_value(pop_value(&mut values)?)?;
                    values.push(UiValue::Bool(value))?;
                }
            }
        }
        if values.len() == 1 {
            pop_value(&mut values)
        } else {
            Err(UiBindingExpressionEvaluationError::InvalidProgram)
        }
    }
}

fn pop_value(
    values: &mut EvaluationStack<UiValue>,
) -> Result<UiValue, UiBindingExpressionEvaluationError> {
    values
        .pop()
        .ok_or(UiBindingExpressionEvaluationError::InvalidProgram)
}

fn bool_value(value: UiValue) -> Result<bool, UiBindingExpressionEvaluationError> {
    match value {
        UiValue::Bool(value) => Ok(value),
        _ => Err(UiBindingExpressionEvaluationError::ExpectedBoolean),
    }
}

fn budget_error(budget: &'static str, limit: usize) -> UiBindingExpressionEvaluationError {
    UiBindingExpressionEvaluationError::BudgetExceeded { budget, limit }
}

#[cfg(test)]
mod tests {
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
}
