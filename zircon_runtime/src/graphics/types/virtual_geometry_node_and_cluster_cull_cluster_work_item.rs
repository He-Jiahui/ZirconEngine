#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct VirtualGeometryNodeAndClusterCullClusterWorkItem {
    pub(crate) instance_index: u32,
    pub(crate) entity: u64,
    pub(crate) cluster_array_index: u32,
    pub(crate) hierarchy_node_id: Option<u32>,
    pub(crate) cluster_budget: u32,
    pub(crate) page_budget: u32,
    pub(crate) forced_mip: Option<u8>,
}

impl VirtualGeometryNodeAndClusterCullClusterWorkItem {
    pub(crate) const GPU_WORD_COUNT: usize = 9;
    const NONE_SENTINEL: u32 = u32::MAX;

    pub(crate) fn packed_words(&self) -> [u32; Self::GPU_WORD_COUNT] {
        [
            self.instance_index,
            (self.entity & u64::from(u32::MAX)) as u32,
            (self.entity >> 32) as u32,
            self.cluster_array_index,
            self.hierarchy_node_id.unwrap_or(Self::NONE_SENTINEL),
            self.cluster_budget,
            self.page_budget,
            self.forced_mip
                .map(u32::from)
                .unwrap_or(Self::NONE_SENTINEL),
            0,
        ]
    }

    #[cfg(test)]
    pub(crate) fn from_packed_words(words: &[u32]) -> Option<Self> {
        if words.len() < Self::GPU_WORD_COUNT {
            return None;
        }

        Some(Self {
            instance_index: words[0],
            entity: u64::from(words[1]) | (u64::from(words[2]) << 32),
            cluster_array_index: words[3],
            hierarchy_node_id: (words[4] != Self::NONE_SENTINEL).then_some(words[4]),
            cluster_budget: words[5],
            page_budget: words[6],
            forced_mip: (words[7] != Self::NONE_SENTINEL).then_some(words[7] as u8),
        })
    }
}
