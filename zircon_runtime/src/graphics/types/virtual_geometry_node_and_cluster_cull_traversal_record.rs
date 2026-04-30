#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VirtualGeometryNodeAndClusterCullTraversalOp {
    VisitNode,
    StoreCluster,
    EnqueueChild,
}

impl VirtualGeometryNodeAndClusterCullTraversalOp {
    fn packed_word(self) -> u32 {
        match self {
            Self::VisitNode => 1,
            Self::StoreCluster => 2,
            Self::EnqueueChild => 3,
        }
    }

    #[cfg(test)]
    fn from_packed_word(word: u32) -> Option<Self> {
        match word {
            1 => Some(Self::VisitNode),
            2 => Some(Self::StoreCluster),
            3 => Some(Self::EnqueueChild),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VirtualGeometryNodeAndClusterCullTraversalChildSource {
    None,
    FixedFanout,
    AuthoredHierarchy,
}

impl VirtualGeometryNodeAndClusterCullTraversalChildSource {
    fn packed_word(self) -> u32 {
        match self {
            Self::None => 0,
            Self::FixedFanout => 1,
            Self::AuthoredHierarchy => 2,
        }
    }

    #[cfg(test)]
    fn from_packed_word(word: u32) -> Option<Self> {
        match word {
            0 => Some(Self::None),
            1 => Some(Self::FixedFanout),
            2 => Some(Self::AuthoredHierarchy),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct VirtualGeometryNodeAndClusterCullTraversalRecord {
    pub(crate) op: VirtualGeometryNodeAndClusterCullTraversalOp,
    pub(crate) child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource,
    pub(crate) instance_index: u32,
    pub(crate) entity: u64,
    pub(crate) cluster_array_index: u32,
    pub(crate) hierarchy_node_id: Option<u32>,
    pub(crate) node_cluster_start: u32,
    pub(crate) node_cluster_count: u32,
    pub(crate) child_base: u32,
    pub(crate) child_count: u32,
    pub(crate) traversal_index: u32,
    pub(crate) cluster_budget: u32,
    pub(crate) page_budget: u32,
    pub(crate) forced_mip: Option<u8>,
}

impl VirtualGeometryNodeAndClusterCullTraversalRecord {
    pub(crate) const GPU_WORD_COUNT: usize = 16;
    const NONE_SENTINEL: u32 = u32::MAX;

    pub(crate) fn packed_words(&self) -> [u32; Self::GPU_WORD_COUNT] {
        [
            self.op.packed_word(),
            self.child_source.packed_word(),
            self.instance_index,
            (self.entity & u64::from(u32::MAX)) as u32,
            (self.entity >> 32) as u32,
            self.cluster_array_index,
            self.hierarchy_node_id.unwrap_or(Self::NONE_SENTINEL),
            self.node_cluster_start,
            self.node_cluster_count,
            self.child_base,
            self.child_count,
            self.traversal_index,
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
            op: VirtualGeometryNodeAndClusterCullTraversalOp::from_packed_word(words[0])?,
            child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::from_packed_word(
                words[1],
            )?,
            instance_index: words[2],
            entity: u64::from(words[3]) | (u64::from(words[4]) << 32),
            cluster_array_index: words[5],
            hierarchy_node_id: (words[6] != Self::NONE_SENTINEL).then_some(words[6]),
            node_cluster_start: words[7],
            node_cluster_count: words[8],
            child_base: words[9],
            child_count: words[10],
            traversal_index: words[11],
            cluster_budget: words[12],
            page_budget: words[13],
            forced_mip: (words[14] != Self::NONE_SENTINEL).then_some(words[14] as u8),
        })
    }
}
