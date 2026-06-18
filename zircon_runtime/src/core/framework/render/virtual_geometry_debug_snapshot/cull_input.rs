use super::super::scene_extract::RenderVirtualGeometryDebugState;
use super::encoding::decode_optional_u32;
use super::sources::RenderVirtualGeometryClusterSelectionInputSource;

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
