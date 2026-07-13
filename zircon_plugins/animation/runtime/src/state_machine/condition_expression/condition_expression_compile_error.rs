use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConditionExpressionCompileError {
    CapacityExceeded,
    ExpressionTooDeep { depth: usize, limit: usize },
}

impl Display for ConditionExpressionCompileError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CapacityExceeded => formatter.write_str("condition expression capacity exceeded"),
            Self::ExpressionTooDeep { depth, limit } => write!(
                formatter,
                "condition expression depth {depth} exceeds limit {limit}"
            ),
        }
    }
}

impl Error for ConditionExpressionCompileError {}
