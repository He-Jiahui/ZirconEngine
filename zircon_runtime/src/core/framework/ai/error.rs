use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AiManagerError {
    EmptyId {
        field: &'static str,
    },
    DuplicateId {
        id: String,
    },
    MissingRootNode {
        tree_id: String,
        root_node: String,
    },
    MissingChildNode {
        tree_id: String,
        node_id: String,
        child_id: String,
    },
    UnknownBehaviorTree {
        id: u64,
    },
    UnknownBlackboardSchema {
        id: u64,
    },
    DuplicateBlackboardKey {
        schema_id: String,
        key: String,
    },
    UnknownBlackboardValueType {
        schema_id: String,
        key: String,
        value_type: String,
    },
    DuplicateBlackboardEntry {
        key: String,
    },
    UnknownBlackboardKey {
        schema_id: String,
        key: String,
    },
    MissingBlackboardKey {
        schema_id: String,
        key: String,
    },
    BlackboardValueTypeMismatch {
        schema_id: String,
        key: String,
        expected: String,
        actual: String,
    },
    NonFiniteBlackboardValue {
        key: String,
    },
    NonFiniteTickDelta,
    PerceptionAgentMismatch {
        expected: u64,
        actual: u64,
    },
    NonFinitePerceptionStimulus {
        source: u64,
    },
}

impl fmt::Display for AiManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId { field } => write!(f, "AI descriptor field `{field}` must not be empty"),
            Self::DuplicateId { id } => write!(f, "AI descriptor id `{id}` is already registered"),
            Self::MissingRootNode { tree_id, root_node } => write!(
                f,
                "AI behavior tree `{tree_id}` references missing root node `{root_node}`"
            ),
            Self::MissingChildNode {
                tree_id,
                node_id,
                child_id,
            } => write!(
                f,
                "AI behavior tree `{tree_id}` node `{node_id}` references missing child `{child_id}`"
            ),
            Self::UnknownBehaviorTree { id } => {
                write!(f, "AI behavior tree handle `{id}` is not registered")
            }
            Self::UnknownBlackboardSchema { id } => {
                write!(f, "AI blackboard schema handle `{id}` is not registered")
            }
            Self::DuplicateBlackboardKey { schema_id, key } => write!(
                f,
                "AI blackboard schema `{schema_id}` declares duplicate key `{key}`"
            ),
            Self::UnknownBlackboardValueType {
                schema_id,
                key,
                value_type,
            } => write!(
                f,
                "AI blackboard schema `{schema_id}` key `{key}` uses unknown value type `{value_type}`"
            ),
            Self::DuplicateBlackboardEntry { key } => {
                write!(f, "AI blackboard entry `{key}` is duplicated")
            }
            Self::UnknownBlackboardKey { schema_id, key } => write!(
                f,
                "AI blackboard entry `{key}` is not declared by schema `{schema_id}`"
            ),
            Self::MissingBlackboardKey { schema_id, key } => write!(
                f,
                "AI blackboard schema `{schema_id}` requires missing key `{key}`"
            ),
            Self::BlackboardValueTypeMismatch {
                schema_id,
                key,
                expected,
                actual,
            } => write!(
                f,
                "AI blackboard schema `{schema_id}` key `{key}` expected `{expected}` but received `{actual}`"
            ),
            Self::NonFiniteBlackboardValue { key } => {
                write!(f, "AI blackboard entry `{key}` contains a non-finite value")
            }
            Self::NonFiniteTickDelta => write!(f, "AI tick delta must be finite"),
            Self::PerceptionAgentMismatch { expected, actual } => write!(
                f,
                "AI perception snapshot agent `{actual}` does not match tick entity `{expected}`"
            ),
            Self::NonFinitePerceptionStimulus { source } => write!(
                f,
                "AI perception stimulus from entity `{source}` contains a non-finite value"
            ),
        }
    }
}

impl Error for AiManagerError {}
