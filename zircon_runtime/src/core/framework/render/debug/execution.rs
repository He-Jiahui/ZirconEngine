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

/// 把绘制提交映回源实例、页与驻留状态，供调试快照解释重复绘制和退化路径。
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

/// Exact per-draw identity copied from its original execution segment in this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderVirtualGeometryExecutionIdentity {
    pub original_index: u32,
    pub draw_ref_index: u32,
    pub stable_instance_key: u64,
    pub instance_index: Option<u32>,
    pub entity: u64,
    pub page_id: u32,
    pub submission_index: Option<u32>,
    pub draw_ref_rank: Option<u32>,
    pub cluster_start_ordinal: u32,
    pub cluster_span_count: u32,
    pub submission_slot: Option<u32>,
}
impl RenderVirtualGeometryExecutionIdentity {
    pub fn from_segment(segment: &RenderVirtualGeometryExecutionSegment) -> Self {
        Self {
            original_index: segment.original_index,
            draw_ref_index: segment.draw_ref_index,
            stable_instance_key: segment.stable_instance_key,
            instance_index: segment.instance_index,
            entity: segment.entity,
            page_id: segment.page_id,
            submission_index: segment.submission_index,
            draw_ref_rank: segment.draw_ref_rank,
            cluster_start_ordinal: segment.cluster_start_ordinal,
            cluster_span_count: segment.cluster_span_count,
            submission_slot: segment.submission_slot,
        }
    }
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

/// 图形运行时选择的簇记录；固定字布局用于调试读回而非持久化资产格式。
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

/// 64 位可视缓冲调试条目；编码后不含完整 entity，解码方只能恢复位域内的身份。
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

