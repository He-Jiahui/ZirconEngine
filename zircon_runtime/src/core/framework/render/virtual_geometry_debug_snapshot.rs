use super::scene_extract::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryDebugState, RenderVirtualGeometryInstance,
};

#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryBvhVisualizationNode {
    pub node_id: u32,
    pub parent_node_id: Option<u32>,
    pub child_node_ids: Vec<u32>,
    pub depth: u32,
    pub page_id: u32,
    pub mip_level: u8,
    pub is_leaf: bool,
    pub cluster_ids: Vec<u32>,
    pub selected_cluster_ids: Vec<u32>,
    pub resident_cluster_ids: Vec<u32>,
    pub bounds_center: [f32; 3],
    pub bounds_radius: f32,
    pub screen_space_error: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryBvhVisualizationInstance {
    pub instance_index: u32,
    pub entity: u64,
    pub mesh_name: Option<String>,
    pub source_hint: Option<String>,
    pub nodes: Vec<RenderVirtualGeometryBvhVisualizationNode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryCpuReferenceNodeVisit {
    pub node_id: u32,
    pub depth: u32,
    pub page_id: u32,
    pub mip_level: u8,
    pub is_leaf: bool,
    pub cluster_ids: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryCpuReferenceLeafCluster {
    pub node_id: u32,
    pub cluster_ordinal: u32,
    pub cluster_id: u32,
    pub page_id: u32,
    pub mip_level: u8,
    pub loaded: bool,
    pub parent_cluster_id: Option<u32>,
    pub bounds_center: [f32; 3],
    pub bounds_radius: f32,
    pub screen_space_error: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryCpuReferenceSelectedCluster {
    pub node_id: u32,
    pub cluster_ordinal: u32,
    pub cluster_id: u32,
    pub page_id: u32,
    pub mip_level: u8,
    pub loaded: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryCpuReferencePageClusterMapEntry {
    pub page_id: u32,
    pub cluster_ids: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryCpuReferenceDepthClusterMapEntry {
    pub depth: u32,
    pub cluster_ids: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryCpuReferenceMipClusterMapEntry {
    pub mip_level: u8,
    pub cluster_ids: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryCpuReferenceInstance {
    pub instance_index: u32,
    pub entity: u64,
    pub mesh_name: Option<String>,
    pub source_hint: Option<String>,
    pub visited_nodes: Vec<RenderVirtualGeometryCpuReferenceNodeVisit>,
    pub leaf_clusters: Vec<RenderVirtualGeometryCpuReferenceLeafCluster>,
    pub loaded_leaf_clusters: Vec<RenderVirtualGeometryCpuReferenceLeafCluster>,
    pub mip_accepted_clusters: Vec<RenderVirtualGeometryCpuReferenceLeafCluster>,
    pub selected_clusters: Vec<RenderVirtualGeometryCpuReferenceSelectedCluster>,
    pub page_cluster_map: Vec<RenderVirtualGeometryCpuReferencePageClusterMapEntry>,
    pub loaded_page_cluster_map: Vec<RenderVirtualGeometryCpuReferencePageClusterMapEntry>,
    pub mip_accepted_page_cluster_map: Vec<RenderVirtualGeometryCpuReferencePageClusterMapEntry>,
    pub loaded_mip_cluster_map: Vec<RenderVirtualGeometryCpuReferenceMipClusterMapEntry>,
    pub selected_page_cluster_map: Vec<RenderVirtualGeometryCpuReferencePageClusterMapEntry>,
    pub depth_cluster_map: Vec<RenderVirtualGeometryCpuReferenceDepthClusterMapEntry>,
    pub loaded_depth_cluster_map: Vec<RenderVirtualGeometryCpuReferenceDepthClusterMapEntry>,
    pub mip_accepted_depth_cluster_map: Vec<RenderVirtualGeometryCpuReferenceDepthClusterMapEntry>,
    pub selected_depth_cluster_map: Vec<RenderVirtualGeometryCpuReferenceDepthClusterMapEntry>,
    pub mip_cluster_map: Vec<RenderVirtualGeometryCpuReferenceMipClusterMapEntry>,
    pub selected_mip_cluster_map: Vec<RenderVirtualGeometryCpuReferenceMipClusterMapEntry>,
}

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

fn decode_optional_u32(value: u32) -> Option<u32> {
    (value != u32::MAX).then_some(value)
}

fn encode_virtual_geometry_debug_flags(debug: RenderVirtualGeometryDebugState) -> u32 {
    let mut flags = 0_u32;
    if debug.freeze_cull {
        flags |= RenderVirtualGeometryCullInputSnapshot::DEBUG_FLAG_FREEZE_CULL;
    }
    if debug.visualize_bvh {
        flags |= RenderVirtualGeometryCullInputSnapshot::DEBUG_FLAG_VISUALIZE_BVH;
    }
    if debug.visualize_visbuffer {
        flags |= RenderVirtualGeometryCullInputSnapshot::DEBUG_FLAG_VISUALIZE_VISBUFFER;
    }
    if debug.print_leaf_clusters {
        flags |= RenderVirtualGeometryCullInputSnapshot::DEBUG_FLAG_PRINT_LEAF_CLUSTERS;
    }
    flags
}

fn decode_virtual_geometry_debug_state(
    forced_mip_word: u32,
    debug_flags: u32,
) -> RenderVirtualGeometryDebugState {
    RenderVirtualGeometryDebugState {
        forced_mip: decode_optional_u32(forced_mip_word).and_then(|mip| u8::try_from(mip).ok()),
        freeze_cull: (debug_flags & RenderVirtualGeometryCullInputSnapshot::DEBUG_FLAG_FREEZE_CULL)
            != 0,
        visualize_bvh: (debug_flags
            & RenderVirtualGeometryCullInputSnapshot::DEBUG_FLAG_VISUALIZE_BVH)
            != 0,
        visualize_visbuffer: (debug_flags
            & RenderVirtualGeometryCullInputSnapshot::DEBUG_FLAG_VISUALIZE_VISBUFFER)
            != 0,
        print_leaf_clusters: (debug_flags
            & RenderVirtualGeometryCullInputSnapshot::DEBUG_FLAG_PRINT_LEAF_CLUSTERS)
            != 0,
    }
}

fn encode_cluster_selection_input_source(
    source: RenderVirtualGeometryClusterSelectionInputSource,
) -> u32 {
    match source {
        RenderVirtualGeometryClusterSelectionInputSource::Unavailable => 0,
        RenderVirtualGeometryClusterSelectionInputSource::ExplicitFrameOwned => 1,
        RenderVirtualGeometryClusterSelectionInputSource::PrepareDerivedFrameOwned => 2,
        RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand => 3,
    }
}

fn decode_cluster_selection_input_source(
    word: u32,
) -> RenderVirtualGeometryClusterSelectionInputSource {
    match word {
        1 => RenderVirtualGeometryClusterSelectionInputSource::ExplicitFrameOwned,
        2 => RenderVirtualGeometryClusterSelectionInputSource::PrepareDerivedFrameOwned,
        3 => RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        _ => RenderVirtualGeometryClusterSelectionInputSource::Unavailable,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderVirtualGeometryVisBuffer64Source {
    #[default]
    Unavailable,
    RenderPathClearOnly,
    RenderPathExecutionSelections,
    SnapshotFallback,
    GpuReadbackFallback,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderVirtualGeometryHardwareRasterizationSource {
    #[default]
    Unavailable,
    RenderPathClearOnly,
    RenderPathExecutionSelections,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderVirtualGeometryNodeAndClusterCullSource {
    #[default]
    Unavailable,
    RenderPathClearOnly,
    RenderPathCullInput,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderVirtualGeometrySelectedClusterSource {
    #[default]
    Unavailable,
    RenderPathClearOnly,
    RenderPathExecutionSelections,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderVirtualGeometryClusterSelectionInputSource {
    #[default]
    Unavailable,
    ExplicitFrameOwned,
    PrepareDerivedFrameOwned,
    PrepareOnDemand,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryCullInputSnapshot {
    pub cluster_budget: u32,
    pub page_budget: u32,
    pub instance_count: u32,
    pub cluster_count: u32,
    pub page_count: u32,
    pub visible_entity_count: u32,
    pub visible_cluster_count: u32,
    pub resident_page_count: u32,
    pub pending_page_request_count: u32,
    pub available_page_slot_count: u32,
    pub evictable_page_count: u32,
    pub debug: RenderVirtualGeometryDebugState,
    pub cluster_selection_input_source: RenderVirtualGeometryClusterSelectionInputSource,
}

impl RenderVirtualGeometryCullInputSnapshot {
    pub const GPU_WORD_COUNT: usize = 14;
    const NONE_SENTINEL: u32 = u32::MAX;
    const DEBUG_FLAG_FREEZE_CULL: u32 = 1 << 0;
    const DEBUG_FLAG_VISUALIZE_BVH: u32 = 1 << 1;
    const DEBUG_FLAG_VISUALIZE_VISBUFFER: u32 = 1 << 2;
    const DEBUG_FLAG_PRINT_LEAF_CLUSTERS: u32 = 1 << 3;

    pub fn packed_words(&self) -> [u32; Self::GPU_WORD_COUNT] {
        [
            self.cluster_budget,
            self.page_budget,
            self.instance_count,
            self.cluster_count,
            self.page_count,
            self.visible_entity_count,
            self.visible_cluster_count,
            self.resident_page_count,
            self.pending_page_request_count,
            self.available_page_slot_count,
            self.evictable_page_count,
            self.debug
                .forced_mip
                .map(u32::from)
                .unwrap_or(Self::NONE_SENTINEL),
            encode_virtual_geometry_debug_flags(self.debug),
            encode_cluster_selection_input_source(self.cluster_selection_input_source),
        ]
    }

    pub fn from_packed_words(words: &[u32]) -> Option<Self> {
        if words.len() < Self::GPU_WORD_COUNT {
            return None;
        }

        Some(Self {
            cluster_budget: words[0],
            page_budget: words[1],
            instance_count: words[2],
            cluster_count: words[3],
            page_count: words[4],
            visible_entity_count: words[5],
            visible_cluster_count: words[6],
            resident_page_count: words[7],
            pending_page_request_count: words[8],
            available_page_slot_count: words[9],
            evictable_page_count: words[10],
            debug: decode_virtual_geometry_debug_state(words[11], words[12]),
            cluster_selection_input_source: decode_cluster_selection_input_source(words[13]),
        })
    }
}

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
    CompatFixedFanout,
    AuthoredHierarchy,
}

impl RenderVirtualGeometryNodeAndClusterCullTraversalChildSource {
    const fn packed_word(self) -> u32 {
        match self {
            Self::None => 0,
            Self::CompatFixedFanout => 1,
            Self::AuthoredHierarchy => 2,
        }
    }

    fn from_packed_word(word: u32) -> Option<Self> {
        match word {
            0 => Some(Self::None),
            1 => Some(Self::CompatFixedFanout),
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderVirtualGeometryDebugSnapshot {
    pub instances: Vec<RenderVirtualGeometryInstance>,
    pub debug: RenderVirtualGeometryDebugState,
    pub cull_input: RenderVirtualGeometryCullInputSnapshot,
    pub cluster_selection_input_source: RenderVirtualGeometryClusterSelectionInputSource,
    pub cpu_reference_instances: Vec<RenderVirtualGeometryCpuReferenceInstance>,
    pub bvh_visualization_instances: Vec<RenderVirtualGeometryBvhVisualizationInstance>,
    pub visible_cluster_ids: Vec<u32>,
    pub selected_clusters: Vec<RenderVirtualGeometrySelectedCluster>,
    pub selected_clusters_source: RenderVirtualGeometrySelectedClusterSource,
    pub node_and_cluster_cull_source: RenderVirtualGeometryNodeAndClusterCullSource,
    pub node_and_cluster_cull_record_count: u32,
    pub node_and_cluster_cull_instance_seeds:
        Vec<RenderVirtualGeometryNodeAndClusterCullInstanceSeed>,
    pub node_and_cluster_cull_instance_work_items:
        Vec<RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem>,
    pub node_and_cluster_cull_cluster_work_items:
        Vec<RenderVirtualGeometryNodeAndClusterCullClusterWorkItem>,
    pub node_and_cluster_cull_child_work_items:
        Vec<RenderVirtualGeometryNodeAndClusterCullChildWorkItem>,
    pub node_and_cluster_cull_traversal_records:
        Vec<RenderVirtualGeometryNodeAndClusterCullTraversalRecord>,
    pub node_and_cluster_cull_hierarchy_child_ids: Vec<u32>,
    pub node_and_cluster_cull_page_request_ids: Vec<u32>,
    pub node_and_cluster_cull_dispatch_setup:
        Option<RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot>,
    pub node_and_cluster_cull_launch_worklist:
        Option<RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot>,
    pub node_and_cluster_cull_global_state:
        Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
    pub hardware_rasterization_records: Vec<RenderVirtualGeometryHardwareRasterizationRecord>,
    pub hardware_rasterization_source: RenderVirtualGeometryHardwareRasterizationSource,
    pub visbuffer_debug_marks: Vec<RenderVirtualGeometryVisBufferMark>,
    pub visbuffer64_source: RenderVirtualGeometryVisBuffer64Source,
    pub visbuffer64_clear_value: u64,
    pub visbuffer64_entries: Vec<RenderVirtualGeometryVisBuffer64Entry>,
    pub requested_pages: Vec<u32>,
    pub resident_pages: Vec<u32>,
    pub dirty_requested_pages: Vec<u32>,
    pub evictable_pages: Vec<u32>,
    pub resident_page_inspections: Vec<RenderVirtualGeometryResidentPageInspection>,
    pub pending_page_request_inspections: Vec<RenderVirtualGeometryPageRequestInspection>,
    pub available_page_slots: Vec<u32>,
    pub evictable_page_inspections: Vec<RenderVirtualGeometryResidentPageInspection>,
    pub leaf_clusters: Vec<RenderVirtualGeometryCluster>,
    pub execution_segment_count: u32,
    pub execution_page_count: u32,
    pub execution_resident_segment_count: u32,
    pub execution_pending_segment_count: u32,
    pub execution_missing_segment_count: u32,
    pub execution_repeated_draw_count: u32,
    pub execution_indirect_offsets: Vec<u64>,
    pub execution_segments: Vec<RenderVirtualGeometryExecutionSegment>,
    pub submission_order: Vec<RenderVirtualGeometrySubmissionEntry>,
    pub submission_records: Vec<RenderVirtualGeometrySubmissionRecord>,
}

#[cfg(test)]
mod tests {
    use super::{
        RenderVirtualGeometryClusterSelectionInputSource, RenderVirtualGeometryCullInputSnapshot,
        RenderVirtualGeometryNodeAndClusterCullChildWorkItem,
        RenderVirtualGeometryNodeAndClusterCullClusterWorkItem,
        RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
        RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
        RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
        RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
        RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
        RenderVirtualGeometryNodeAndClusterCullTraversalChildSource,
        RenderVirtualGeometryNodeAndClusterCullTraversalOp,
        RenderVirtualGeometryNodeAndClusterCullTraversalRecord,
    };
    use crate::core::framework::render::RenderVirtualGeometryDebugState;

    #[test]
    fn cull_input_snapshot_roundtrips_through_gpu_word_layout() {
        let snapshot = RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 12,
            page_budget: 7,
            instance_count: 3,
            cluster_count: 42,
            page_count: 9,
            visible_entity_count: 2,
            visible_cluster_count: 17,
            resident_page_count: 5,
            pending_page_request_count: 4,
            available_page_slot_count: 6,
            evictable_page_count: 1,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(10),
                freeze_cull: true,
                visualize_bvh: true,
                visualize_visbuffer: false,
                print_leaf_clusters: true,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        };

        let words = snapshot.packed_words();
        let decoded = RenderVirtualGeometryCullInputSnapshot::from_packed_words(&words)
            .expect("expected cull-input snapshot to decode from its stable GPU word layout");

        assert_eq!(
            decoded, snapshot,
            "expected the future NaniteGlobalStateBuffer-compatible word layout to round-trip every authored budget/debug/provenance field without host-side reinterpretation"
        );
    }

    #[test]
    fn node_and_cluster_cull_global_state_roundtrips_through_gpu_word_layout() {
        let snapshot = RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot {
            cull_input: RenderVirtualGeometryCullInputSnapshot {
                cluster_budget: 12,
                page_budget: 7,
                instance_count: 3,
                cluster_count: 42,
                page_count: 9,
                visible_entity_count: 2,
                visible_cluster_count: 17,
                resident_page_count: 5,
                pending_page_request_count: 4,
                available_page_slot_count: 6,
                evictable_page_count: 1,
                debug: RenderVirtualGeometryDebugState {
                    forced_mip: Some(10),
                    freeze_cull: true,
                    visualize_bvh: true,
                    visualize_visbuffer: false,
                    print_leaf_clusters: true,
                },
                cluster_selection_input_source:
                    RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
            },
            viewport_size: [1920, 1080],
            camera_translation: [1.25, -2.5, 3.75],
            child_split_screen_space_error_threshold: 0.375,
            child_frustum_culling_enabled: true,
            view_proj: [
                [1.0, 2.0, 3.0, 4.0],
                [5.0, 6.0, 7.0, 8.0],
                [9.0, 10.0, 11.0, 12.0],
                [13.0, 14.0, 15.0, 16.0],
            ],
            previous_camera_translation: [-1.25, 2.5, -3.75],
            previous_view_proj: [
                [17.0, 18.0, 19.0, 20.0],
                [21.0, 22.0, 23.0, 24.0],
                [25.0, 26.0, 27.0, 28.0],
                [29.0, 30.0, 31.0, 32.0],
            ],
        };

        let words = snapshot.packed_words();
        let decoded =
            RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot::from_packed_words(&words)
                .expect("expected node-and-cluster-cull global state to decode");

        assert_eq!(
            decoded, snapshot,
            "expected the NodeAndClusterCull global-state word layout to round-trip cull input, viewport, camera origin, and view-projection data without host-side reinterpretation"
        );
    }

    #[test]
    fn node_and_cluster_cull_instance_seed_roundtrips_through_gpu_word_layout() {
        let seed = RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
            instance_index: 3,
            entity: 42,
            cluster_offset: 10,
            cluster_count: 4,
            page_offset: 7,
            page_count: 2,
        };

        let words = seed.packed_words();
        let decoded =
            RenderVirtualGeometryNodeAndClusterCullInstanceSeed::from_packed_words(&words)
                .expect("expected node-and-cluster-cull instance seed to decode");

        assert_eq!(
            decoded, seed,
            "expected the NodeAndClusterCull instance-seed word layout to round-trip the per-instance root worklist contract without host-side reinterpretation"
        );
    }

    #[test]
    fn node_and_cluster_cull_instance_work_item_roundtrips_through_gpu_word_layout() {
        let work_item = RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem {
            instance_index: 3,
            entity: 42,
            cluster_offset: 10,
            cluster_count: 4,
            page_offset: 7,
            page_count: 2,
            cluster_budget: 12,
            page_budget: 7,
            forced_mip: Some(10),
        };

        let words = work_item.packed_words();
        let decoded =
            RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem::from_packed_words(&words)
                .expect("expected node-and-cluster-cull instance work item to decode");

        assert_eq!(
            decoded, work_item,
            "expected the NodeAndClusterCull instance-work-item word layout to round-trip the first compute-stub output contract so the renderer-owned GPU buffer and compat pass can share one typed per-instance seam"
        );
    }

    #[test]
    fn node_and_cluster_cull_cluster_work_item_roundtrips_through_gpu_word_layout() {
        let work_item = RenderVirtualGeometryNodeAndClusterCullClusterWorkItem {
            instance_index: 3,
            entity: 42,
            cluster_array_index: 10,
            hierarchy_node_id: Some(7),
            cluster_budget: 12,
            page_budget: 5,
            forced_mip: Some(2),
        };

        let words = work_item.packed_words();
        let decoded =
            RenderVirtualGeometryNodeAndClusterCullClusterWorkItem::from_packed_words(&words)
                .expect("expected node-and-cluster-cull cluster work item to decode");

        assert_eq!(
            decoded, work_item,
            "expected the public NodeAndClusterCull cluster-work-item word layout to round-trip the per-cluster traversal input contract used by renderer-owned buffers and debug snapshots"
        );
    }

    #[test]
    fn node_and_cluster_cull_child_work_item_roundtrips_through_gpu_word_layout() {
        let work_item = RenderVirtualGeometryNodeAndClusterCullChildWorkItem {
            instance_index: 3,
            entity: 42,
            parent_cluster_array_index: 10,
            parent_hierarchy_node_id: Some(7),
            child_node_id: 70,
            child_table_index: 2,
            traversal_index: 9,
            cluster_budget: 12,
            page_budget: 5,
            forced_mip: Some(2),
        };

        let words = work_item.packed_words();
        let decoded =
            RenderVirtualGeometryNodeAndClusterCullChildWorkItem::from_packed_words(&words)
                .expect("expected node-and-cluster-cull child work item to decode");

        assert_eq!(
            decoded, work_item,
            "expected the public NodeAndClusterCull child-work-item word layout to round-trip authored child traversal input without private renderer-side reinterpretation"
        );
    }

    #[test]
    fn node_and_cluster_cull_traversal_record_roundtrips_through_gpu_word_layout() {
        let record = RenderVirtualGeometryNodeAndClusterCullTraversalRecord {
            op: RenderVirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild,
            child_source:
                RenderVirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy,
            instance_index: 3,
            entity: 42,
            cluster_array_index: 10,
            hierarchy_node_id: Some(7),
            node_cluster_start: 70,
            node_cluster_count: 4,
            child_base: 2,
            child_count: 3,
            traversal_index: 9,
            cluster_budget: 12,
            page_budget: 5,
            forced_mip: Some(2),
        };

        let words = record.packed_words();
        let decoded =
            RenderVirtualGeometryNodeAndClusterCullTraversalRecord::from_packed_words(&words)
                .expect("expected node-and-cluster-cull traversal record to decode");

        assert_eq!(
            decoded, record,
            "expected the public NodeAndClusterCull traversal-record word layout to round-trip VisitNode/StoreCluster/EnqueueChild decisions without private renderer-side reinterpretation"
        );
    }

    #[test]
    fn node_and_cluster_cull_launch_worklist_roundtrips_through_gpu_word_layout() {
        let snapshot = RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot {
            global_state: RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot {
                cull_input: RenderVirtualGeometryCullInputSnapshot {
                    cluster_budget: 12,
                    page_budget: 7,
                    instance_count: 3,
                    cluster_count: 42,
                    page_count: 9,
                    visible_entity_count: 2,
                    visible_cluster_count: 17,
                    resident_page_count: 5,
                    pending_page_request_count: 4,
                    available_page_slot_count: 6,
                    evictable_page_count: 1,
                    debug: RenderVirtualGeometryDebugState {
                        forced_mip: Some(10),
                        freeze_cull: true,
                        visualize_bvh: true,
                        visualize_visbuffer: false,
                        print_leaf_clusters: true,
                    },
                    cluster_selection_input_source:
                        RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
                },
                viewport_size: [1920, 1080],
                camera_translation: [1.25, -2.5, 3.75],
                child_split_screen_space_error_threshold: 0.375,
                child_frustum_culling_enabled: true,
                view_proj: [
                    [1.0, 2.0, 3.0, 4.0],
                    [5.0, 6.0, 7.0, 8.0],
                    [9.0, 10.0, 11.0, 12.0],
                    [13.0, 14.0, 15.0, 16.0],
                ],
                previous_camera_translation: [-1.25, 2.5, -3.75],
                previous_view_proj: [
                    [17.0, 18.0, 19.0, 20.0],
                    [21.0, 22.0, 23.0, 24.0],
                    [25.0, 26.0, 27.0, 28.0],
                    [29.0, 30.0, 31.0, 32.0],
                ],
            },
            dispatch_setup: RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot {
                instance_seed_count: 2,
                cluster_budget: 12,
                page_budget: 7,
                workgroup_size: 64,
                dispatch_group_count: [1, 1, 1],
            },
            instance_seeds: vec![
                RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
                    instance_index: 0,
                    entity: 42,
                    cluster_offset: 10,
                    cluster_count: 4,
                    page_offset: 7,
                    page_count: 2,
                },
                RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
                    instance_index: 1,
                    entity: 99,
                    cluster_offset: 20,
                    cluster_count: 8,
                    page_offset: 11,
                    page_count: 3,
                },
            ],
        };

        let words = snapshot.packed_words();
        let decoded =
            RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot::from_packed_words(
                &words,
            )
            .expect("expected node-and-cluster-cull launch worklist to decode");

        assert_eq!(
            decoded, snapshot,
            "expected the NodeAndClusterCull launch-worklist word layout to round-trip the combined global state, dispatch setup, and root seeds so the renderer-owned GPU buffer can stay the single compat compute-stub contract"
        );
    }
}
