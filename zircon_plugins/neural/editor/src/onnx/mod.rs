mod convert;
mod executable_contract;
mod reader;

use std::collections::BTreeMap;

pub use convert::convert_graph;
pub use reader::{read_onnx_graph, OnnxReadError};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OnnxTensorDataType {
    #[default]
    F32,
    Other,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OnnxTensor {
    pub name: String,
    pub shape: Vec<u32>,
    pub data_type: OnnxTensorDataType,
    pub values: Option<Vec<f32>>,
}

impl OnnxTensor {
    pub fn shape_only(name: impl Into<String>, shape: impl Into<Vec<u32>>) -> Self {
        Self {
            name: name.into(),
            shape: shape.into(),
            data_type: OnnxTensorDataType::F32,
            values: None,
        }
    }

    pub fn f32(name: impl Into<String>, shape: impl Into<Vec<u32>>, values: Vec<f32>) -> Self {
        Self {
            name: name.into(),
            shape: shape.into(),
            data_type: OnnxTensorDataType::F32,
            values: Some(values),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum OnnxAttribute {
    Int(i64),
    Float(f32),
    String(String),
    Floats(Vec<f32>),
    Ints(Vec<i64>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct OnnxNode {
    pub name: String,
    pub op_type: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub attributes: BTreeMap<String, OnnxAttribute>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OnnxGraph {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub tensors: BTreeMap<String, OnnxTensor>,
    pub nodes: Vec<OnnxNode>,
}