    // TODO: [CR-RENDER-MISC-0002] 核实簇、页、实例和 LOD 分配上界；此处直接截断为 20/20/16/6 位，超界 ID 会别名化。
    /// 调用方须保证各 ID 落在位域内；结果作为读回流中的压缩身份。
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

/// Exact graph execution input, independent of optional diagnostic snapshots.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryExecutionInputs {
    pub selections: Option<Vec<RenderVirtualGeometryClusterSelectionInput>>,
    pub prepare: Option<RenderVirtualGeometryPrepareFrameInput>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryClusterSelectionInput {
    pub submission_index: u32,
    pub instance_index: Option<u32>,
    pub entity: u64,
    pub cluster_id: u32,
    pub cluster_ordinal: u32,
    pub page_id: u32,
    pub lod_level: u8,
    pub submission_page_id: u32,
    pub submission_lod_level: u8,
    pub entity_cluster_start_ordinal: usize,
    pub entity_cluster_span_count: usize,
    pub entity_cluster_total_count: usize,
    pub lineage_depth: u32,
    pub frontier_rank: u32,
    pub resident_slot: Option<u32>,
    pub submission_slot: Option<u32>,
    pub state: RenderVirtualGeometryExecutionState,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryPrepareFrameInput {
    pub visible_entities: Vec<u64>,
    pub visible_clusters: Vec<RenderVirtualGeometryPrepareClusterInput>,
    pub cluster_draw_segments: Vec<RenderVirtualGeometryPrepareDrawSegmentInput>,
    pub resident_pages: Vec<RenderVirtualGeometryPreparePageInput>,
    pub pending_page_requests: Vec<RenderVirtualGeometryPrepareRequestInput>,
    pub available_slots: Vec<u32>,
    pub evictable_pages: Vec<RenderVirtualGeometryPreparePageInput>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryPrepareClusterInput {
    pub entity: u64,
    pub cluster_id: u32,
    pub page_id: u32,
    pub lod_level: u8,
    pub resident_slot: Option<u32>,
    pub state: RenderVirtualGeometryExecutionState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryPrepareDrawSegmentInput {
    pub entity: u64,
    pub cluster_id: u32,
    pub page_id: u32,
    pub resident_slot: Option<u32>,
    pub cluster_ordinal: u32,
    pub cluster_span_count: u32,
    pub cluster_count: u32,
    pub lineage_depth: u32,
    pub lod_level: u8,
    pub state: RenderVirtualGeometryExecutionState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryPreparePageInput {
    pub page_id: u32,
    pub slot: u32,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryPrepareRequestInput {
    pub page_id: u32,
    pub size_bytes: u64,
    pub generation: u64,
    pub frontier_rank: u32,
    pub assigned_slot: Option<u32>,
    pub recycled_page_id: Option<u32>,
}

/// Normal frame-owned VG execution facts. Debug display can be disabled independently.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderVirtualGeometryExecutionData {
    pub instances: Vec<super::super::scene_extract::RenderVirtualGeometryInstance>,
    pub resident_page_payloads: Vec<super::page_payload::RenderVirtualGeometryPagePayload>,
    pub debug: super::super::scene_extract::RenderVirtualGeometryDebugState,
    pub cull_input: super::cull_input::RenderVirtualGeometryCullInputSnapshot,
    pub selected_clusters: Vec<RenderVirtualGeometrySelectedCluster>,
    pub execution_segments: Vec<RenderVirtualGeometryExecutionSegment>,
    pub prepared_cpu_frame: Option<std::sync::Arc<RenderVirtualGeometryPreparedCpuFrame>>,
    /// CPU author/asset attributes only; selection and completion remain GPU-owned.
    pub author_cluster_catalog: Vec<crate::core::framework::render::RenderVirtualGeometryCluster>,
    /// Per GPU upload row: (instance, original array index, normal policy total).
    pub candidate_author_indices: Vec<(u32, u32, u32)>,
}

/// CPU policy output from the real provider prepare owner; never a mapped GPU result.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryPreparedCpuInspection {
    pub source: super::sources::RenderVirtualGeometryNodeAndClusterCullSource,
    pub record_count: u32,
    pub cull_input: Option<super::cull_input::RenderVirtualGeometryCullInputSnapshot>,
    pub global_state: Option<
        super::node_and_cluster_cull::RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    >,
    pub dispatch_setup: Option<
        super::node_and_cluster_cull::RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    >,
    pub launch_worklist: Option<
        super::node_and_cluster_cull::RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
    >,
    pub instance_seeds:
        Vec<super::node_and_cluster_cull::RenderVirtualGeometryNodeAndClusterCullInstanceSeed>,
    pub instance_work_items:
        Vec<super::node_and_cluster_cull::RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem>,
    pub cluster_work_items:
        Vec<super::node_and_cluster_cull::RenderVirtualGeometryNodeAndClusterCullClusterWorkItem>,
    pub hierarchy_child_ids: Vec<u32>,
    pub child_work_items:
        Vec<super::node_and_cluster_cull::RenderVirtualGeometryNodeAndClusterCullChildWorkItem>,
    pub traversal_records:
        Vec<super::node_and_cluster_cull::RenderVirtualGeometryNodeAndClusterCullTraversalRecord>,
    pub page_request_ids: Vec<u32>,
}
impl RenderVirtualGeometryPreparedCpuInspection {
    pub fn empty(source: super::sources::RenderVirtualGeometryNodeAndClusterCullSource) -> Self {
        Self {
            source,
            record_count: 0,
            cull_input: None,
            global_state: None,
            dispatch_setup: None,
            launch_worklist: None,
            instance_seeds: Vec::new(),
            instance_work_items: Vec::new(),
            cluster_work_items: Vec::new(),
            hierarchy_child_ids: Vec::new(),
            child_work_items: Vec::new(),
            traversal_records: Vec::new(),
            page_request_ids: Vec::new(),
        }
    }
}

/// Only the successful current-frame feedback may acknowledge this exact preparation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryCpuPrepareToken {
    pub generation: u64,
    pub revision: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryPreparedCpuFrame {
    pub inspection: RenderVirtualGeometryPreparedCpuInspection,
    pub commit_token: Option<RenderVirtualGeometryCpuPrepareToken>,
    pub execution_inputs: RenderVirtualGeometryExecutionInputs,
    /// Original page-owner facts, independent of paired input overrides.
    pub author_resident_page_slots: Vec<(u32, u32)>,
    /// (page_id, frontier_rank, assigned_slot, recycled_page_id), original order.
    pub author_pending_page_requests: Vec<(u32, u32, Option<u32>, Option<u32>)>,
    /// Author declarations from the paired input; actual visibility stays GPU-owned.
    pub paired_execution_segments: Option<Vec<RenderVirtualGeometryExecutionSegment>>,
}
impl RenderVirtualGeometryPreparedCpuFrame {
    pub fn unavailable() -> Self {
        Self {
            inspection: RenderVirtualGeometryPreparedCpuInspection::empty(
                super::sources::RenderVirtualGeometryNodeAndClusterCullSource::Unavailable,
            ),
            commit_token: None,
            execution_inputs: RenderVirtualGeometryExecutionInputs::default(),
            author_resident_page_slots: Vec::new(),
            author_pending_page_requests: Vec::new(),
            paired_execution_segments: None,
        }
    }
}
