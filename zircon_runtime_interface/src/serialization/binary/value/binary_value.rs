use std::fmt;

use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

use super::super::wire::MAX_BINARY_NODES;

/// Flat, self-describing value stream shared by binary schema versions.
/// 节点按深度优先前序排列；容器长度表示直接子项数量，后续节点承载其内容。
/// `serde(transparent)` 使线格式直接使用节点序列，不增加外层包装字段。
#[derive(Serialize)]
#[serde(transparent)]
pub(in crate::serialization) struct BinaryValue {
    /// 拥有型线性 token 缓冲区，长度预算也计入其中的结构标记。
    pub(in crate::serialization) nodes: Vec<BinaryNode>,
}

/// Variant order is part of wire v1 and is locked by the golden-byte test.
/// Adding or reordering variants requires a wire-version bump.
/// 单个节点表示 JSON 标量、容器起点或对象键；对象键后必须跟随对应值的节点子树。
#[derive(Deserialize, Serialize)]
pub(in crate::serialization) enum BinaryNode {
    /// JSON null 标量。
    Null,
    /// JSON 布尔标量。
    Bool(bool),
    /// 有符号 64 位 JSON 整数。
    I64(i64),
    /// 无符号 64 位 JSON 整数。
    U64(u64),
    /// 双精度浮点值；读取路径会拒绝非有限数。
    F64(f64),
    /// UTF-8 字符串标量，受字符串字节预算约束。
    String(String),
    /// 数组起点，`len` 是紧随其后的直接元素数。
    Array { len: u32 },
    /// 对象起点，`len` 是紧随其后的键值对数。
    Object { len: u32 },
    /// 对象字段键；只允许出现在对象的键位置。
    ObjectKey(String),
}

impl BinaryValue {
    /// 用节点向量建立流包装，不校验结构或预算；解释流之前由相应读取路径执行值域校验。
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
                // size_hint 只用于预分配且先封顶；每次追加前限制扁平流长度，结构校验留给后续读取路径。
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
