use super::{
    RenderVirtualGeometryDebugSnapshotReadbackStreamFootprint,
    RenderVirtualGeometryDebugSnapshotReadbackStreamReport,
    RenderVirtualGeometryDebugSnapshotReadbackStreamSection,
    RenderVirtualGeometryDebugSnapshotReadbackStreamSummary,
    RenderVirtualGeometryDebugSnapshotReadbackStreams,
    RenderVirtualGeometryNodeAndClusterCullWordStreams, RenderVirtualGeometryRenderPathWordStreams,
    RenderVirtualGeometryVisBuffer64ReadbackStream, U32_WORD_BYTE_COUNT, U64_U32_WORD_COUNT,
};

impl RenderVirtualGeometryNodeAndClusterCullWordStreams {
    pub fn has_payload(&self) -> bool {
        self.payload_u32_word_count() != 0
    }

    pub fn is_empty(&self) -> bool {
        !self.has_payload()
    }

    pub fn u32_word_count(&self) -> usize {
        self.global_state.as_ref().map_or(0, Vec::len)
            + self.dispatch_setup.as_ref().map_or(0, Vec::len)
            + self.launch_worklist.as_ref().map_or(0, Vec::len)
            + self.instance_seeds.len()
            + self.instance_work_items.len()
            + self.cluster_work_items.len()
            + self.child_work_items.len()
            + self.traversal_records.len()
            + self.hierarchy_child_ids.len()
            + self.page_request_ids.len()
    }

    pub fn payload_u32_word_count(&self) -> usize {
        self.u32_word_count()
    }

    pub fn payload_byte_count(&self) -> usize {
        self.payload_u32_word_count() * U32_WORD_BYTE_COUNT
    }

    pub fn byte_count(&self) -> usize {
        self.u32_word_count() * U32_WORD_BYTE_COUNT
    }
}

impl RenderVirtualGeometryRenderPathWordStreams {
    pub fn has_payload(&self) -> bool {
        self.payload_u32_word_count() != 0
    }

    pub fn is_empty(&self) -> bool {
        !self.has_payload()
    }

    pub fn u32_word_count(&self) -> usize {
        self.selected_clusters.len() + self.hardware_rasterization_records.len()
    }

    pub fn payload_u32_word_count(&self) -> usize {
        self.u32_word_count()
    }

    pub fn payload_byte_count(&self) -> usize {
        self.payload_u32_word_count() * U32_WORD_BYTE_COUNT
    }

    pub fn byte_count(&self) -> usize {
        self.u32_word_count() * U32_WORD_BYTE_COUNT
    }
}

impl RenderVirtualGeometryVisBuffer64ReadbackStream {
    pub fn has_payload(&self) -> bool {
        self.payload_u32_word_count() != 0
    }

    pub fn is_empty(&self) -> bool {
        !self.has_payload()
    }

    pub fn u32_word_count(&self) -> usize {
        U64_U32_WORD_COUNT
            + self.entry_indices.len()
            + (self.packed_values.len() * U64_U32_WORD_COUNT)
    }

    pub fn payload_u32_word_count(&self) -> usize {
        self.entry_indices.len() + (self.packed_values.len() * U64_U32_WORD_COUNT)
    }

    pub fn payload_byte_count(&self) -> usize {
        self.payload_u32_word_count() * U32_WORD_BYTE_COUNT
    }

    pub fn byte_count(&self) -> usize {
        self.u32_word_count() * U32_WORD_BYTE_COUNT
    }
}

impl RenderVirtualGeometryDebugSnapshotReadbackStreams {
    pub fn has_payload(&self) -> bool {
        self.payload_u32_word_count() != 0
    }

    pub fn is_empty(&self) -> bool {
        !self.has_payload()
    }

    pub fn u32_word_count(&self) -> usize {
        self.node_and_cluster_cull.u32_word_count()
            + self.render_path.u32_word_count()
            + self.visbuffer64.u32_word_count()
    }

    pub fn payload_u32_word_count(&self) -> usize {
        self.node_and_cluster_cull.payload_u32_word_count()
            + self.render_path.payload_u32_word_count()
            + self.visbuffer64.payload_u32_word_count()
    }

    pub fn byte_count(&self) -> usize {
        self.u32_word_count() * U32_WORD_BYTE_COUNT
    }

    pub fn payload_byte_count(&self) -> usize {
        self.payload_u32_word_count() * U32_WORD_BYTE_COUNT
    }

