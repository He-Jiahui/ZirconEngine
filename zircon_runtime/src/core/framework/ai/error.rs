use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AiManagerError {
    EmptyId {
        field: &'static str,
    },
    InvalidBehaviorTreeFormatVersion {
        expected: u32,
        actual: u32,
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
    InvalidBehaviorNodeChildCount {
        tree_id: String,
        node_id: String,
        expected: &'static str,
        actual: usize,
    },
    InvalidBehaviorTreeTopology {
        tree_id: String,
        node_id: String,
        reason: &'static str,
    },
    DuplicateBehaviorNodeParameter {
        tree_id: String,
        node_id: String,
        key: String,
    },
    NonFiniteBehaviorNodeParameter {
        tree_id: String,
        node_id: String,
        key: String,
    },
    UnknownBehaviorNodeImplementation {
        tree_id: String,
        node_id: String,
        implementation: String,
    },
    BehaviorNodeCatalogDescriptorMissing {
        tree_id: String,
        node_id: String,
        implementation: String,
    },
    BehaviorNodeImplementationCategoryMismatch {
        tree_id: String,
        node_id: String,
        implementation: String,
        expected: &'static str,
        actual: &'static str,
    },
    StandardBehaviorNodeCatalogUnavailable {
        tree_id: String,
    },
    InvalidBehaviorNodeParameter {
        tree_id: String,
        node_id: String,
        key: String,
        expected: &'static str,
        actual: &'static str,
    },
    InvalidBehaviorNodeParameterOwner {
        tree_id: String,
        node_id: String,
        key: String,
        expected: &'static str,
    },
    InvalidBehaviorNodeParameterValue {
        tree_id: String,
        node_id: String,
        key: String,
        expected: &'static str,
        actual: String,
    },
    InvalidBehaviorSubtreeTarget {
        tree_id: String,
        node_id: String,
        target_tree: String,
        reason: &'static str,
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
    BehaviorObserverMissingBlackboardKey {
        tree_id: String,
        node_id: String,
    },
    BehaviorObserverRequiresBlackboardSchema {
        tree_id: String,
    },
    BehaviorObserverUnknownBlackboardKey {
        tree_id: String,
        node_id: String,
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
            Self::InvalidBehaviorTreeFormatVersion { expected, actual } => write!(
                f,
                "behavior-tree format version {actual} is unsupported; expected {expected}"
            ),
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
            Self::InvalidBehaviorNodeChildCount {
                tree_id,
                node_id,
                expected,
                actual,
            } => write!(
                f,
                "AI behavior tree `{tree_id}` node `{node_id}` expected {expected} child nodes but received `{actual}`"
            ),
            Self::InvalidBehaviorTreeTopology {
                tree_id,
                node_id,
                reason,
            } => write!(
                f,
                "AI behavior tree `{tree_id}` node `{node_id}` has invalid topology: {reason}"
            ),
            Self::DuplicateBehaviorNodeParameter {
                tree_id,
                node_id,
                key,
            } => write!(
                f,
                "AI behavior tree `{tree_id}` node `{node_id}` declares duplicate parameter `{key}`"
            ),
            Self::NonFiniteBehaviorNodeParameter {
                tree_id,
                node_id,
                key,
            } => write!(
                f,
                "AI behavior tree `{tree_id}` node `{node_id}` parameter `{key}` contains a non-finite value"
            ),
            Self::UnknownBehaviorNodeImplementation {
                tree_id,
                node_id,
                implementation,
            } => write!(
                f,
                "AI behavior tree `{tree_id}` node `{node_id}` references unknown implementation `{implementation}`"
            ),
            Self::BehaviorNodeCatalogDescriptorMissing {
                tree_id,
                node_id,
                implementation,
            } => write!(
                f,
                "AI behavior tree `{tree_id}` node `{node_id}` resolved `{implementation}` without a catalog descriptor"
            ),
            Self::BehaviorNodeImplementationCategoryMismatch {
                tree_id,
                node_id,
                implementation,
                expected,
                actual,
            } => write!(
                f,
                "AI behavior tree `{tree_id}` node `{node_id}` implementation `{implementation}` has category `{actual}`, expected `{expected}`"
            ),
            Self::StandardBehaviorNodeCatalogUnavailable { tree_id } => write!(
                f,
                "AI behavior tree `{tree_id}` cannot use the unavailable standard node catalog"
            ),
            Self::InvalidBehaviorNodeParameter {
                tree_id,
                node_id,
                key,
                expected,
                actual,
            } => write!(
                f,
                "AI behavior tree `{tree_id}` node `{node_id}` parameter `{key}` expected `{expected}` but received `{actual}`"
            ),
            Self::InvalidBehaviorNodeParameterOwner {
                tree_id,
                node_id,
                key,
                expected,
            } => write!(
                f,
                "AI behavior tree `{tree_id}` node `{node_id}` parameter `{key}` can only be declared by {expected}"
            ),
            Self::InvalidBehaviorNodeParameterValue {
                tree_id,
                node_id,
                key,
                expected,
                actual,
            } => write!(
                f,
                "AI behavior tree `{tree_id}` node `{node_id}` parameter `{key}` expected {expected} but received `{actual}`"
            ),
            Self::InvalidBehaviorSubtreeTarget {
                tree_id,
                node_id,
                target_tree,
                reason,
            } => write!(
                f,
                "AI behavior tree `{tree_id}` subtree node `{node_id}` target `{target_tree}` is invalid: {reason}"
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
            Self::BehaviorObserverMissingBlackboardKey { tree_id, node_id } => write!(
                f,
                "AI behavior-tree `{tree_id}` observer node `{node_id}` does not declare `blackboard_key`"
            ),
            Self::BehaviorObserverRequiresBlackboardSchema { tree_id } => write!(
                f,
                "AI behavior-tree `{tree_id}` declares observer aborts but no blackboard schema was bound"
            ),
            Self::BehaviorObserverUnknownBlackboardKey {
                tree_id,
                node_id,
                schema_id,
                key,
            } => write!(
                f,
                "AI behavior-tree `{tree_id}` observer node `{node_id}` references key `{key}` outside schema `{schema_id}`"
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
