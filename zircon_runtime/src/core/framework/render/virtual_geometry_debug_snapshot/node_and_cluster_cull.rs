use super::cull_input::RenderVirtualGeometryCullInputSnapshot;

#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot {
    pub cull_input: RenderVirtualGeometryCullInputSnapshot,
    pub viewport_size: [u32; 2],
    pub camera_translation: [f32; 3],
    pub child_split_screen_space_error_threshold: f32,
    pub child_frustum_culling_enabled: bool,
    pub view_proj: [[f32; 4]; 4],
    pub previous_camera_translation: [f32; 3],
    pub previous_view_proj: [[f32; 4]; 4],
}

impl RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot {
    pub const GPU_WORD_COUNT: usize = RenderVirtualGeometryCullInputSnapshot::GPU_WORD_COUNT + 42;

    pub fn packed_words(&self) -> [u32; Self::GPU_WORD_COUNT] {
        let mut words = [0_u32; Self::GPU_WORD_COUNT];
        let cull_words = self.cull_input.packed_words();
        words[..RenderVirtualGeometryCullInputSnapshot::GPU_WORD_COUNT]
            .copy_from_slice(&cull_words);
        let mut cursor = RenderVirtualGeometryCullInputSnapshot::GPU_WORD_COUNT;
        words[cursor] = self.viewport_size[0];
        words[cursor + 1] = self.viewport_size[1];
        cursor += 2;
        for value in self.camera_translation {
            words[cursor] = value.to_bits();
            cursor += 1;
        }
        words[cursor] = self.child_split_screen_space_error_threshold.to_bits();
        words[cursor + 1] = u32::from(self.child_frustum_culling_enabled);
        cursor += 2;
        for column in self.view_proj {
            for value in column {
                words[cursor] = value.to_bits();
                cursor += 1;
            }
        }
        for value in self.previous_camera_translation {
            words[cursor] = value.to_bits();
            cursor += 1;
        }
        for column in self.previous_view_proj {
            for value in column {
                words[cursor] = value.to_bits();
                cursor += 1;
            }
        }
        words
    }

