use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::{BlendSpaceCompileError, ConditionExpressionCompileError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnimationStateMachineCompileError {
    DuplicateState { name: String },
    MissingState { name: String },
    CapacityExceeded,
    BlendSpace(BlendSpaceCompileError),
    ConditionExpression(ConditionExpressionCompileError),
}

impl Display for AnimationStateMachineCompileError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateState { name } => write!(formatter, "duplicate state `{name}`"),
            Self::MissingState { name } => write!(formatter, "missing state `{name}`"),
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
