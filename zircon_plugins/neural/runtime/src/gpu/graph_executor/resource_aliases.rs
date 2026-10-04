// 只为可由 u16 tensor id 寻址的描述符分配 alias 槽。
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
        // 先解析 source，使连续 reshape 的别名直接指向最初的存储资源。
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
#[path = "resource_aliases/tests/performance_tests.rs"]
mod performance_tests;
