use serde_json::{Map, Number, Value};

use super::super::wire::{
    MAX_BINARY_CONTAINER_ENTRIES, MAX_BINARY_DEPTH, MAX_BINARY_NODES, MAX_BINARY_STRING_BYTES,
};
use super::{BinaryNode, BinaryValue, BinaryValueError};

enum DecodeFrame {
    Array {
        remaining: usize,
        values: Vec<Value>,
    },
    Object {
        remaining: usize,
        values: Map<String, Value>,
        pending_key: Option<String>,
    },
}

impl TryFrom<BinaryValue> for Value {
    type Error = BinaryValueError;

    fn try_from(value: BinaryValue) -> Result<Self, Self::Error> {
        if value.nodes.len() > MAX_BINARY_NODES {
            return Err(BinaryValueError::NodeLimitExceeded {
                max: MAX_BINARY_NODES,
                found: value.nodes.len(),
            });
        }

        let mut frames = Vec::new();
        let mut root = None;
        for node in value.nodes {
            match node {
                BinaryNode::ObjectKey(key) => assign_object_key(&mut frames, key)?,
                BinaryNode::Array { len } => {
                    let len = validate_container(len)?;
                    validate_depth(frames.len() + 1)?;
                    if len == 0 {
                        attach_value(Value::Array(Vec::new()), &mut frames, &mut root)?;
                    } else {
                        frames.push(DecodeFrame::Array {
                            remaining: len,
                            values: Vec::with_capacity(len.min(1024)),
                        });
                    }
                }
                BinaryNode::Object { len } => {
                    let len = validate_container(len)?;
                    validate_depth(frames.len() + 1)?;
                    if len == 0 {
                        attach_value(Value::Object(Map::new()), &mut frames, &mut root)?;
                    } else {
                        frames.push(DecodeFrame::Object {
                            remaining: len,
                            values: Map::new(),
                            pending_key: None,
                        });
                    }
                }
                node => attach_value(primitive_value(node)?, &mut frames, &mut root)?,
            }
        }

        if !frames.is_empty() {
            return Err(BinaryValueError::IncompleteContainer);
        }
        root.ok_or(BinaryValueError::EmptyValue)
    }
}

fn primitive_value(node: BinaryNode) -> Result<Value, BinaryValueError> {
    match node {
        BinaryNode::Null => Ok(Value::Null),
        BinaryNode::Bool(value) => Ok(Value::Bool(value)),
        BinaryNode::I64(value) => Ok(Value::Number(Number::from(value))),
        BinaryNode::U64(value) => Ok(Value::Number(Number::from(value))),
        BinaryNode::F64(value) => Number::from_f64(value)
            .map(Value::Number)
            .ok_or(BinaryValueError::NonFiniteFloat { value }),
        BinaryNode::String(value) => {
            validate_string(&value)?;
            Ok(Value::String(value))
        }
        BinaryNode::Array { .. } => Err(BinaryValueError::UnexpectedNodeKind { kind: "array" }),
        BinaryNode::Object { .. } => Err(BinaryValueError::UnexpectedNodeKind { kind: "object" }),
        BinaryNode::ObjectKey(_) => {
            Err(BinaryValueError::UnexpectedNodeKind { kind: "object-key" })
        }
    }
}

fn assign_object_key(frames: &mut [DecodeFrame], key: String) -> Result<(), BinaryValueError> {
    validate_string(&key)?;
    let Some(DecodeFrame::Object { pending_key, .. }) = frames.last_mut() else {
        return Err(BinaryValueError::UnexpectedObjectKey { key });
    };
    if pending_key.is_some() {
        return Err(BinaryValueError::UnexpectedObjectKey { key });
    }
    *pending_key = Some(key);
    Ok(())
}

fn attach_value(
    mut value: Value,
    frames: &mut Vec<DecodeFrame>,
    root: &mut Option<Value>,
) -> Result<(), BinaryValueError> {
    loop {
        let Some(frame) = frames.last_mut() else {
            if root.replace(value).is_some() {
                return Err(BinaryValueError::MultipleRootValues);
            }
            return Ok(());
        };
        let completed = match frame {
            DecodeFrame::Array { remaining, values } => {
                values.push(value);
                *remaining -= 1;
                (*remaining == 0).then(|| Value::Array(std::mem::take(values)))
            }
            DecodeFrame::Object {
                remaining,
                values,
                pending_key,
            } => {
                let key = pending_key
                    .take()
                    .ok_or(BinaryValueError::MissingObjectKey)?;
                if values.contains_key(&key) {
                    return Err(BinaryValueError::DuplicateObjectKey { key });
                }
                values.insert(key, value);
                *remaining -= 1;
                (*remaining == 0).then(|| Value::Object(std::mem::take(values)))
            }
        };
        let Some(container) = completed else {
            return Ok(());
        };
        frames.pop();
        value = container;
    }
}

fn validate_container(found: u32) -> Result<usize, BinaryValueError> {
    let found = found as usize;
    if found <= MAX_BINARY_CONTAINER_ENTRIES {
        return Ok(found);
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
