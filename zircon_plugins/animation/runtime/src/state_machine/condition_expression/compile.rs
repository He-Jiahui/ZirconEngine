use zircon_runtime::core::framework::animation::compiler::state_machine::AnimationCompiledTransitionCondition;
use zircon_runtime::core::framework::animation::AnimationTransitionConditionAsset;

use super::compiled_condition_expression::{CompiledConditionExpression, CompiledConditionProgram};
use super::instruction::ConditionInstruction;
use super::{ConditionExpression, ConditionExpressionCompileError, ParameterTableBuilder};

const MAX_CONDITION_EXPRESSION_DEPTH: usize = 64;

impl CompiledConditionExpression {
    pub fn compile(
        expression: &ConditionExpression,
    ) -> Result<Self, ConditionExpressionCompileError> {
        let mut parameters = ParameterTableBuilder::default();
        let program = compile_program(expression, &mut parameters)?;
        Ok(Self {
            parameter_names: parameters.finish(),
            program,
        })
    }
}

pub(super) fn compile_program(
    expression: &ConditionExpression,
    parameters: &mut ParameterTableBuilder,
) -> Result<CompiledConditionProgram, ConditionExpressionCompileError> {
    let mut instructions = Vec::new();
    compile_node(expression, parameters, &mut instructions, 1)?;
    Ok(CompiledConditionProgram {
        instructions: instructions.into_boxed_slice(),
    })
}

pub(in crate::state_machine) fn compile_all_conditions(
    conditions: &[AnimationTransitionConditionAsset],
    parameters: &mut ParameterTableBuilder,
) -> Result<CompiledConditionProgram, ConditionExpressionCompileError> {
    let expression = ConditionExpression::all(
        conditions
            .iter()
            .cloned()
            .map(ConditionExpression::condition),
    );
    compile_program(&expression, parameters)
}

pub(in crate::state_machine) fn compile_shared_conditions(
    conditions: &[AnimationCompiledTransitionCondition],
) -> Result<CompiledConditionProgram, ConditionExpressionCompileError> {
    let mut instructions = conditions
        .iter()
        .map(|condition| {
            Ok(ConditionInstruction::Compare {
                parameter: ParameterSlot::new(condition.parameter())?,
                operator: condition.operator(),
                value: condition.value().cloned(),
            })
        })
        .collect::<Result<Vec<_>, ConditionExpressionCompileError>>()?;
    instructions.push(ConditionInstruction::All(child_count(conditions.len())?));
    Ok(CompiledConditionProgram {
        instructions: instructions.into_boxed_slice(),
    })
}

fn compile_node(
    expression: &ConditionExpression,
    parameters: &mut ParameterTableBuilder,
    output: &mut Vec<ConditionInstruction>,
    depth: usize,
) -> Result<(), ConditionExpressionCompileError> {
    if depth > MAX_CONDITION_EXPRESSION_DEPTH {
        return Err(ConditionExpressionCompileError::ExpressionTooDeep {
            depth,
            limit: MAX_CONDITION_EXPRESSION_DEPTH,
        });
    }
    match expression {
        ConditionExpression::Condition(condition) => {
            output.push(ConditionInstruction::Compare {
                parameter: parameters.intern(&condition.parameter)?,
                operator: condition.operator,
                value: condition.value.clone(),
            });
        }
        ConditionExpression::All(children) => {
            for child in children {
                compile_node(child, parameters, output, depth + 1)?;
            }
            output.push(ConditionInstruction::All(child_count(children.len())?));
        }
        ConditionExpression::Any(children) => {
            for child in children {
                compile_node(child, parameters, output, depth + 1)?;
            }
            output.push(ConditionInstruction::Any(child_count(children.len())?));
        }
        ConditionExpression::Not(child) => {
            compile_node(child, parameters, output, depth + 1)?;
            output.push(ConditionInstruction::Not);
        }
    }
    Ok(())
}

fn child_count(count: usize) -> Result<u32, ConditionExpressionCompileError> {
    u32::try_from(count).map_err(|_| ConditionExpressionCompileError::CapacityExceeded)
}
