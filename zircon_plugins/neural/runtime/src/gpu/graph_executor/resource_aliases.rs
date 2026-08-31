const MAX_TENSOR_SLOTS: usize = u16::MAX as usize + 1;

pub(super) struct ResourceAliases {
    canonical_sources: Vec<Option<u16>>,
}

impl ResourceAliases {
    pub(super) fn new(tensor_count: usize) -> Self {
        Self {
            canonical_sources: vec![None; tensor_count.min(MAX_TENSOR_SLOTS)],
        }
    }

    pub(super) fn alias(&mut self, output: u16, source: u16) -> bool {
        let canonical_source = self.resolve(source);
        let Some(slot) = self.canonical_sources.get_mut(usize::from(output)) else {
            return false;
        };
        *slot = Some(canonical_source);
        true
    }

    pub(super) fn resolve(&self, tensor: u16) -> u16 {
        self.canonical_sources
            .get(usize::from(tensor))
            .copied()
            .flatten()
            .unwrap_or(tensor)
    }
}

#[cfg(test)]
mod performance_tests;
