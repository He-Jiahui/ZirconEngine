use std::collections::{BTreeMap, BTreeSet};

use zircon_plugin_neural_runtime::NnTensorKind;

use crate::onnx::{OnnxGraph, OnnxTensor};

pub(super) struct TensorProjection<'a> {
    tensor_ids: BTreeMap<&'a str, u16>,
    inputs: BTreeSet<&'a str>,
    outputs: BTreeSet<&'a str>,
}

impl<'a> TensorProjection<'a> {
    pub(super) fn new(graph: &'a OnnxGraph) -> Result<Self, ()> {
        let tensor_ids = graph
            .tensors
            .keys()
            .enumerate()
            .map(|(index, name)| u16::try_from(index).map(|id| (name.as_str(), id)))
            .collect::<Result<BTreeMap<_, _>, _>>()
            .map_err(|_| ())?;
        let inputs = graph.inputs.iter().map(String::as_str).collect();
        let outputs = graph.outputs.iter().map(String::as_str).collect();
        Ok(Self {
            tensor_ids,
            inputs,
            outputs,
        })
    }

    pub(super) fn id(&self, name: &str) -> Option<u16> {
        self.tensor_ids.get(name).copied()
    }

    pub(super) fn kind(&self, name: &str, tensor: &OnnxTensor) -> NnTensorKind {
        if tensor.values.is_some() {
            NnTensorKind::Weight
        } else if self.inputs.contains(name) {
            NnTensorKind::Input
        } else if self.outputs.contains(name) {
            NnTensorKind::Output
        } else {
            NnTensorKind::Intermediate
        }
    }
}

#[cfg(test)]
mod performance_tests;
