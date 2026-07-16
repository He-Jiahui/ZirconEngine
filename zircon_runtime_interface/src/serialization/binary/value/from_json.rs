use serde_json::{Number, Value};

use super::super::wire::{
    MAX_BINARY_CONTAINER_ENTRIES, MAX_BINARY_DEPTH, MAX_BINARY_NODES, MAX_BINARY_STRING_BYTES,
};
use super::{BinaryNode, BinaryValue, BinaryValueError};

enum EncodeTask {
    Value { value: Value, depth: usize },
    ObjectKey(String),
}

impl TryFrom<Value> for BinaryValue {
    type Error = BinaryValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let mut nodes = Vec::new();
        let mut tasks = vec![EncodeTask::Value { value, depth: 0 }];

        while let Some(task) = tasks.pop() {
            match task {
                EncodeTask::ObjectKey(key) => {
                    validate_string(&key)?;
                    push_node(&mut nodes, BinaryNode::ObjectKey(key))?;
                }
                EncodeTask::Value { value, depth } => {
                    encode_value(value, depth, &mut nodes, &mut tasks)?;
                }
            }
        }

        Ok(Self::from_nodes(nodes))
    }
}

fn encode_value(
    value: Value,
    depth: usize,
    nodes: &mut Vec<BinaryNode>,
    tasks: &mut Vec<EncodeTask>,
) -> Result<(), BinaryValueError> {
    let node = match value {
        Value::Null => BinaryNode::Null,
        Value::Bool(value) => BinaryNode::Bool(value),
        Value::Number(value) => number_from_json(value)?,
        Value::String(value) => {
            validate_string(&value)?;
            BinaryNode::String(value)
        }
        Value::Array(values) => {
            validate_container(values.len())?;
            validate_depth(depth + 1)?;
            let len = values.len() as u32;
            tasks.extend(values.into_iter().rev().map(|value| EncodeTask::Value {
                value,
                depth: depth + 1,
            }));
            BinaryNode::Array { len }
        }
        Value::Object(values) => {
            validate_container(values.len())?;
            validate_depth(depth + 1)?;
            let len = values.len() as u32;
            for (key, value) in values.into_iter().rev() {
                tasks.push(EncodeTask::Value {
                    value,
                    depth: depth + 1,
                });
                tasks.push(EncodeTask::ObjectKey(key));
            }
            BinaryNode::Object { len }
        }
    };
    push_node(nodes, node)
}

fn number_from_json(value: Number) -> Result<BinaryNode, BinaryValueError> {
    if let Some(value) = value.as_i64() {
        return Ok(BinaryNode::I64(value));
    }
    if let Some(value) = value.as_u64() {
        return Ok(BinaryNode::U64(value));
    }
    value
        .as_f64()
        .map(BinaryNode::F64)
        .ok_or_else(|| BinaryValueError::InvalidJsonNumber {
            value: value.to_string(),
        })
}

fn push_node(nodes: &mut Vec<BinaryNode>, node: BinaryNode) -> Result<(), BinaryValueError> {
    if nodes.len() == MAX_BINARY_NODES {
        return Err(BinaryValueError::NodeLimitExceeded {
            max: MAX_BINARY_NODES,
            found: MAX_BINARY_NODES + 1,
        });
    }
    nodes.push(node);
    Ok(())
}

fn validate_container(found: usize) -> Result<(), BinaryValueError> {
    if found <= MAX_BINARY_CONTAINER_ENTRIES {
        return Ok(());
    }
    Err(BinaryValueError::ContainerLimitExceeded {
        max: MAX_BINARY_CONTAINER_ENTRIES,
        found,
    })
}

fn validate_depth(found: usize) -> Result<(), BinaryValueError> {
    if found <= MAX_BINARY_DEPTH {
        return Ok(());
    }
    Err(BinaryValueError::DepthLimitExceeded {
        max: MAX_BINARY_DEPTH,
        found,
    })
}

fn validate_string(value: &str) -> Result<(), BinaryValueError> {
    if value.len() <= MAX_BINARY_STRING_BYTES {
        return Ok(());
    }
    Err(BinaryValueError::StringLimitExceeded {
        max: MAX_BINARY_STRING_BYTES,
        found: value.len(),
    })
}
