const MAX_TENSOR_SLOTS: usize = u16::MAX as usize + 1;

pub(super) struct TensorWorkspace {
    slots: Vec<Option<Vec<f32>>>,
}

impl TensorWorkspace {
    pub(super) fn new(tensor_count: usize) -> Self {
        let slot_count = tensor_count.min(MAX_TENSOR_SLOTS);
        Self {
            slots: vec![None; slot_count],
        }
    }

    pub(super) fn get(&self, tensor_id: u16) -> Option<&[f32]> {
        self.slots
            .get(usize::from(tensor_id))
            .and_then(Option::as_deref)
    }

    pub(super) fn store(&mut self, tensor_id: u16, values: Vec<f32>) -> bool {
        let Some(slot) = self.slots.get_mut(usize::from(tensor_id)) else {
            return false;
        };
        *slot = Some(values);
        true
    }

    pub(super) fn take(&mut self, tensor_id: u16) -> Option<Vec<f32>> {
        self.slots
            .get_mut(usize::from(tensor_id))
            .and_then(Option::take)
    }
}

pub(super) struct InputBindings<'a> {
    slots: Vec<Option<&'a [f32]>>,
}

impl<'a> InputBindings<'a> {
    pub(super) fn new(tensor_count: usize, inputs: &[(u16, &'a [f32])]) -> Self {
        let slot_count = tensor_count.min(MAX_TENSOR_SLOTS);
        let mut slots = vec![None; slot_count];
        for &(tensor_id, values) in inputs {
            if let Some(slot) = slots.get_mut(usize::from(tensor_id)) {
                if slot.is_none() {
                    *slot = Some(values);
                }
            }
        }
        Self { slots }
    }

    pub(super) fn get(&self, tensor_id: u16) -> Option<&'a [f32]> {
        self.slots.get(usize::from(tensor_id)).copied().flatten()
    }
}

#[cfg(test)]
mod performance_tests;
