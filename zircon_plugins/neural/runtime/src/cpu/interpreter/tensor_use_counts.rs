use crate::{NnModelAsset, NnTensorKind};

const MAX_TENSOR_SLOTS: usize = u16::MAX as usize + 1;

pub(super) struct TensorUseCounts {
    remaining: Vec<usize>,
}

impl TensorUseCounts {
    pub(super) fn new(model: &NnModelAsset) -> Self {
        let mut remaining = vec![0_usize; model.tensors.len().min(MAX_TENSOR_SLOTS)];
        for op in &model.ops {
            for &tensor_id in &op.inputs {
                if let Some(count) = remaining.get_mut(usize::from(tensor_id)) {
                    *count = count.saturating_add(1);
                }
            }
        }
        for (tensor_id, descriptor) in model.tensors.iter().enumerate() {
            if descriptor.kind == NnTensorKind::Output {
                remaining[tensor_id] = remaining[tensor_id].saturating_add(1);
            }
        }
        Self { remaining }
    }

    pub(super) fn is_last_consumer(&self, tensor_id: u16) -> bool {
        self.remaining.get(usize::from(tensor_id)).copied() == Some(1)
    }

    pub(super) fn consume(&mut self, tensor_ids: &[u16]) {
        for &tensor_id in tensor_ids {
            if let Some(count) = self.remaining.get_mut(usize::from(tensor_id)) {
                *count = count.saturating_sub(1);
            }
        }
    }

    #[cfg(test)]
    pub(super) fn remaining(&self, tensor_id: u16) -> usize {
        self.remaining
            .get(usize::from(tensor_id))
            .copied()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod performance_tests;
