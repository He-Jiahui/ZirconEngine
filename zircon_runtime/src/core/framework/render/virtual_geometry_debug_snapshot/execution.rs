use super::encoding::decode_optional_u32;
use crate::core::framework::render::render_mesh_stable_instance_key;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometrySubmissionEntry {
    pub instance_index: Option<u32>,
    pub entity: u64,
    pub page_id: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometrySubmissionRecord {
    pub instance_index: Option<u32>,
    pub entity: u64,
    pub page_id: u32,
    pub draw_ref_index: Option<u32>,
    pub submission_index: u32,
    pub draw_ref_rank: u32,
    pub original_index: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderVirtualGeometryExecutionState {
    Resident,
    PendingUpload,
    Missing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryExecutionSegment {
    pub original_index: u32,
    pub instance_index: Option<u32>,
    pub entity: u64,
    pub stable_instance_key: u64,
    pub page_id: u32,
    pub draw_ref_index: u32,
    pub submission_index: Option<u32>,
    pub draw_ref_rank: Option<u32>,
    pub cluster_start_ordinal: u32,
    pub cluster_span_count: u32,
    pub cluster_total_count: u32,
    pub submission_slot: Option<u32>,
    pub state: RenderVirtualGeometryExecutionState,
    pub lineage_depth: u32,
    pub lod_level: u8,
    pub frontier_rank: u32,
}

impl RenderVirtualGeometryExecutionSegment {
    /// Preserves execution snapshots produced before virtual geometry carried the render key.
    pub fn stable_instance_key_or_legacy(&self) -> u64 {
        if self.stable_instance_key == 0 {
            render_mesh_stable_instance_key(self.entity, 0)
        } else {
            self.stable_instance_key
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometrySelectedCluster {
    pub instance_index: Option<u32>,
    pub entity: u64,
    pub cluster_id: u32,
    pub cluster_ordinal: u32,
    pub page_id: u32,
    pub lod_level: u8,
    pub state: RenderVirtualGeometryExecutionState,
}

impl RenderVirtualGeometrySelectedCluster {
    pub const GPU_WORD_COUNT: usize = 8;
    const NONE_SENTINEL: u32 = u32::MAX;

    pub fn packed_words(&self) -> [u32; Self::GPU_WORD_COUNT] {
        let entity_low = (self.entity & u64::from(u32::MAX)) as u32;
        let entity_high = (self.entity >> 32) as u32;
        [
            self.instance_index.unwrap_or(Self::NONE_SENTINEL),
            entity_low,
            entity_high,
            self.cluster_id,
            self.cluster_ordinal,
            self.page_id,
            u32::from(self.lod_level),
            match self.state {
                RenderVirtualGeometryExecutionState::Resident => 0,
                RenderVirtualGeometryExecutionState::PendingUpload => 1,
                RenderVirtualGeometryExecutionState::Missing => 2,
            },
        ]
    }

    pub fn from_packed_words(words: &[u32]) -> Option<Self> {
        if words.len() < Self::GPU_WORD_COUNT {
            return None;
        }

        Some(Self {
            instance_index: decode_optional_u32(words[0]),
            entity: u64::from(words[1]) | (u64::from(words[2]) << 32),
            cluster_id: words[3],
            cluster_ordinal: words[4],
            page_id: words[5],
            lod_level: u8::try_from(words[6]).unwrap_or(u8::MAX),
            state: match words[7] {
                0 => RenderVirtualGeometryExecutionState::Resident,
                1 => RenderVirtualGeometryExecutionState::PendingUpload,
                _ => RenderVirtualGeometryExecutionState::Missing,
            },
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryHardwareRasterizationRecord {
    pub instance_index: Option<u32>,
    pub entity: u64,
    pub cluster_id: u32,
    pub cluster_ordinal: u32,
    pub page_id: u32,
    pub lod_level: u8,
    pub submission_index: u32,
    pub submission_page_id: u32,
    pub submission_lod_level: u8,
    pub entity_cluster_start_ordinal: u32,
    pub entity_cluster_span_count: u32,
    pub entity_cluster_total_count: u32,
    pub lineage_depth: u32,
    pub frontier_rank: u32,
    pub resident_slot: Option<u32>,
    pub submission_slot: Option<u32>,
    pub state: RenderVirtualGeometryExecutionState,
}

impl RenderVirtualGeometryHardwareRasterizationRecord {
    pub const GPU_WORD_COUNT: usize = 18;
    const NONE_SENTINEL: u32 = u32::MAX;

    pub fn packed_words(&self) -> [u32; Self::GPU_WORD_COUNT] {
        let entity_low = (self.entity & u64::from(u32::MAX)) as u32;
        let entity_high = (self.entity >> 32) as u32;
        [
            self.instance_index.unwrap_or(Self::NONE_SENTINEL),
            entity_low,
            entity_high,
            self.cluster_id,
            self.cluster_ordinal,
            self.page_id,
            u32::from(self.lod_level),
            self.submission_index,
            self.submission_page_id,
            u32::from(self.submission_lod_level),
            self.entity_cluster_start_ordinal,
            self.entity_cluster_span_count,
            self.entity_cluster_total_count,
            self.lineage_depth,
            self.frontier_rank,
            self.resident_slot.unwrap_or(Self::NONE_SENTINEL),
            self.submission_slot.unwrap_or(Self::NONE_SENTINEL),
            match self.state {
                RenderVirtualGeometryExecutionState::Resident => 0,
                RenderVirtualGeometryExecutionState::PendingUpload => 1,
                RenderVirtualGeometryExecutionState::Missing => 2,
            },
        ]
    }

    pub fn from_packed_words(words: &[u32]) -> Option<Self> {
        if words.len() < Self::GPU_WORD_COUNT {
            return None;
        }

        Some(Self {
            instance_index: decode_optional_u32(words[0]),
            entity: u64::from(words[1]) | (u64::from(words[2]) << 32),
            cluster_id: words[3],
            cluster_ordinal: words[4],
            page_id: words[5],
            lod_level: u8::try_from(words[6]).unwrap_or(u8::MAX),
            submission_index: words[7],
            submission_page_id: words[8],
            submission_lod_level: u8::try_from(words[9]).unwrap_or(u8::MAX),
            entity_cluster_start_ordinal: words[10],
            entity_cluster_span_count: words[11],
            entity_cluster_total_count: words[12],
            lineage_depth: words[13],
            frontier_rank: words[14],
            resident_slot: decode_optional_u32(words[15]),
            submission_slot: decode_optional_u32(words[16]),
            state: match words[17] {
                0 => RenderVirtualGeometryExecutionState::Resident,
                1 => RenderVirtualGeometryExecutionState::PendingUpload,
                _ => RenderVirtualGeometryExecutionState::Missing,
            },
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryResidentPageInspection {
    pub page_id: u32,
    pub slot: u32,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryPageRequestInspection {
    pub page_id: u32,
    pub size_bytes: u64,
    pub generation: u64,
    pub frontier_rank: u32,
    pub assigned_slot: Option<u32>,
    pub recycled_page_id: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryVisBufferMark {
    pub instance_index: Option<u32>,
    pub entity: u64,
    pub cluster_id: u32,
    pub page_id: u32,
    pub lod_level: u8,
    pub state: RenderVirtualGeometryExecutionState,
    pub color_rgba: [u8; 4],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryVisBuffer64Entry {
    pub entry_index: u32,
    pub packed_value: u64,
    pub instance_index: Option<u32>,
    pub entity: u64,
    pub cluster_id: u32,
    pub page_id: u32,
    pub lod_level: u8,
    pub state: RenderVirtualGeometryExecutionState,
}

impl RenderVirtualGeometryVisBuffer64Entry {
    pub const CLEAR_VALUE: u64 = 0;
    const CLUSTER_BITS: u64 = 20;
    const PAGE_BITS: u64 = 20;
    const INSTANCE_BITS: u64 = 16;
    const LOD_BITS: u64 = 6;
    const CLUSTER_MASK: u64 = (1_u64 << Self::CLUSTER_BITS) - 1;
    const PAGE_MASK: u64 = (1_u64 << Self::PAGE_BITS) - 1;
    const INSTANCE_MASK: u64 = (1_u64 << Self::INSTANCE_BITS) - 1;
    const LOD_MASK: u64 = (1_u64 << Self::LOD_BITS) - 1;
    const PAGE_SHIFT: u64 = Self::CLUSTER_BITS;
    const INSTANCE_SHIFT: u64 = Self::PAGE_SHIFT + Self::PAGE_BITS;
    const LOD_SHIFT: u64 = Self::INSTANCE_SHIFT + Self::INSTANCE_BITS;
    const STATE_SHIFT: u64 = Self::LOD_SHIFT + Self::LOD_BITS;

    pub fn packed_value_for(
        instance_index: Option<u32>,
        cluster_id: u32,
        page_id: u32,
        lod_level: u8,
        state: RenderVirtualGeometryExecutionState,
    ) -> u64 {
        let encoded_instance =
            u64::from(instance_index.unwrap_or(u16::MAX as u32)) & Self::INSTANCE_MASK;
        let encoded_state = match state {
            RenderVirtualGeometryExecutionState::Resident => 0_u64,
            RenderVirtualGeometryExecutionState::PendingUpload => 1_u64,
            RenderVirtualGeometryExecutionState::Missing => 2_u64,
        };

        (u64::from(cluster_id) & Self::CLUSTER_MASK)
            | ((u64::from(page_id) & Self::PAGE_MASK) << Self::PAGE_SHIFT)
            | (encoded_instance << Self::INSTANCE_SHIFT)
            | ((u64::from(lod_level) & Self::LOD_MASK) << Self::LOD_SHIFT)
            | (encoded_state << Self::STATE_SHIFT)
    }

    pub fn from_selected_cluster(
        entry_index: u32,
        cluster: &RenderVirtualGeometrySelectedCluster,
    ) -> Self {
        Self {
            entry_index,
            packed_value: Self::packed_value_for(
                cluster.instance_index,
                cluster.cluster_id,
                cluster.page_id,
                cluster.lod_level,
                cluster.state,
            ),
            instance_index: cluster.instance_index,
            entity: cluster.entity,
            cluster_id: cluster.cluster_id,
            page_id: cluster.page_id,
            lod_level: cluster.lod_level,
            state: cluster.state,
        }
    }
}
