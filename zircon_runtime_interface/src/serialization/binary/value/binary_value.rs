use std::fmt;

use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

use super::super::wire::MAX_BINARY_NODES;

/// Flat, self-describing value stream shared by binary schema versions.
#[derive(Serialize)]
#[serde(transparent)]
pub(in crate::serialization) struct BinaryValue {
    pub(in crate::serialization) nodes: Vec<BinaryNode>,
}

/// Variant order is part of wire v1 and is locked by the golden-byte test.
/// Adding or reordering variants requires a wire-version bump.
#[derive(Deserialize, Serialize)]
pub(in crate::serialization) enum BinaryNode {
    Null,
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    String(String),
    Array { len: u32 },
    Object { len: u32 },
    ObjectKey(String),
}

impl BinaryValue {
    pub(in crate::serialization) fn from_nodes(nodes: Vec<BinaryNode>) -> Self {
        Self { nodes }
    }
}

impl<'de> Deserialize<'de> for BinaryValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BinaryValueVisitor;

        impl<'de> Visitor<'de> for BinaryValueVisitor {
            type Value = BinaryValue;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a bounded flat binary value stream")
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let hinted = sequence.size_hint().unwrap_or(0).min(MAX_BINARY_NODES);
                let mut nodes = Vec::with_capacity(hinted);
                while let Some(node) = sequence.next_element()? {
                    if nodes.len() == MAX_BINARY_NODES {
                        return Err(serde::de::Error::custom(format_args!(
                            "binary value node limit {MAX_BINARY_NODES} exceeded"
                        )));
                    }
                    nodes.push(node);
                }
                Ok(BinaryValue { nodes })
            }
        }

        deserializer.deserialize_seq(BinaryValueVisitor)
    }
}
