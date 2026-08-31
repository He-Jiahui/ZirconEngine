use std::error::Error;
use std::fmt::{Display, Formatter};

use zircon_runtime::core::framework::animation::compiler::AnimationCompileDiagnostic;

use crate::{BlendSpaceCompileError, ConditionExpressionCompileError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnimationStateMachineCompileError {
    SourceDiagnostics(Vec<AnimationCompileDiagnostic>),
    CapacityExceeded,
    BlendSpace(BlendSpaceCompileError),
    ConditionExpression(ConditionExpressionCompileError),
}

impl Display for AnimationStateMachineCompileError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceDiagnostics(diagnostics) => match diagnostics.first() {
                Some(diagnostic) => write!(
                    formatter,
                    "animation state-machine source rejected by {}: {}",
                    diagnostic.code(),
                    diagnostic.message()
                ),
                None => formatter
                    .write_str("animation state-machine source rejected without diagnostics"),
            },
            Self::CapacityExceeded => formatter.write_str("state-machine capacity exceeded"),
            Self::BlendSpace(error) => Display::fmt(error, formatter),
            Self::ConditionExpression(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for AnimationStateMachineCompileError {}

impl From<ConditionExpressionCompileError> for AnimationStateMachineCompileError {
    fn from(value: ConditionExpressionCompileError) -> Self {
        Self::ConditionExpression(value)
    }
}

impl From<BlendSpaceCompileError> for AnimationStateMachineCompileError {
    fn from(value: BlendSpaceCompileError) -> Self {
        Self::BlendSpace(value)
    }
}