    pub fn footprint(&self) -> RenderVirtualGeometryDebugSnapshotReadbackStreamFootprint {
        let node_and_cluster_cull_u32_word_count = self.node_and_cluster_cull.u32_word_count();
        let render_path_u32_word_count = self.render_path.u32_word_count();
        let visbuffer64_u32_word_count = self.visbuffer64.u32_word_count();
        let total_u32_word_count = node_and_cluster_cull_u32_word_count
            + render_path_u32_word_count
            + visbuffer64_u32_word_count;

        RenderVirtualGeometryDebugSnapshotReadbackStreamFootprint {
            node_and_cluster_cull_u32_word_count,
            render_path_u32_word_count,
            visbuffer64_u32_word_count,
            total_u32_word_count,
            total_byte_count: total_u32_word_count * U32_WORD_BYTE_COUNT,
        }
    }
}

impl RenderVirtualGeometryDebugSnapshotReadbackStreamFootprint {
    pub fn payload_u32_word_count(&self) -> usize {
        self.node_and_cluster_cull_u32_word_count
            + self.render_path_u32_word_count
            + self
                .visbuffer64_u32_word_count
                .saturating_sub(U64_U32_WORD_COUNT)
    }

    pub fn payload_byte_count(&self) -> usize {
        self.payload_u32_word_count() * U32_WORD_BYTE_COUNT
    }

    pub fn section_u32_word_count(
        &self,
        section: RenderVirtualGeometryDebugSnapshotReadbackStreamSection,
    ) -> usize {
        match section {
            RenderVirtualGeometryDebugSnapshotReadbackStreamSection::NodeAndClusterCull => {
                self.node_and_cluster_cull_u32_word_count
            }
            RenderVirtualGeometryDebugSnapshotReadbackStreamSection::RenderPath => {
                self.render_path_u32_word_count
            }
            RenderVirtualGeometryDebugSnapshotReadbackStreamSection::VisBuffer64 => {
                self.visbuffer64_u32_word_count
            }
        }
    }

    pub fn section_byte_count(
        &self,
        section: RenderVirtualGeometryDebugSnapshotReadbackStreamSection,
    ) -> usize {
        self.section_u32_word_count(section) * U32_WORD_BYTE_COUNT
    }

    pub fn section_payload_u32_word_count(
        &self,
        section: RenderVirtualGeometryDebugSnapshotReadbackStreamSection,
    ) -> usize {
        match section {
            RenderVirtualGeometryDebugSnapshotReadbackStreamSection::NodeAndClusterCull
            | RenderVirtualGeometryDebugSnapshotReadbackStreamSection::RenderPath => {
                self.section_u32_word_count(section)
            }
            RenderVirtualGeometryDebugSnapshotReadbackStreamSection::VisBuffer64 => self
                .visbuffer64_u32_word_count
                .saturating_sub(U64_U32_WORD_COUNT),
        }
    }

    pub fn section_payload_byte_count(
        &self,
        section: RenderVirtualGeometryDebugSnapshotReadbackStreamSection,
    ) -> usize {
        self.section_payload_u32_word_count(section) * U32_WORD_BYTE_COUNT
    }
}

impl RenderVirtualGeometryDebugSnapshotReadbackStreamReport {
    pub fn is_decodable(&self) -> bool {
        self.decode_error.is_none()
    }

    pub fn has_decoded_payload(&self) -> bool {
        self.summary.as_ref().map_or(
            false,
            RenderVirtualGeometryDebugSnapshotReadbackStreamSummary::has_payload,
        )
    }

    pub fn payload_u32_word_count(&self) -> usize {
        self.footprint.payload_u32_word_count()
    }

    pub fn payload_byte_count(&self) -> usize {
        self.footprint.payload_byte_count()
    }

    pub fn decode_error_section_u32_word_count(&self) -> Option<usize> {
        self.decode_error_section()
            .map(|section| self.footprint.section_u32_word_count(section))
    }

    pub fn decode_error_section_byte_count(&self) -> Option<usize> {
        self.decode_error_section()
            .map(|section| self.footprint.section_byte_count(section))
    }
}

impl RenderVirtualGeometryDebugSnapshotReadbackStreamSummary {
    pub fn has_payload(&self) -> bool {
        self.node_and_cluster_cull_global_state_present
            || self.node_and_cluster_cull_dispatch_setup_present
            || self.node_and_cluster_cull_launch_worklist_present
            || self.node_and_cluster_cull_instance_seed_count != 0
            || self.node_and_cluster_cull_instance_work_item_count != 0
            || self.node_and_cluster_cull_cluster_work_item_count != 0
            || self.node_and_cluster_cull_child_work_item_count != 0
            || self.node_and_cluster_cull_traversal_record_count != 0
            || self.node_and_cluster_cull_hierarchy_child_id_count != 0
            || self.node_and_cluster_cull_page_request_id_count != 0
            || self.selected_cluster_count != 0
            || self.hardware_rasterization_record_count != 0
            || self.visbuffer64_entry_count != 0
    }

    pub fn is_empty(&self) -> bool {
        !self.has_payload()
    }
}