    pub fn from_packed_words(words: &[u32]) -> Option<Self> {
        if words.len() < Self::GPU_WORD_COUNT {
            return None;
        }

        let cull_input = RenderVirtualGeometryCullInputSnapshot::from_packed_words(
            &words[..RenderVirtualGeometryCullInputSnapshot::GPU_WORD_COUNT],
        )?;
        let mut cursor = RenderVirtualGeometryCullInputSnapshot::GPU_WORD_COUNT;
        let viewport_size = [words[cursor], words[cursor + 1]];
        cursor += 2;
        let camera_translation = [
            f32::from_bits(words[cursor]),
            f32::from_bits(words[cursor + 1]),
            f32::from_bits(words[cursor + 2]),
        ];
        cursor += 3;
        let child_split_screen_space_error_threshold = f32::from_bits(words[cursor]);
        let child_frustum_culling_enabled = words[cursor + 1] != 0;
        cursor += 2;
        let mut view_proj = [[0.0_f32; 4]; 4];
        for column in &mut view_proj {
            for value in column.iter_mut() {
                *value = f32::from_bits(words[cursor]);
                cursor += 1;
            }
        }
        let previous_camera_translation = [
            f32::from_bits(words[cursor]),
            f32::from_bits(words[cursor + 1]),
            f32::from_bits(words[cursor + 2]),
        ];
        cursor += 3;
        let mut previous_view_proj = [[0.0_f32; 4]; 4];
        for column in &mut previous_view_proj {
            for value in column.iter_mut() {
                *value = f32::from_bits(words[cursor]);
                cursor += 1;
            }
        }

        Some(Self {
            cull_input,
            viewport_size,
            camera_translation,
            child_split_screen_space_error_threshold,
            child_frustum_culling_enabled,
            view_proj,
            previous_camera_translation,
            previous_view_proj,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
    pub instance_index: u32,
    pub entity: u64,
    pub cluster_offset: u32,
    pub cluster_count: u32,
    pub page_offset: u32,
    pub page_count: u32,
}

impl RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
    pub const GPU_WORD_COUNT: usize = 7;

    pub fn packed_words(&self) -> [u32; Self::GPU_WORD_COUNT] {
        let entity_low = (self.entity & u64::from(u32::MAX)) as u32;
        let entity_high = (self.entity >> 32) as u32;
        [
            self.instance_index,
            entity_low,
            entity_high,
            self.cluster_offset,
            self.cluster_count,
            self.page_offset,
            self.page_count,
        ]
    }

    pub fn from_packed_words(words: &[u32]) -> Option<Self> {
        if words.len() < Self::GPU_WORD_COUNT {
            return None;
        }

        Some(Self {
            instance_index: words[0],
            entity: u64::from(words[1]) | (u64::from(words[2]) << 32),
            cluster_offset: words[3],
            cluster_count: words[4],
            page_offset: words[5],
            page_count: words[6],
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem {
    pub instance_index: u32,
    pub entity: u64,
    pub cluster_offset: u32,
    pub cluster_count: u32,
    pub page_offset: u32,
    pub page_count: u32,
    pub cluster_budget: u32,
    pub page_budget: u32,
    pub forced_mip: Option<u8>,
}

impl RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem {
    pub const GPU_WORD_COUNT: usize = 10;
    const NONE_SENTINEL: u32 = u32::MAX;

    pub fn packed_words(&self) -> [u32; Self::GPU_WORD_COUNT] {
        let entity_low = (self.entity & u64::from(u32::MAX)) as u32;
        let entity_high = (self.entity >> 32) as u32;
        [
            self.instance_index,
            entity_low,
            entity_high,
            self.cluster_offset,
            self.cluster_count,
            self.page_offset,
            self.page_count,
            self.cluster_budget,
            self.page_budget,
            self.forced_mip
                .map(u32::from)
                .unwrap_or(Self::NONE_SENTINEL),
        ]
    }

    pub fn from_packed_words(words: &[u32]) -> Option<Self> {
        if words.len() < Self::GPU_WORD_COUNT {
            return None;
        }

        Some(Self {
            instance_index: words[0],
            entity: u64::from(words[1]) | (u64::from(words[2]) << 32),
            cluster_offset: words[3],
            cluster_count: words[4],
            page_offset: words[5],
            page_count: words[6],
            cluster_budget: words[7],
            page_budget: words[8],
            forced_mip: (words[9] != Self::NONE_SENTINEL).then_some(words[9] as u8),
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryNodeAndClusterCullClusterWorkItem {
    pub instance_index: u32,
    pub entity: u64,
    pub cluster_array_index: u32,
    pub hierarchy_node_id: Option<u32>,
    pub cluster_budget: u32,
    pub page_budget: u32,
    pub forced_mip: Option<u8>,
}

impl RenderVirtualGeometryNodeAndClusterCullClusterWorkItem {
    pub const GPU_WORD_COUNT: usize = 9;
    const NONE_SENTINEL: u32 = u32::MAX;

    pub fn packed_words(&self) -> [u32; Self::GPU_WORD_COUNT] {
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

    pub fn from_packed_words(words: &[u32]) -> Option<Self> {
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryNodeAndClusterCullChildWorkItem {
    pub instance_index: u32,
    pub entity: u64,
    pub parent_cluster_array_index: u32,
    pub parent_hierarchy_node_id: Option<u32>,
    pub child_node_id: u32,
    pub child_table_index: u32,
    pub traversal_index: u32,
    pub cluster_budget: u32,
    pub page_budget: u32,
    pub forced_mip: Option<u8>,
}

impl RenderVirtualGeometryNodeAndClusterCullChildWorkItem {
    pub const GPU_WORD_COUNT: usize = 12;
    const NONE_SENTINEL: u32 = u32::MAX;

    pub fn packed_words(&self) -> [u32; Self::GPU_WORD_COUNT] {
        [
            self.instance_index,
            (self.entity & u64::from(u32::MAX)) as u32,
            (self.entity >> 32) as u32,
            self.parent_cluster_array_index,
            self.parent_hierarchy_node_id.unwrap_or(Self::NONE_SENTINEL),
            self.child_node_id,
            self.child_table_index,
            self.traversal_index,
            self.cluster_budget,
            self.page_budget,
            self.forced_mip
                .map(u32::from)
                .unwrap_or(Self::NONE_SENTINEL),
            0,
        ]
    }

    pub fn from_packed_words(words: &[u32]) -> Option<Self> {
        if words.len() < Self::GPU_WORD_COUNT {
            return None;
        }

        Some(Self {
            instance_index: words[0],
            entity: u64::from(words[1]) | (u64::from(words[2]) << 32),
            parent_cluster_array_index: words[3],
            parent_hierarchy_node_id: (words[4] != Self::NONE_SENTINEL).then_some(words[4]),
            child_node_id: words[5],
            child_table_index: words[6],
            traversal_index: words[7],
            cluster_budget: words[8],
            page_budget: words[9],
            forced_mip: (words[10] != Self::NONE_SENTINEL).then_some(words[10] as u8),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderVirtualGeometryNodeAndClusterCullTraversalOp {
    VisitNode,
    StoreCluster,
    EnqueueChild,
}

impl RenderVirtualGeometryNodeAndClusterCullTraversalOp {
    const fn packed_word(self) -> u32 {
        match self {
            Self::VisitNode => 1,
            Self::StoreCluster => 2,
            Self::EnqueueChild => 3,
        }
    }

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
pub enum RenderVirtualGeometryNodeAndClusterCullTraversalChildSource {
    None,
    FixedFanout,
    AuthoredHierarchy,
}

impl RenderVirtualGeometryNodeAndClusterCullTraversalChildSource {
    const fn packed_word(self) -> u32 {
        match self {
            Self::None => 0,
            Self::FixedFanout => 1,
            Self::AuthoredHierarchy => 2,
        }
    }

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
pub struct RenderVirtualGeometryNodeAndClusterCullTraversalRecord {
    pub op: RenderVirtualGeometryNodeAndClusterCullTraversalOp,
    pub child_source: RenderVirtualGeometryNodeAndClusterCullTraversalChildSource,
    pub instance_index: u32,
    pub entity: u64,
    pub cluster_array_index: u32,
    pub hierarchy_node_id: Option<u32>,
    pub node_cluster_start: u32,
    pub node_cluster_count: u32,
    pub child_base: u32,
    pub child_count: u32,
    pub traversal_index: u32,
    pub cluster_budget: u32,
    pub page_budget: u32,
    pub forced_mip: Option<u8>,
}

impl RenderVirtualGeometryNodeAndClusterCullTraversalRecord {
    pub const GPU_WORD_COUNT: usize = 16;
    const NONE_SENTINEL: u32 = u32::MAX;

    pub fn packed_words(&self) -> [u32; Self::GPU_WORD_COUNT] {
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

    pub fn from_packed_words(words: &[u32]) -> Option<Self> {
        if words.len() < Self::GPU_WORD_COUNT {
            return None;
        }

        Some(Self {
            op: RenderVirtualGeometryNodeAndClusterCullTraversalOp::from_packed_word(words[0])?,
            child_source:
                RenderVirtualGeometryNodeAndClusterCullTraversalChildSource::from_packed_word(
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot {
    pub instance_seed_count: u32,
    pub cluster_budget: u32,
    pub page_budget: u32,
    pub workgroup_size: u32,
    pub dispatch_group_count: [u32; 3],
}

impl RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot {
    pub const GPU_WORD_COUNT: usize = 7;

    pub fn packed_words(&self) -> [u32; Self::GPU_WORD_COUNT] {
        [
            self.instance_seed_count,
            self.cluster_budget,
            self.page_budget,
            self.workgroup_size,
            self.dispatch_group_count[0],
            self.dispatch_group_count[1],
            self.dispatch_group_count[2],
        ]
    }

    pub fn from_packed_words(words: &[u32]) -> Option<Self> {
        if words.len() < Self::GPU_WORD_COUNT {
            return None;
        }

        Some(Self {
            instance_seed_count: words[0],
            cluster_budget: words[1],
            page_budget: words[2],
            workgroup_size: words[3],
            dispatch_group_count: [words[4], words[5], words[6]],
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot {
    pub global_state: RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    pub dispatch_setup: RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    pub instance_seeds: Vec<RenderVirtualGeometryNodeAndClusterCullInstanceSeed>,
}

impl RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot {
    pub const GPU_HEADER_WORD_COUNT: usize =
        RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot::GPU_WORD_COUNT
            + RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot::GPU_WORD_COUNT;

    pub fn packed_words(&self) -> Vec<u32> {
        let mut words = Vec::with_capacity(
            Self::GPU_HEADER_WORD_COUNT
                + self.instance_seeds.len()
                    * RenderVirtualGeometryNodeAndClusterCullInstanceSeed::GPU_WORD_COUNT,
        );
        words.extend_from_slice(&self.global_state.packed_words());
        words.extend_from_slice(&self.dispatch_setup.packed_words());
        words.extend(
            self.instance_seeds
                .iter()
                .flat_map(RenderVirtualGeometryNodeAndClusterCullInstanceSeed::packed_words),
        );
        words
    }

    pub fn from_packed_words(words: &[u32]) -> Option<Self> {
        if words.len() < Self::GPU_HEADER_WORD_COUNT {
            return None;
        }

        let mut cursor = 0;
        let global_state_end =
            cursor + RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot::GPU_WORD_COUNT;
        let global_state =
            RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot::from_packed_words(
                &words[cursor..global_state_end],
            )?;
        cursor = global_state_end;

        let dispatch_setup_end =
            cursor + RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot::GPU_WORD_COUNT;
        let dispatch_setup =
            RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot::from_packed_words(
                &words[cursor..dispatch_setup_end],
            )?;
        cursor = dispatch_setup_end;

        let instance_seed_count = usize::try_from(dispatch_setup.instance_seed_count).ok()?;
        let required_word_count = cursor
            .checked_add(instance_seed_count.checked_mul(
                RenderVirtualGeometryNodeAndClusterCullInstanceSeed::GPU_WORD_COUNT,
            )?)?;
        if words.len() < required_word_count {
            return None;
        }

        let instance_seeds = words[cursor..required_word_count]
            .chunks_exact(RenderVirtualGeometryNodeAndClusterCullInstanceSeed::GPU_WORD_COUNT)
            .map(RenderVirtualGeometryNodeAndClusterCullInstanceSeed::from_packed_words)
            .collect::<Option<Vec<_>>>()?;

        Some(Self {
            global_state,
            dispatch_setup,
            instance_seeds,
        })
    }
}
