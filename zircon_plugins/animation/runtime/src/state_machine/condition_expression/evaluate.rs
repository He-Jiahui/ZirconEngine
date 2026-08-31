use zircon_runtime::core::framework::animation::AnimationConditionOperatorAsset;
use zircon_runtime::core::framework::animation::AnimationParameterValue;
use zircon_runtime::core::math::Real;

use super::compiled_condition_expression::CompiledConditionProgram;
use super::instruction::ConditionInstruction;

pub(in crate::state_machine) trait ParameterValueTable {
    fn value(&self, index: usize) -> Option<&AnimationParameterValue>;
}

impl ParameterValueTable for [Option<AnimationParameterValue>] {
    fn value(&self, index: usize) -> Option<&AnimationParameterValue> {
        self.get(index).and_then(Option::as_ref)
    }
}

impl ParameterValueTable for [Option<&AnimationParameterValue>] {
    fn value(&self, index: usize) -> Option<&AnimationParameterValue> {
        self.get(index).copied().flatten()
    }
}

impl CompiledConditionProgram {
    pub(in crate::state_machine) fn evaluate<T>(&self, values: &T) -> bool
    where
        T: ParameterValueTable + ?Sized,
    {
        let Some(last) = self.instructions.len().checked_sub(1) else {
            return false;
        };
        let mut cursor = last;
        evaluate_instruction(&self.instructions, &mut cursor, values)
    }
}

fn evaluate_instruction<T>(
    instructions: &[ConditionInstruction],
    cursor: &mut usize,
    values: &T,
) -> bool
where
    T: ParameterValueTable + ?Sized,
{
    let instruction = &instructions[*cursor];
    match instruction {
        ConditionInstruction::Compare {
            parameter,
            operator,
            value,
        } => compare(values.value(parameter.index()), *operator, value.as_ref()),
        ConditionInstruction::Not => {
            *cursor = cursor.saturating_sub(1);
            !evaluate_instruction(instructions, cursor, values)
        }
        ConditionInstruction::All(count) => evaluate_children(
            instructions,
            cursor,
            values,
            *count as usize,
            true,
            |left, right| left && right,
        ),
        ConditionInstruction::Any(count) => evaluate_children(
            instructions,
            cursor,
            values,
            *count as usize,
            false,
            |left, right| left || right,
        ),
    }
}

fn evaluate_children<T>(
    instructions: &[ConditionInstruction],
    cursor: &mut usize,
    values: &T,
    count: usize,
    identity: bool,
    combine: impl Fn(bool, bool) -> bool,
) -> bool
where
    T: ParameterValueTable + ?Sized,
{
    let mut result = identity;
    for _ in 0..count {
        *cursor = cursor.saturating_sub(1);
        result = combine(result, evaluate_instruction(instructions, cursor, values));
    }
    result
}

fn compare(
    current: Option<&AnimationParameterValue>,
    operator: AnimationConditionOperatorAsset,
    expected: Option<&AnimationParameterValue>,
) -> bool {
    let Some(current) = current.filter(|value| parameter_is_finite(value)) else {
        return false;
    };
    if operator == AnimationConditionOperatorAsset::Triggered {
        return matches!(current, AnimationParameterValue::Trigger);
    }
    let Some(expected) = expected.filter(|value| parameter_is_finite(value)) else {
        return false;
    };
    match operator {
        AnimationConditionOperatorAsset::Equal => current == expected,
        AnimationConditionOperatorAsset::NotEqual => current != expected,
        AnimationConditionOperatorAsset::Greater => {
            compare_numeric(current, expected, |a, b| a > b)
        }
        AnimationConditionOperatorAsset::GreaterEqual => {
            compare_numeric(current, expected, |a, b| a >= b)
        }
        AnimationConditionOperatorAsset::Less => compare_numeric(current, expected, |a, b| a < b),
        AnimationConditionOperatorAsset::LessEqual => {
            compare_numeric(current, expected, |a, b| a <= b)
        }
        AnimationConditionOperatorAsset::Triggered => false,
    }
}

fn numeric(value: &AnimationParameterValue) -> Option<Real> {
    match value {
        AnimationParameterValue::Scalar(value) => Some(*value),
        AnimationParameterValue::Integer(value) => Some(*value as Real),
        AnimationParameterValue::Bool(value) => Some(if *value { 1.0 } else { 0.0 }),
        _ => None,
    }
}

fn compare_numeric(
    current: &AnimationParameterValue,
    expected: &AnimationParameterValue,
    predicate: impl Fn(Real, Real) -> bool,
) -> bool {
    let (Some(current), Some(expected)) = (numeric(current), numeric(expected)) else {
        return false;
    };
    predicate(current, expected)
}

fn parameter_is_finite(value: &AnimationParameterValue) -> bool {
    match value {
        AnimationParameterValue::Scalar(value) => value.is_finite(),
        AnimationParameterValue::Vec2(value) => value.iter().all(|value| value.is_finite()),
        AnimationParameterValue::Vec3(value) => value.iter().all(|value| value.is_finite()),
        AnimationParameterValue::Vec4(value) => value.iter().all(|value| value.is_finite()),
        AnimationParameterValue::Bool(_)
        | AnimationParameterValue::Integer(_)
        | AnimationParameterValue::Trigger => true,
    }
}
