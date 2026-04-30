use super::virtual_geometry_debug_snapshot::{
    RenderVirtualGeometryDebugSnapshot, RenderVirtualGeometryExecutionState,
    RenderVirtualGeometryHardwareRasterizationRecord,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullChildWorkItem,
    RenderVirtualGeometryNodeAndClusterCullClusterWorkItem,
    RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
    RenderVirtualGeometryNodeAndClusterCullSource,
    RenderVirtualGeometryNodeAndClusterCullTraversalRecord, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64Entry,
    RenderVirtualGeometryVisBuffer64Source,
};

const U32_WORD_BYTE_COUNT: usize = core::mem::size_of::<u32>();
const U64_U32_WORD_COUNT: usize = core::mem::size_of::<u64>() / U32_WORD_BYTE_COUNT;

mod diagnostics;
mod metrics;

pub use diagnostics::{
    RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeDiagnostic,
    RenderVirtualGeometryDebugSnapshotReadbackStreamSection,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryNodeAndClusterCullWordStreams {
    pub source: RenderVirtualGeometryNodeAndClusterCullSource,
    pub global_state: Option<Vec<u32>>,
    pub dispatch_setup: Option<Vec<u32>>,
    pub launch_worklist: Option<Vec<u32>>,
    pub instance_seeds: Vec<u32>,
    pub instance_work_items: Vec<u32>,
    pub cluster_work_items: Vec<u32>,
    pub child_work_items: Vec<u32>,
    pub traversal_records: Vec<u32>,
    pub hierarchy_child_ids: Vec<u32>,
    pub page_request_ids: Vec<u32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderVirtualGeometryNodeAndClusterCullDecodedStreams {
    pub source: RenderVirtualGeometryNodeAndClusterCullSource,
    pub global_state: Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
    pub dispatch_setup: Option<RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot>,
    pub launch_worklist: Option<RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot>,
    pub instance_seeds: Vec<RenderVirtualGeometryNodeAndClusterCullInstanceSeed>,
    pub instance_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem>,
    pub cluster_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullClusterWorkItem>,
    pub child_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullChildWorkItem>,
    pub traversal_records: Vec<RenderVirtualGeometryNodeAndClusterCullTraversalRecord>,
    pub hierarchy_child_ids: Vec<u32>,
    pub page_request_ids: Vec<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryRenderPathWordStreams {
    pub selected_clusters_source: RenderVirtualGeometrySelectedClusterSource,
    pub hardware_rasterization_source: RenderVirtualGeometryHardwareRasterizationSource,
    pub selected_clusters: Vec<u32>,
    pub hardware_rasterization_records: Vec<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryRenderPathDecodedStreams {
    pub selected_clusters_source: RenderVirtualGeometrySelectedClusterSource,
    pub hardware_rasterization_source: RenderVirtualGeometryHardwareRasterizationSource,
    pub selected_clusters: Vec<RenderVirtualGeometrySelectedCluster>,
    pub hardware_rasterization_records: Vec<RenderVirtualGeometryHardwareRasterizationRecord>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryVisBuffer64ReadbackStream {
    pub source: RenderVirtualGeometryVisBuffer64Source,
    pub clear_value: u64,
    pub entry_indices: Vec<u32>,
    pub packed_values: Vec<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryVisBuffer64DecodedStream {
    pub source: RenderVirtualGeometryVisBuffer64Source,
    pub clear_value: u64,
    pub entries: Vec<RenderVirtualGeometryVisBuffer64Entry>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryDebugSnapshotReadbackStreams {
    pub node_and_cluster_cull: RenderVirtualGeometryNodeAndClusterCullWordStreams,
    pub render_path: RenderVirtualGeometryRenderPathWordStreams,
    pub visbuffer64: RenderVirtualGeometryVisBuffer64ReadbackStream,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryDebugSnapshotReadbackStreamFootprint {
    pub node_and_cluster_cull_u32_word_count: usize,
    pub render_path_u32_word_count: usize,
    pub visbuffer64_u32_word_count: usize,
    pub total_u32_word_count: usize,
    pub total_byte_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryDebugSnapshotReadbackStreamReport {
    pub footprint: RenderVirtualGeometryDebugSnapshotReadbackStreamFootprint,
    pub summary: Option<RenderVirtualGeometryDebugSnapshotReadbackStreamSummary>,
    pub decode_error: Option<RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderVirtualGeometryDebugSnapshotDecodedStreams {
    pub node_and_cluster_cull: RenderVirtualGeometryNodeAndClusterCullDecodedStreams,
    pub render_path: RenderVirtualGeometryRenderPathDecodedStreams,
    pub visbuffer64: RenderVirtualGeometryVisBuffer64DecodedStream,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderVirtualGeometryNodeAndClusterCullWordStreamDecodeError {
    GlobalState { word_count: usize },
    DispatchSetup { word_count: usize },
    LaunchWorklist { word_count: usize },
    InstanceSeeds { word_count: usize },
    InstanceWorkItems { word_count: usize },
    ClusterWorkItems { word_count: usize },
    ChildWorkItems { word_count: usize },
    TraversalRecords { word_count: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderVirtualGeometryRenderPathWordStreamDecodeError {
    SelectedClusters { word_count: usize },
    HardwareRasterizationRecords { word_count: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderVirtualGeometryVisBuffer64ReadbackStreamDecodeError {
    MismatchedEntryAndValueCount {
        entry_index_count: usize,
        packed_value_count: usize,
    },
    InvalidPackedState {
        entry_index: u32,
        packed_value: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError {
    NodeAndClusterCull(RenderVirtualGeometryNodeAndClusterCullWordStreamDecodeError),
    RenderPath(RenderVirtualGeometryRenderPathWordStreamDecodeError),
    VisBuffer64(RenderVirtualGeometryVisBuffer64ReadbackStreamDecodeError),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryDebugSnapshotReadbackStreamSummary {
    pub node_and_cluster_cull_source: RenderVirtualGeometryNodeAndClusterCullSource,
    pub node_and_cluster_cull_global_state_present: bool,
    pub node_and_cluster_cull_dispatch_setup_present: bool,
    pub node_and_cluster_cull_launch_worklist_present: bool,
    pub node_and_cluster_cull_instance_seed_count: usize,
    pub node_and_cluster_cull_instance_work_item_count: usize,
    pub node_and_cluster_cull_cluster_work_item_count: usize,
    pub node_and_cluster_cull_child_work_item_count: usize,
    pub node_and_cluster_cull_traversal_record_count: usize,
    pub node_and_cluster_cull_hierarchy_child_id_count: usize,
    pub node_and_cluster_cull_page_request_id_count: usize,
    pub selected_clusters_source: RenderVirtualGeometrySelectedClusterSource,
    pub selected_cluster_count: usize,
    pub hardware_rasterization_source: RenderVirtualGeometryHardwareRasterizationSource,
    pub hardware_rasterization_record_count: usize,
    pub visbuffer64_source: RenderVirtualGeometryVisBuffer64Source,
    pub visbuffer64_clear_value: u64,
    pub visbuffer64_entry_count: usize,
}

impl RenderVirtualGeometryDebugSnapshot {
    pub fn debug_readback_streams(&self) -> RenderVirtualGeometryDebugSnapshotReadbackStreams {
        RenderVirtualGeometryDebugSnapshotReadbackStreams {
            node_and_cluster_cull: self.node_and_cluster_cull_word_streams(),
            render_path: self.render_path_word_streams(),
            visbuffer64: self.visbuffer64_readback_stream(),
        }
    }

    pub fn debug_decoded_streams(
        &self,
    ) -> Option<RenderVirtualGeometryDebugSnapshotDecodedStreams> {
        Self::decode_debug_readback_streams(&self.debug_readback_streams())
    }

    pub fn try_debug_decoded_streams(
        &self,
    ) -> Result<
        RenderVirtualGeometryDebugSnapshotDecodedStreams,
        RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError,
    > {
        Self::try_decode_debug_readback_streams(&self.debug_readback_streams())
    }

    pub fn decode_debug_readback_streams(
        streams: &RenderVirtualGeometryDebugSnapshotReadbackStreams,
    ) -> Option<RenderVirtualGeometryDebugSnapshotDecodedStreams> {
        Self::try_decode_debug_readback_streams(streams).ok()
    }

    pub fn try_decode_debug_readback_streams(
        streams: &RenderVirtualGeometryDebugSnapshotReadbackStreams,
    ) -> Result<
        RenderVirtualGeometryDebugSnapshotDecodedStreams,
        RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError,
    > {
        Ok(RenderVirtualGeometryDebugSnapshotDecodedStreams {
            node_and_cluster_cull: Self::try_decode_node_and_cluster_cull_word_streams(
                &streams.node_and_cluster_cull,
            )
            .map_err(
                RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError::NodeAndClusterCull,
            )?,
            render_path: Self::try_decode_render_path_word_streams(&streams.render_path)
                .map_err(RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError::RenderPath)?,
            visbuffer64: Self::try_decode_visbuffer64_readback_stream(&streams.visbuffer64)
                .map_err(
                    RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError::VisBuffer64,
                )?,
        })
    }

    pub fn debug_readback_stream_summary(
        &self,
    ) -> Option<RenderVirtualGeometryDebugSnapshotReadbackStreamSummary> {
        Self::summarize_debug_readback_streams(&self.debug_readback_streams())
    }

    pub fn debug_readback_stream_footprint(
        &self,
    ) -> RenderVirtualGeometryDebugSnapshotReadbackStreamFootprint {
        Self::summarize_debug_readback_stream_footprint(&self.debug_readback_streams())
    }

    pub fn debug_readback_stream_report(
        &self,
    ) -> RenderVirtualGeometryDebugSnapshotReadbackStreamReport {
        Self::report_debug_readback_streams(&self.debug_readback_streams())
    }

    pub fn summarize_debug_readback_stream_footprint(
        streams: &RenderVirtualGeometryDebugSnapshotReadbackStreams,
    ) -> RenderVirtualGeometryDebugSnapshotReadbackStreamFootprint {
        streams.footprint()
    }

    pub fn report_debug_readback_streams(
        streams: &RenderVirtualGeometryDebugSnapshotReadbackStreams,
    ) -> RenderVirtualGeometryDebugSnapshotReadbackStreamReport {
        let footprint = Self::summarize_debug_readback_stream_footprint(streams);
        match Self::try_summarize_debug_readback_streams(streams) {
            Ok(summary) => RenderVirtualGeometryDebugSnapshotReadbackStreamReport {
                footprint,
                summary: Some(summary),
                decode_error: None,
            },
            Err(error) => RenderVirtualGeometryDebugSnapshotReadbackStreamReport {
                footprint,
                summary: None,
                decode_error: Some(error),
            },
        }
    }

    pub fn try_debug_readback_stream_summary(
        &self,
    ) -> Result<
        RenderVirtualGeometryDebugSnapshotReadbackStreamSummary,
        RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError,
    > {
        Self::try_summarize_debug_readback_streams(&self.debug_readback_streams())
    }

    pub fn summarize_debug_readback_streams(
        streams: &RenderVirtualGeometryDebugSnapshotReadbackStreams,
    ) -> Option<RenderVirtualGeometryDebugSnapshotReadbackStreamSummary> {
        Self::try_summarize_debug_readback_streams(streams).ok()
    }

    pub fn try_summarize_debug_readback_streams(
        streams: &RenderVirtualGeometryDebugSnapshotReadbackStreams,
    ) -> Result<
        RenderVirtualGeometryDebugSnapshotReadbackStreamSummary,
        RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError,
    > {
        let decoded = Self::try_decode_debug_readback_streams(streams)?;

        Ok(RenderVirtualGeometryDebugSnapshotReadbackStreamSummary {
            node_and_cluster_cull_source: decoded.node_and_cluster_cull.source,
            node_and_cluster_cull_global_state_present: decoded
                .node_and_cluster_cull
                .global_state
                .is_some(),
            node_and_cluster_cull_dispatch_setup_present: decoded
                .node_and_cluster_cull
                .dispatch_setup
                .is_some(),
            node_and_cluster_cull_launch_worklist_present: decoded
                .node_and_cluster_cull
                .launch_worklist
                .is_some(),
            node_and_cluster_cull_instance_seed_count: decoded
                .node_and_cluster_cull
                .instance_seeds
                .len(),
            node_and_cluster_cull_instance_work_item_count: decoded
                .node_and_cluster_cull
                .instance_work_items
                .len(),
            node_and_cluster_cull_cluster_work_item_count: decoded
                .node_and_cluster_cull
                .cluster_work_items
                .len(),
            node_and_cluster_cull_child_work_item_count: decoded
                .node_and_cluster_cull
                .child_work_items
                .len(),
            node_and_cluster_cull_traversal_record_count: decoded
                .node_and_cluster_cull
                .traversal_records
                .len(),
            node_and_cluster_cull_hierarchy_child_id_count: decoded
                .node_and_cluster_cull
                .hierarchy_child_ids
                .len(),
            node_and_cluster_cull_page_request_id_count: decoded
                .node_and_cluster_cull
                .page_request_ids
                .len(),
            selected_clusters_source: decoded.render_path.selected_clusters_source,
            selected_cluster_count: decoded.render_path.selected_clusters.len(),
            hardware_rasterization_source: decoded.render_path.hardware_rasterization_source,
            hardware_rasterization_record_count: decoded
                .render_path
                .hardware_rasterization_records
                .len(),
            visbuffer64_source: decoded.visbuffer64.source,
            visbuffer64_clear_value: decoded.visbuffer64.clear_value,
            visbuffer64_entry_count: decoded.visbuffer64.entries.len(),
        })
    }

    pub fn node_and_cluster_cull_word_streams(
        &self,
    ) -> RenderVirtualGeometryNodeAndClusterCullWordStreams {
        RenderVirtualGeometryNodeAndClusterCullWordStreams {
            source: self.node_and_cluster_cull_source,
            global_state: self.node_and_cluster_cull_global_state_words(),
            dispatch_setup: self.node_and_cluster_cull_dispatch_setup_words(),
            launch_worklist: self.node_and_cluster_cull_launch_worklist_words(),
            instance_seeds: self.node_and_cluster_cull_instance_seed_words(),
            instance_work_items: self.node_and_cluster_cull_instance_work_item_words(),
            cluster_work_items: self.node_and_cluster_cull_cluster_work_item_words(),
            child_work_items: self.node_and_cluster_cull_child_work_item_words(),
            traversal_records: self.node_and_cluster_cull_traversal_record_words(),
            hierarchy_child_ids: self.node_and_cluster_cull_hierarchy_child_id_words(),
            page_request_ids: self.node_and_cluster_cull_page_request_id_words(),
        }
    }

    pub fn node_and_cluster_cull_decoded_streams(
        &self,
    ) -> Option<RenderVirtualGeometryNodeAndClusterCullDecodedStreams> {
        Self::decode_node_and_cluster_cull_word_streams(&self.node_and_cluster_cull_word_streams())
    }

    pub fn decode_node_and_cluster_cull_word_streams(
        streams: &RenderVirtualGeometryNodeAndClusterCullWordStreams,
    ) -> Option<RenderVirtualGeometryNodeAndClusterCullDecodedStreams> {
        Self::try_decode_node_and_cluster_cull_word_streams(streams).ok()
    }

    pub fn try_decode_node_and_cluster_cull_word_streams(
        streams: &RenderVirtualGeometryNodeAndClusterCullWordStreams,
    ) -> Result<
        RenderVirtualGeometryNodeAndClusterCullDecodedStreams,
        RenderVirtualGeometryNodeAndClusterCullWordStreamDecodeError,
    > {
        let global_state = match streams.global_state.as_deref() {
            Some(words) => Some(
                Self::decode_node_and_cluster_cull_global_state_words(words).ok_or(
                    RenderVirtualGeometryNodeAndClusterCullWordStreamDecodeError::GlobalState {
                        word_count: words.len(),
                    },
                )?,
            ),
            None => None,
        };
        let dispatch_setup = match streams.dispatch_setup.as_deref() {
            Some(words) => Some(
                Self::decode_node_and_cluster_cull_dispatch_setup_words(words).ok_or(
                    RenderVirtualGeometryNodeAndClusterCullWordStreamDecodeError::DispatchSetup {
                        word_count: words.len(),
                    },
                )?,
            ),
            None => None,
        };
        let launch_worklist = match streams.launch_worklist.as_deref() {
            Some(words) => Some(
                Self::decode_node_and_cluster_cull_launch_worklist_words(words).ok_or(
                    RenderVirtualGeometryNodeAndClusterCullWordStreamDecodeError::LaunchWorklist {
                        word_count: words.len(),
                    },
                )?,
            ),
            None => None,
        };

        Ok(RenderVirtualGeometryNodeAndClusterCullDecodedStreams {
            source: streams.source,
            global_state,
            dispatch_setup,
            launch_worklist,
            instance_seeds: Self::decode_node_and_cluster_cull_instance_seed_words(
                &streams.instance_seeds,
            )
            .ok_or(
                RenderVirtualGeometryNodeAndClusterCullWordStreamDecodeError::InstanceSeeds {
                    word_count: streams.instance_seeds.len(),
                },
            )?,
            instance_work_items: Self::decode_node_and_cluster_cull_instance_work_item_words(
                &streams.instance_work_items,
            )
            .ok_or(
                RenderVirtualGeometryNodeAndClusterCullWordStreamDecodeError::InstanceWorkItems {
                    word_count: streams.instance_work_items.len(),
                },
            )?,
            cluster_work_items: Self::decode_node_and_cluster_cull_cluster_work_item_words(
                &streams.cluster_work_items,
            )
            .ok_or(
                RenderVirtualGeometryNodeAndClusterCullWordStreamDecodeError::ClusterWorkItems {
                    word_count: streams.cluster_work_items.len(),
                },
            )?,
            child_work_items: Self::decode_node_and_cluster_cull_child_work_item_words(
                &streams.child_work_items,
            )
            .ok_or(
                RenderVirtualGeometryNodeAndClusterCullWordStreamDecodeError::ChildWorkItems {
                    word_count: streams.child_work_items.len(),
                },
            )?,
            traversal_records: Self::decode_node_and_cluster_cull_traversal_record_words(
                &streams.traversal_records,
            )
            .ok_or(
                RenderVirtualGeometryNodeAndClusterCullWordStreamDecodeError::TraversalRecords {
                    word_count: streams.traversal_records.len(),
                },
            )?,
            hierarchy_child_ids: Self::decode_node_and_cluster_cull_hierarchy_child_id_words(
                &streams.hierarchy_child_ids,
            ),
            page_request_ids: Self::decode_node_and_cluster_cull_page_request_id_words(
                &streams.page_request_ids,
            ),
        })
    }

    pub fn render_path_word_streams(&self) -> RenderVirtualGeometryRenderPathWordStreams {
        RenderVirtualGeometryRenderPathWordStreams {
            selected_clusters_source: self.selected_clusters_source,
            hardware_rasterization_source: self.hardware_rasterization_source,
            selected_clusters: self.selected_cluster_words(),
            hardware_rasterization_records: self.hardware_rasterization_record_words(),
        }
    }

    pub fn render_path_decoded_streams(
        &self,
    ) -> Option<RenderVirtualGeometryRenderPathDecodedStreams> {
        Self::decode_render_path_word_streams(&self.render_path_word_streams())
    }

    pub fn decode_render_path_word_streams(
        streams: &RenderVirtualGeometryRenderPathWordStreams,
    ) -> Option<RenderVirtualGeometryRenderPathDecodedStreams> {
        Self::try_decode_render_path_word_streams(streams).ok()
    }

    pub fn try_decode_render_path_word_streams(
        streams: &RenderVirtualGeometryRenderPathWordStreams,
    ) -> Result<
        RenderVirtualGeometryRenderPathDecodedStreams,
        RenderVirtualGeometryRenderPathWordStreamDecodeError,
    > {
        Ok(RenderVirtualGeometryRenderPathDecodedStreams {
            selected_clusters_source: streams.selected_clusters_source,
            hardware_rasterization_source: streams.hardware_rasterization_source,
            selected_clusters: Self::decode_selected_cluster_words(&streams.selected_clusters)
                .ok_or(RenderVirtualGeometryRenderPathWordStreamDecodeError::SelectedClusters {
                    word_count: streams.selected_clusters.len(),
                })?,
            hardware_rasterization_records: Self::decode_hardware_rasterization_record_words(
                &streams.hardware_rasterization_records,
            )
            .ok_or(
                RenderVirtualGeometryRenderPathWordStreamDecodeError::HardwareRasterizationRecords {
                    word_count: streams.hardware_rasterization_records.len(),
                },
            )?,
        })
    }

    pub fn selected_cluster_words(&self) -> Vec<u32> {
        packed_word_stream(
            &self.selected_clusters,
            RenderVirtualGeometrySelectedCluster::packed_words,
        )
    }

    pub fn hardware_rasterization_record_words(&self) -> Vec<u32> {
        packed_word_stream(
            &self.hardware_rasterization_records,
            RenderVirtualGeometryHardwareRasterizationRecord::packed_words,
        )
    }

    pub fn decode_selected_cluster_words(
        words: &[u32],
    ) -> Option<Vec<RenderVirtualGeometrySelectedCluster>> {
        decode_packed_word_stream::<
            RenderVirtualGeometrySelectedCluster,
            { RenderVirtualGeometrySelectedCluster::GPU_WORD_COUNT },
        >(
            words,
            RenderVirtualGeometrySelectedCluster::from_packed_words,
        )
    }

    pub fn decode_hardware_rasterization_record_words(
        words: &[u32],
    ) -> Option<Vec<RenderVirtualGeometryHardwareRasterizationRecord>> {
        decode_packed_word_stream::<
            RenderVirtualGeometryHardwareRasterizationRecord,
            { RenderVirtualGeometryHardwareRasterizationRecord::GPU_WORD_COUNT },
        >(
            words,
            RenderVirtualGeometryHardwareRasterizationRecord::from_packed_words,
        )
    }

    pub fn visbuffer64_readback_stream(&self) -> RenderVirtualGeometryVisBuffer64ReadbackStream {
        RenderVirtualGeometryVisBuffer64ReadbackStream {
            source: self.visbuffer64_source,
            clear_value: self.visbuffer64_clear_value,
            entry_indices: self
                .visbuffer64_entries
                .iter()
                .map(|entry| entry.entry_index)
                .collect(),
            packed_values: self
                .visbuffer64_entries
                .iter()
                .map(|entry| entry.packed_value)
                .collect(),
        }
    }

    pub fn visbuffer64_decoded_stream(
        &self,
    ) -> Option<RenderVirtualGeometryVisBuffer64DecodedStream> {
        Self::decode_visbuffer64_readback_stream(&self.visbuffer64_readback_stream())
    }

    pub fn decode_visbuffer64_readback_stream(
        stream: &RenderVirtualGeometryVisBuffer64ReadbackStream,
    ) -> Option<RenderVirtualGeometryVisBuffer64DecodedStream> {
        Self::try_decode_visbuffer64_readback_stream(stream).ok()
    }

    pub fn try_decode_visbuffer64_readback_stream(
        stream: &RenderVirtualGeometryVisBuffer64ReadbackStream,
    ) -> Result<
        RenderVirtualGeometryVisBuffer64DecodedStream,
        RenderVirtualGeometryVisBuffer64ReadbackStreamDecodeError,
    > {
        if stream.entry_indices.len() != stream.packed_values.len() {
            return Err(
                RenderVirtualGeometryVisBuffer64ReadbackStreamDecodeError::MismatchedEntryAndValueCount {
                    entry_index_count: stream.entry_indices.len(),
                    packed_value_count: stream.packed_values.len(),
                },
            );
        }

        let entries = stream
            .entry_indices
            .iter()
            .copied()
            .zip(stream.packed_values.iter().copied())
            .map(|(entry_index, packed_value)| decode_visbuffer64_entry(entry_index, packed_value))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(RenderVirtualGeometryVisBuffer64DecodedStream {
            source: stream.source,
            clear_value: stream.clear_value,
            entries,
        })
    }

    pub fn node_and_cluster_cull_instance_seed_words(&self) -> Vec<u32> {
        packed_word_stream(
            &self.node_and_cluster_cull_instance_seeds,
            RenderVirtualGeometryNodeAndClusterCullInstanceSeed::packed_words,
        )
    }

    pub fn node_and_cluster_cull_instance_work_item_words(&self) -> Vec<u32> {
        packed_word_stream(
            &self.node_and_cluster_cull_instance_work_items,
            RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem::packed_words,
        )
    }

    pub fn node_and_cluster_cull_cluster_work_item_words(&self) -> Vec<u32> {
        packed_word_stream(
            &self.node_and_cluster_cull_cluster_work_items,
            RenderVirtualGeometryNodeAndClusterCullClusterWorkItem::packed_words,
        )
    }

    pub fn node_and_cluster_cull_child_work_item_words(&self) -> Vec<u32> {
        packed_word_stream(
            &self.node_and_cluster_cull_child_work_items,
            RenderVirtualGeometryNodeAndClusterCullChildWorkItem::packed_words,
        )
    }

    pub fn node_and_cluster_cull_traversal_record_words(&self) -> Vec<u32> {
        packed_word_stream(
            &self.node_and_cluster_cull_traversal_records,
            RenderVirtualGeometryNodeAndClusterCullTraversalRecord::packed_words,
        )
    }

    pub fn node_and_cluster_cull_hierarchy_child_id_words(&self) -> Vec<u32> {
        self.node_and_cluster_cull_hierarchy_child_ids.clone()
    }

    pub fn node_and_cluster_cull_page_request_id_words(&self) -> Vec<u32> {
        self.node_and_cluster_cull_page_request_ids.clone()
    }

    pub fn node_and_cluster_cull_global_state_words(&self) -> Option<Vec<u32>> {
        self.node_and_cluster_cull_global_state
            .as_ref()
            .map(|global_state| global_state.packed_words().to_vec())
    }

    pub fn node_and_cluster_cull_dispatch_setup_words(&self) -> Option<Vec<u32>> {
        self.node_and_cluster_cull_dispatch_setup
            .as_ref()
            .map(|dispatch_setup| dispatch_setup.packed_words().to_vec())
    }

    pub fn node_and_cluster_cull_launch_worklist_words(&self) -> Option<Vec<u32>> {
        self.node_and_cluster_cull_launch_worklist
            .as_ref()
            .map(|worklist| worklist.packed_words())
    }

    pub fn decode_node_and_cluster_cull_instance_seed_words(
        words: &[u32],
    ) -> Option<Vec<RenderVirtualGeometryNodeAndClusterCullInstanceSeed>> {
        decode_packed_word_stream::<
            RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
            { RenderVirtualGeometryNodeAndClusterCullInstanceSeed::GPU_WORD_COUNT },
        >(
            words,
            RenderVirtualGeometryNodeAndClusterCullInstanceSeed::from_packed_words,
        )
    }

    pub fn decode_node_and_cluster_cull_instance_work_item_words(
        words: &[u32],
    ) -> Option<Vec<RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem>> {
        decode_packed_word_stream::<
            RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
            { RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem::GPU_WORD_COUNT },
        >(
            words,
            RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem::from_packed_words,
        )
    }

    pub fn decode_node_and_cluster_cull_cluster_work_item_words(
        words: &[u32],
    ) -> Option<Vec<RenderVirtualGeometryNodeAndClusterCullClusterWorkItem>> {
        decode_packed_word_stream::<
            RenderVirtualGeometryNodeAndClusterCullClusterWorkItem,
            { RenderVirtualGeometryNodeAndClusterCullClusterWorkItem::GPU_WORD_COUNT },
        >(
            words,
            RenderVirtualGeometryNodeAndClusterCullClusterWorkItem::from_packed_words,
        )
    }

    pub fn decode_node_and_cluster_cull_child_work_item_words(
        words: &[u32],
    ) -> Option<Vec<RenderVirtualGeometryNodeAndClusterCullChildWorkItem>> {
        decode_packed_word_stream::<
            RenderVirtualGeometryNodeAndClusterCullChildWorkItem,
            { RenderVirtualGeometryNodeAndClusterCullChildWorkItem::GPU_WORD_COUNT },
        >(
            words,
            RenderVirtualGeometryNodeAndClusterCullChildWorkItem::from_packed_words,
        )
    }

    pub fn decode_node_and_cluster_cull_traversal_record_words(
        words: &[u32],
    ) -> Option<Vec<RenderVirtualGeometryNodeAndClusterCullTraversalRecord>> {
        decode_packed_word_stream::<
            RenderVirtualGeometryNodeAndClusterCullTraversalRecord,
            { RenderVirtualGeometryNodeAndClusterCullTraversalRecord::GPU_WORD_COUNT },
        >(
            words,
            RenderVirtualGeometryNodeAndClusterCullTraversalRecord::from_packed_words,
        )
    }

    pub fn decode_node_and_cluster_cull_hierarchy_child_id_words(words: &[u32]) -> Vec<u32> {
        words.to_vec()
    }

    pub fn decode_node_and_cluster_cull_page_request_id_words(words: &[u32]) -> Vec<u32> {
        words.to_vec()
    }

    pub fn decode_node_and_cluster_cull_global_state_words(
        words: &[u32],
    ) -> Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot> {
        decode_exact_packed_words::<
            RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
            { RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot::GPU_WORD_COUNT },
        >(
            words,
            RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot::from_packed_words,
        )
    }

    pub fn decode_node_and_cluster_cull_dispatch_setup_words(
        words: &[u32],
    ) -> Option<RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot> {
        decode_exact_packed_words::<
            RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
            { RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot::GPU_WORD_COUNT },
        >(
            words,
            RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot::from_packed_words,
        )
    }

    pub fn decode_node_and_cluster_cull_launch_worklist_words(
        words: &[u32],
    ) -> Option<RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot> {
        let worklist =
            RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot::from_packed_words(
                words,
            )?;
        (worklist.packed_words().len() == words.len()).then_some(worklist)
    }
}

fn packed_word_stream<T, const N: usize>(entries: &[T], pack: impl Fn(&T) -> [u32; N]) -> Vec<u32> {
    entries.iter().flat_map(pack).collect()
}

fn decode_packed_word_stream<T, const N: usize>(
    words: &[u32],
    decode: impl Fn(&[u32]) -> Option<T>,
) -> Option<Vec<T>> {
    if words.len() % N != 0 {
        return None;
    }

    words.chunks_exact(N).map(decode).collect()
}

fn decode_exact_packed_words<T, const N: usize>(
    words: &[u32],
    decode: impl Fn(&[u32]) -> Option<T>,
) -> Option<T> {
    if words.len() != N {
        return None;
    }

    decode(words)
}

fn decode_visbuffer64_entry(
    entry_index: u32,
    packed_value: u64,
) -> Result<
    RenderVirtualGeometryVisBuffer64Entry,
    RenderVirtualGeometryVisBuffer64ReadbackStreamDecodeError,
> {
    const CLUSTER_BITS: u64 = 20;
    const PAGE_BITS: u64 = 20;
    const INSTANCE_BITS: u64 = 16;
    const LOD_BITS: u64 = 6;
    const CLUSTER_MASK: u64 = (1_u64 << CLUSTER_BITS) - 1;
    const PAGE_MASK: u64 = (1_u64 << PAGE_BITS) - 1;
    const INSTANCE_MASK: u64 = (1_u64 << INSTANCE_BITS) - 1;
    const LOD_MASK: u64 = (1_u64 << LOD_BITS) - 1;
    const PAGE_SHIFT: u64 = CLUSTER_BITS;
    const INSTANCE_SHIFT: u64 = PAGE_SHIFT + PAGE_BITS;
    const LOD_SHIFT: u64 = INSTANCE_SHIFT + INSTANCE_BITS;
    const STATE_SHIFT: u64 = LOD_SHIFT + LOD_BITS;
    const INSTANCE_NONE_SENTINEL: u32 = u16::MAX as u32;

    let state = match (packed_value >> STATE_SHIFT) & 0b11 {
        0 => RenderVirtualGeometryExecutionState::Resident,
        1 => RenderVirtualGeometryExecutionState::PendingUpload,
        2 => RenderVirtualGeometryExecutionState::Missing,
        _ => {
            return Err(
                RenderVirtualGeometryVisBuffer64ReadbackStreamDecodeError::InvalidPackedState {
                    entry_index,
                    packed_value,
                },
            )
        }
    };
    let instance_word = ((packed_value >> INSTANCE_SHIFT) & INSTANCE_MASK) as u32;

    Ok(RenderVirtualGeometryVisBuffer64Entry {
        entry_index,
        packed_value,
        instance_index: (instance_word != INSTANCE_NONE_SENTINEL).then_some(instance_word),
        entity: 0,
        cluster_id: (packed_value & CLUSTER_MASK) as u32,
        page_id: ((packed_value >> PAGE_SHIFT) & PAGE_MASK) as u32,
        lod_level: ((packed_value >> LOD_SHIFT) & LOD_MASK) as u8,
        state,
    })
}
