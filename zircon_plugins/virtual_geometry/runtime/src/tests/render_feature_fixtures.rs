use std::{
    ops::{Deref, DerefMut},
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use zircon_runtime::asset::pipeline::manager::{
    AssetManager, ProjectAssetManager, ProjectAssetManagerAccess,
};
use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::render as native;
use zircon_runtime::core::framework::render::RenderVirtualGeometryDebugSnapshot;
use zircon_runtime::core::framework::render::{
    RenderFramework, RenderFrameworkError, RenderQualityProfile, RenderViewportDescriptor,
};
use zircon_runtime::core::manager::{manager_service_handle, RegisteredManagerService};
use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::core::{
    CoreRuntime, ManagerDescriptor, ModuleDescriptor, RegistryName, ServiceKind, StartupMode,
};
use zircon_runtime::graphics::{
    CompiledRenderPipeline, RenderFeatureDescriptor, WgpuRenderFramework,
};

pub(crate) use crate::render_pass_executors::IndirectReadbackWords;
use crate::render_pass_executors::{IndirectGpuReadback, NodeClusterCullReadback};
use crate::virtual_geometry::{
    uploader_test_collector_registration, VirtualGeometryUploaderTestCollector,
};

static NEXT_UPLOADER_PROJECT: AtomicU64 = AtomicU64::new(0);

const TEST_ASSET_MODULE_NAME: &str = "VirtualGeometryTestAssetRuntime";
const TEST_ASSET_SERVICE_NAME: &str = "VirtualGeometryTestAssetRuntime.Manager.ProjectAssetManager";

pub(crate) struct PluginizedWgpuRenderFrameworkFixture {
    framework: WgpuRenderFramework,
    latest_viewport: Option<native::RenderViewportHandle>,
    _asset_runtime: CoreRuntime,
    uploader_collector: Option<Arc<VirtualGeometryUploaderTestCollector>>,
    visible_cluster_readback: NodeClusterCullReadback,
    latest_visible_cluster_words: Option<Result<Vec<u32>, String>>,
    latest_cull_gpu_outputs:
        Option<Result<crate::render_pass_executors::CullGpuReadbackWords, String>>,
    indirect_gpu_readback: IndirectGpuReadback,
    latest_indirect_gpu_words: Option<Result<IndirectReadbackWords, String>>,
    latest_native_probe: Option<(
        zircon_runtime::core::framework::render::RenderViewportHandle,
        u64,
    )>,
    // No CPU submission projections are manufactured by this production fixture.
    legacy_cpu_submission_projections: [Option<Vec<u32>>; 2],
}

impl PluginizedWgpuRenderFrameworkFixture {
    fn native_probe_origin(
        &self,
    ) -> Result<
        (
            zircon_runtime::core::framework::render::RenderViewportHandle,
            u64,
        ),
        String,
    > {
        self.latest_native_probe
            .ok_or_else(|| "no native probe completed for this frame".into())
    }
    fn drop_native_probe_source(&self, role: u32) {
        let (v, s) = self
            .native_probe_origin()
            .expect("actual native probe origin");
        self.framework
            .test_support_virtual_geometry_native_probe_source(v, s, role, true)
            .expect("drop actual native lease");
    }
    fn has_native_probe_source(&self, role: u32) -> bool {
        let (v, s) = self
            .native_probe_origin()
            .expect("actual native probe origin");
        self.framework
            .test_support_virtual_geometry_native_probe_source(v, s, role, false)
            .expect("query actual native lease")
    }
    fn read_native_probe_rows(&self, kind: u32, stride: usize) -> Result<Vec<Vec<u32>>, String> {
        let (v, s) = self.native_probe_origin()?;
        let bytes = self
            .framework
            .test_support_read_virtual_geometry_native_probe(v, s, kind)?;
        if bytes.len() < 4 || bytes.len() % 4 != 0 {
            return Err("invalid native word payload".into());
        }
        let words = bytes
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect::<Vec<_>>();
        let count = usize::try_from(words[0]).map_err(|_| "native count overflow")?;
        let end = count
            .checked_mul(stride)
            .and_then(|n| n.checked_add(1))
            .ok_or("native row count overflow")?;
        if end > words.len() {
            return Err("GPU count exceeds mapped capacity".into());
        }
        Ok(words[1..end]
            .chunks_exact(stride)
            .map(|r| r.to_vec())
            .collect())
    }
    pub(crate) fn read_last_virtual_geometry_indirect_args(&self) -> Result<Vec<[u32; 4]>, String> {
        self.read_native_probe_rows(3, 4)?
            .into_iter()
            .map(|row| {
                row.try_into()
                    .map_err(|_| "invalid mapped args4 row".into())
            })
            .collect()
    }
    pub(crate) fn read_last_virtual_geometry_indirect_segments(
        &self,
    ) -> Result<
        Vec<(
            u32,
            u32,
            u32,
            u32,
            u32,
            crate::virtual_geometry::VirtualGeometryPrepareClusterState,
            u32,
            u32,
            u32,
        )>,
        String,
    > {
        use crate::virtual_geometry::VirtualGeometryPrepareClusterState as State;
        self.read_native_probe_rows(4, 12)?
            .into_iter()
            .map(|row| {
                let state = match row[8] {
                    0 => State::Resident,
                    1 => State::PendingUpload,
                    2 => State::Missing,
                    _ => return Err("invalid mapped author segment state".into()),
                };
                Ok((
                    row[3], row[4], row[5], row[6], row[7], state, row[9], row[10], row[11],
                ))
            })
            .collect()
    }
    pub(crate) fn read_last_virtual_geometry_mesh_draw_submission_records_with_tokens(
        &self,
    ) -> Result<Vec<(u64, u32, u32, u32)>, String> {
        Ok(self
            .read_native_probe_rows(0, 5)?
            .into_iter()
            .map(|r| (u64::from(r[0]) | (u64::from(r[1]) << 32), r[2], r[3], r[4]))
            .collect())
    }
    pub(crate) fn read_last_virtual_geometry_indirect_execution_draw_ref_indices(
        &self,
    ) -> Result<Vec<u32>, String> {
        Ok(self
            .read_native_probe_rows(1, 1)?
            .into_iter()
            .map(|r| r[0])
            .collect())
    }
    pub(crate) fn read_last_virtual_geometry_indirect_execution_records(
        &self,
    ) -> Result<Vec<(u32, u64, u32, u32, u32)>, String> {
        Ok(self
            .read_native_probe_rows(2, 6)?
            .into_iter()
            .map(|r| {
                (
                    r[0],
                    u64::from(r[1]) | (u64::from(r[2]) << 32),
                    r[3],
                    r[4],
                    r[5],
                )
            })
            .collect())
    }
    pub(crate) fn drop_last_virtual_geometry_mesh_draw_submission_records_for_test(&mut self) {
        drop(self.legacy_cpu_submission_projections[0].take());
    }
    pub(crate) fn drop_last_virtual_geometry_mesh_draw_submission_token_records_for_test(
        &mut self,
    ) {
        drop(self.legacy_cpu_submission_projections[1].take());
    }

    pub(crate) fn drop_last_virtual_geometry_indirect_args_buffer_for_test(&mut self) {
        self.drop_native_probe_source(0);
    }

    pub(crate) fn drop_last_virtual_geometry_indirect_authority_buffer_for_test(&mut self) {
        self.drop_native_probe_source(1);
    }

    pub(crate) fn drop_last_virtual_geometry_indirect_submission_buffer_for_test(&mut self) {
        self.drop_native_probe_source(2);
    }

    pub(crate) fn drop_last_virtual_geometry_indirect_draw_refs_buffer_for_test(&mut self) {
        self.drop_native_probe_source(3);
    }

    pub(crate) fn drop_last_virtual_geometry_indirect_segments_buffer_for_test(&mut self) {
        self.drop_native_probe_source(4);
    }

    pub(crate) fn drop_last_virtual_geometry_indirect_execution_args_buffer_for_test(&mut self) {
        self.drop_native_probe_source(5);
    }

    pub(crate) fn drop_last_virtual_geometry_indirect_execution_authority_buffer_for_test(
        &mut self,
    ) {
        self.drop_native_probe_source(6);
    }

    pub(crate) fn drop_last_virtual_geometry_indirect_execution_buffer_for_test(&mut self) {
        self.drop_native_probe_source(8);
    }

    pub(crate) fn drop_last_virtual_geometry_indirect_execution_records_buffer_for_test(&mut self) {
        self.drop_native_probe_source(9);
    }

    pub(crate) fn has_last_virtual_geometry_indirect_execution_buffer_for_test(&self) -> bool {
        self.has_native_probe_source(8)
    }

    pub(crate) fn has_last_virtual_geometry_indirect_execution_records_buffer_for_test(
        &self,
    ) -> bool {
        self.has_native_probe_source(9)
    }

    fn native_outputs(&self) -> Result<native::RenderVirtualGeometryReadbackOutputs, String> {
        self.latest_viewport
            .ok_or("no current virtual geometry frame has submitted")?;
        self.framework
            .test_support_last_virtual_geometry_readback_outputs()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "current virtual geometry terminal native map did not complete".into())
    }

    fn native_inspection(
        &self,
    ) -> Result<native::RenderVirtualGeometryNativeReadbackInspection, String> {
        self.latest_viewport
            .ok_or("no current virtual geometry frame has submitted")?;
        self.framework
            .test_support_last_virtual_geometry_native_inspection()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "current virtual geometry startup native map did not complete".into())
    }

    /// Uploader completion remains independent from the terminal render-path map.
    /// Both actual completions are required before attaching the mapped render-path bytes.
    pub(crate) fn take_last_virtual_geometry_gpu_readback(
        &self,
    ) -> Option<crate::virtual_geometry::VirtualGeometryGpuReadback> {
        let collector = self.uploader_collector.as_ref()?;
        let mut readback = collector
            .completed_readback()
            .expect("actual uploader page-table and completion maps must succeed")?;
        let outputs = self
            .native_outputs()
            .expect("actual terminal map must complete");
        let inspection = self
            .native_inspection()
            .expect("actual native provenance must complete");
        let hardware_count = u32::try_from(outputs.hardware_rasterization_records.len())
            .expect("native hardware row count fits its ABI");
        let selected_count = u32::try_from(outputs.selected_clusters.len())
            .expect("native selected row count fits its ABI");
        let visbuffer_count = u32::try_from(outputs.visbuffer64_entries.len())
            .expect("native VisBuffer64 row count fits its ABI");
        readback.replace_render_path_readback(
            hardware_count,
            inspection.hardware_rasterization_source,
            outputs.hardware_rasterization_records,
            selected_count,
            inspection.selected_cluster_source,
            outputs.selected_clusters,
            visbuffer_count,
            inspection.visbuffer64_source,
            native::RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
            outputs.visbuffer64_entries,
        );
        readback.replace_node_cluster_cull_readback(outputs.node_cluster_cull);
        Some(readback)
    }

    pub(crate) fn read_last_virtual_geometry_selected_clusters(
        &self,
    ) -> Result<Vec<native::RenderVirtualGeometrySelectedCluster>, String> {
        Ok(self.native_outputs()?.selected_clusters)
    }

    pub(crate) fn read_last_virtual_geometry_hardware_rasterization_records(
        &self,
    ) -> Result<Vec<native::RenderVirtualGeometryHardwareRasterizationRecord>, String> {
        Ok(self.native_outputs()?.hardware_rasterization_records)
    }

    pub(crate) fn read_last_virtual_geometry_node_and_cluster_cull_page_requests(
        &self,
    ) -> Result<Vec<u32>, String> {
        Ok(self.native_outputs()?.node_cluster_cull.page_request_ids)
    }

    pub(crate) fn read_last_virtual_geometry_visbuffer64_words(
        &self,
    ) -> Result<(u64, Vec<u64>), String> {
        Ok((
            native::RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
            self.native_outputs()?
                .visbuffer64_entries
                .into_iter()
                .map(|entry| entry.packed_value)
                .collect(),
        ))
    }

    pub(crate) fn read_last_virtual_geometry_cluster_selection_input_source(
        &self,
    ) -> native::RenderVirtualGeometryClusterSelectionInputSource {
        self.native_inspection()
            .expect("actual mapped cull input")
            .cull_input
            .cluster_selection_input_source
    }

    pub(crate) fn read_last_virtual_geometry_node_and_cluster_cull_source(
        &self,
    ) -> native::RenderVirtualGeometryNodeAndClusterCullSource {
        {
            self.native_inspection()
                .expect("actual encoded cull pass and mapped input");
            native::RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput
        }
    }

    pub(crate) fn read_last_virtual_geometry_selected_cluster_source(
        &self,
    ) -> native::RenderVirtualGeometrySelectedClusterSource {
        self.native_inspection()
            .expect("actual mapped terminal provenance")
            .selected_cluster_source
    }

    pub(crate) fn read_last_virtual_geometry_hardware_rasterization_source(
        &self,
    ) -> native::RenderVirtualGeometryHardwareRasterizationSource {
        self.native_inspection()
            .expect("actual mapped terminal provenance")
            .hardware_rasterization_source
    }

    pub(crate) fn read_last_virtual_geometry_visbuffer64_source(
        &self,
    ) -> native::RenderVirtualGeometryVisBuffer64Source {
        self.native_inspection()
            .expect("actual mapped terminal provenance")
            .visbuffer64_source
    }

    pub(crate) fn last_virtual_geometry_hardware_rasterization_record_count(&self) -> u32 {
        u32::try_from(
            self.native_outputs()
                .expect("actual mapped hardware output")
                .hardware_rasterization_records
                .len(),
        )
        .expect("bounded native row count")
    }

    pub(crate) fn last_virtual_geometry_node_and_cluster_cull_page_request_count(&self) -> u32 {
        u32::try_from(
            self.native_outputs()
                .expect("actual mapped page requests")
                .node_cluster_cull
                .page_request_ids
                .len(),
        )
        .expect("bounded native page count")
    }

    pub(crate) fn last_virtual_geometry_node_and_cluster_cull_record_count(&self) -> u32 {
        {
            self.native_inspection()
                .expect("actual encoded cull pass and mapped input");
            1
        }
    }

    pub(crate) fn read_last_virtual_geometry_cull_input_snapshot(
        &self,
    ) -> Result<Option<native::RenderVirtualGeometryCullInputSnapshot>, String> {
        Ok(Some(self.native_inspection()?.cull_input))
    }

    pub(crate) fn read_last_virtual_geometry_node_and_cluster_cull_input_snapshot(
        &self,
    ) -> Result<Option<native::RenderVirtualGeometryCullInputSnapshot>, String> {
        Ok(Some(self.native_inspection()?.cull_input))
    }

    pub(crate) fn read_last_virtual_geometry_node_and_cluster_cull_global_state_snapshot(
        &self,
    ) -> Result<Option<native::RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>, String>
    {
        Ok(self
            .native_inspection()?
            .startup
            .and_then(|startup| startup.global_state))
    }

    pub(crate) fn read_last_virtual_geometry_node_and_cluster_cull_dispatch_setup_snapshot(
        &self,
    ) -> Result<Option<native::RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot>, String>
    {
        Ok(self
            .native_inspection()?
            .startup
            .and_then(|startup| startup.dispatch_setup))
    }

    pub(crate) fn read_last_virtual_geometry_node_and_cluster_cull_launch_worklist_snapshot(
        &self,
    ) -> Result<Option<native::RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot>, String>
    {
        Ok(self
            .native_inspection()?
            .startup
            .and_then(|startup| startup.launch_worklist))
    }

    pub(crate) fn read_last_virtual_geometry_node_and_cluster_cull_instance_seeds(
        &self,
    ) -> Result<Vec<native::RenderVirtualGeometryNodeAndClusterCullInstanceSeed>, String> {
        Ok(self
            .native_inspection()?
            .startup
            .ok_or("native startup buffers were not recorded")?
            .instance_seeds)
    }

    pub(crate) fn read_last_virtual_geometry_node_and_cluster_cull_instance_work_items(
        &self,
    ) -> Result<Vec<native::RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem>, String> {
        Ok(self
            .native_inspection()?
            .startup
            .ok_or("native startup buffers were not recorded")?
            .instance_work_items)
    }

    pub(crate) fn read_last_virtual_geometry_node_and_cluster_cull_hierarchy_child_ids(
        &self,
    ) -> Result<Vec<u32>, String> {
        Ok(self
            .native_inspection()?
            .startup
            .ok_or("native startup buffers were not recorded")?
            .hierarchy_child_ids)
    }

    pub(crate) fn read_last_virtual_geometry_node_and_cluster_cull_cluster_work_items(
        &self,
    ) -> Result<
        Vec<crate::virtual_geometry::VirtualGeometryNodeAndClusterCullClusterWorkItem>,
        String,
    > {
        Ok(self
            .native_inspection()?
            .startup
            .ok_or("native startup buffers were not recorded")?
            .cluster_work_items
            .into_iter()
            .map(
                |row| crate::virtual_geometry::VirtualGeometryNodeAndClusterCullClusterWorkItem {
                    instance_index: row.instance_index,
                    entity: row.entity,
                    cluster_array_index: row.cluster_array_index,
                    hierarchy_node_id: row.hierarchy_node_id,
                    cluster_budget: row.cluster_budget,
                    page_budget: row.page_budget,
                    forced_mip: row.forced_mip,
                },
            )
            .collect())
    }

    pub(crate) fn read_last_virtual_geometry_node_and_cluster_cull_child_work_items(
        &self,
    ) -> Result<Vec<crate::virtual_geometry::VirtualGeometryNodeAndClusterCullChildWorkItem>, String>
    {
        Ok(self
            .native_inspection()?
            .startup
            .ok_or("native startup buffers were not recorded")?
            .child_work_items
            .into_iter()
            .map(
                |row| crate::virtual_geometry::VirtualGeometryNodeAndClusterCullChildWorkItem {
                    instance_index: row.instance_index,
                    entity: row.entity,
                    parent_cluster_array_index: row.parent_cluster_array_index,
                    parent_hierarchy_node_id: row.parent_hierarchy_node_id,
                    child_node_id: row.child_node_id,
                    child_table_index: row.child_table_index,
                    traversal_index: row.traversal_index,
                    cluster_budget: row.cluster_budget,
                    page_budget: row.page_budget,
                    forced_mip: row.forced_mip,
                },
            )
            .collect())
    }

    pub(crate) fn read_last_virtual_geometry_node_and_cluster_cull_traversal_records(
        &self,
    ) -> Result<
        Vec<crate::virtual_geometry::VirtualGeometryNodeAndClusterCullTraversalRecord>,
        String,
    > {
        Ok(self.native_inspection()?.startup.ok_or("native startup buffers were not recorded")?.traversal_records
            .into_iter().map(|row| crate::virtual_geometry::VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: match row.op {
                    native::RenderVirtualGeometryNodeAndClusterCullTraversalOp::VisitNode => crate::virtual_geometry::VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
                    native::RenderVirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster => crate::virtual_geometry::VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster,
                    native::RenderVirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild => crate::virtual_geometry::VirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild,
                },
                child_source: match row.child_source {
                    native::RenderVirtualGeometryNodeAndClusterCullTraversalChildSource::None => crate::virtual_geometry::VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                    native::RenderVirtualGeometryNodeAndClusterCullTraversalChildSource::FixedFanout => crate::virtual_geometry::VirtualGeometryNodeAndClusterCullTraversalChildSource::FixedFanout,
                    native::RenderVirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy => crate::virtual_geometry::VirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy,
                },
                instance_index: row.instance_index,
                entity: row.entity,
                cluster_array_index: row.cluster_array_index,
                hierarchy_node_id: row.hierarchy_node_id,
                node_cluster_start: row.node_cluster_start,
                node_cluster_count: row.node_cluster_count,
                child_base: row.child_base,
                child_count: row.child_count,
                traversal_index: row.traversal_index,
                cluster_budget: row.cluster_budget,
                page_budget: row.page_budget,
                forced_mip: row.forced_mip,
            }).collect())
    }

    pub(crate) fn read_last_virtual_geometry_gpu_readback_visbuffer64(
        &self,
    ) -> Option<(u64, Vec<native::RenderVirtualGeometryVisBuffer64Entry>)> {
        self.take_last_virtual_geometry_gpu_readback()
            .map(|readback| {
                (
                    readback.visbuffer64_clear_value(),
                    readback.visbuffer64_entries().to_vec(),
                )
            })
    }

    pub(crate) fn read_last_virtual_geometry_gpu_readback_visbuffer64_source(
        &self,
    ) -> Option<native::RenderVirtualGeometryVisBuffer64Source> {
        self.take_last_virtual_geometry_gpu_readback()
            .map(|readback| readback.visbuffer64_source())
    }

    pub(crate) fn read_last_virtual_geometry_gpu_readback_visbuffer64_entry_count(
        &self,
    ) -> Option<u32> {
        self.take_last_virtual_geometry_gpu_readback()
            .map(|readback| readback.visbuffer64_entry_count())
    }

    pub(crate) fn read_last_virtual_geometry_gpu_readback_hardware_rasterization_source(
        &self,
    ) -> Option<native::RenderVirtualGeometryHardwareRasterizationSource> {
        self.take_last_virtual_geometry_gpu_readback()
            .map(|readback| readback.hardware_rasterization_source())
    }

    pub(crate) fn read_last_virtual_geometry_gpu_readback_hardware_rasterization_record_count(
        &self,
    ) -> Option<u32> {
        self.take_last_virtual_geometry_gpu_readback()
            .map(|readback| readback.hardware_rasterization_record_count())
    }

    pub(crate) fn read_last_virtual_geometry_gpu_readback_selected_clusters(
        &self,
    ) -> Option<Vec<native::RenderVirtualGeometrySelectedCluster>> {
        self.take_last_virtual_geometry_gpu_readback()
            .map(|readback| readback.selected_clusters().to_vec())
    }

    pub(crate) fn read_last_virtual_geometry_gpu_readback_selected_cluster_source(
        &self,
    ) -> Option<native::RenderVirtualGeometrySelectedClusterSource> {
        self.take_last_virtual_geometry_gpu_readback()
            .map(|readback| readback.selected_cluster_source())
    }

    pub(crate) fn read_last_virtual_geometry_gpu_readback_selected_cluster_count(
        &self,
    ) -> Option<u32> {
        self.take_last_virtual_geometry_gpu_readback()
            .map(|readback| readback.selected_cluster_count())
    }

    pub(crate) fn framework(&self) -> &WgpuRenderFramework {
        &self.framework
    }

    pub(crate) fn framework_mut(&mut self) -> &mut WgpuRenderFramework {
        &mut self.framework
    }

    pub(crate) fn uploader_collector(&self) -> &VirtualGeometryUploaderTestCollector {
        self.uploader_collector
            .as_deref()
            .expect("uploader test collector is installed")
    }

    pub(crate) fn read_visible_cluster_words_after_submit(&self) -> Result<Vec<u32>, String> {
        self.latest_visible_cluster_words
            .clone()
            .ok_or_else(|| "virtual geometry frame has not submitted a cull readback".to_owned())?
    }

    pub(crate) fn read_nonresident_page_request_words_after_submit(
        &self,
    ) -> Result<Vec<u32>, String> {
        self.latest_cull_gpu_outputs
            .clone()
            .ok_or_else(|| "no cull GPU frame has completed".to_owned())?
            .map(|outputs| outputs.requested_pages)
    }

    pub(crate) fn read_visbuffer64_words_after_submit(&self) -> Result<Vec<u32>, String> {
        self.latest_cull_gpu_outputs
            .clone()
            .ok_or_else(|| "no cull GPU frame has completed".to_owned())?
            .map(|outputs| outputs.visbuffer64)
    }

    pub(crate) fn read_indirect_words_after_submit(&self) -> Result<IndirectReadbackWords, String> {
        match &self.latest_indirect_gpu_words {
            Some(words) => words.clone(),
            None => self.indirect_gpu_readback.read_latest_words(),
        }
    }

    pub(crate) fn render_virtual_geometry_test_frame(
        &mut self,
        viewport_size: impl Into<zircon_runtime::core::math::UVec2>,
        extract: zircon_runtime::core::framework::render::RenderFrameExtract,
        prepare: Option<crate::virtual_geometry::VirtualGeometryPrepareFrame>,
    ) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
        self.uploader_collector().set_prepare_option(prepare);
        let viewport = self.create_viewport(RenderViewportDescriptor::new(viewport_size.into()))?;
        self.set_quality_profile(
            viewport,
            RenderQualityProfile::new("virtual-geometry-test-frame")
                .with_virtual_geometry(true)
                .with_hybrid_global_illumination(false)
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_reflection_probes(false)
                .with_baked_lighting(false)
                .with_particle_rendering(false)
                .with_async_compute(false),
        )?;
        self.latest_visible_cluster_words = None;
        self.latest_indirect_gpu_words = None;
        self.latest_native_probe = None;
        self.legacy_cpu_submission_projections = [None, None];
        self.latest_viewport = Some(viewport);
        let submitted = self
            .framework
            .test_support_submit_virtual_geometry_frame(viewport, extract);
        let outputs = self.visible_cluster_readback.read_latest_outputs();
        self.latest_visible_cluster_words =
            Some(outputs.clone().map(|outputs| outputs.visible_clusters));
        self.latest_cull_gpu_outputs = Some(outputs);
        self.latest_indirect_gpu_words = Some(self.indirect_gpu_readback.read_latest_words());
        if submitted.is_ok() {
            self.latest_native_probe = self
                .framework
                .test_support_virtual_geometry_native_probe_scope(viewport)?
                .map(|scope| (viewport, scope));
        }
        submitted
    }

    /// Submits the exact graph compiled by a legacy renderer regression through the framework's
    /// real prepare collector, renderer provider, GPU feedback, and readback lifecycle.
    pub(crate) fn render_compiled_virtual_geometry_test_frame(
        &mut self,
        viewport_size: impl Into<zircon_runtime::core::math::UVec2>,
        extract: zircon_runtime::core::framework::render::RenderFrameExtract,
        prepare: Option<crate::virtual_geometry::VirtualGeometryPrepareFrame>,
        pipeline: &CompiledRenderPipeline,
    ) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
        self.render_compiled_virtual_geometry_test_frame_with_selections(
            viewport_size,
            extract,
            prepare,
            pipeline,
            None,
        )
    }

    pub(crate) fn render_compiled_virtual_geometry_test_frame_with_selections(
        &mut self,
        viewport_size: impl Into<zircon_runtime::core::math::UVec2>,
        extract: zircon_runtime::core::framework::render::RenderFrameExtract,
        prepare: Option<crate::virtual_geometry::VirtualGeometryPrepareFrame>,
        pipeline: &CompiledRenderPipeline,
        selections: Option<Vec<crate::virtual_geometry::VirtualGeometryClusterSelection>>,
    ) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
        let inputs = execution_inputs(prepare.clone(), selections);
        self.uploader_collector().set_prepare_option(prepare);
        let viewport = self.create_viewport(RenderViewportDescriptor::new(viewport_size.into()))?;
        self.set_quality_profile(
            viewport,
            RenderQualityProfile::new("virtual-geometry-compiled-test-frame")
                .with_virtual_geometry(true)
                .with_hybrid_global_illumination(false)
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_reflection_probes(false)
                .with_baked_lighting(false)
                .with_particle_rendering(false)
                .with_async_compute(false),
        )?;
        self.latest_visible_cluster_words = None;
        self.latest_indirect_gpu_words = None;
        self.latest_native_probe = None;
        self.legacy_cpu_submission_projections = [None, None];
        self.latest_viewport = Some(viewport);
        let submitted = self
            .framework
            .test_support_submit_compiled_virtual_geometry_frame_with_inputs(
                viewport, extract, pipeline, inputs,
            );
        let outputs = self.visible_cluster_readback.read_latest_outputs();
        self.latest_visible_cluster_words =
            Some(outputs.clone().map(|outputs| outputs.visible_clusters));
        self.latest_cull_gpu_outputs = Some(outputs);
        self.latest_indirect_gpu_words = Some(self.indirect_gpu_readback.read_latest_words());
        if submitted.is_ok() {
            self.latest_native_probe = self
                .framework
                .test_support_virtual_geometry_native_probe_scope(viewport)?
                .map(|scope| (viewport, scope));
        }
        submitted
    }

    /// Captures pixels and reads indirect GPU words from the same compiled frame. The returned
    /// authority remains the executor's actual 14-word GPU output; this does not rebuild it from
    /// renderer feedback or CPU records.
    pub(crate) fn render_compiled_virtual_geometry_captured_test_frame(
        &mut self,
        viewport_size: impl Into<zircon_runtime::core::math::UVec2>,
        extract: zircon_runtime::core::framework::render::RenderFrameExtract,
        prepare: Option<crate::virtual_geometry::VirtualGeometryPrepareFrame>,
        pipeline: &CompiledRenderPipeline,
    ) -> Result<
        (
            Option<RenderVirtualGeometryDebugSnapshot>,
            Vec<u8>,
            IndirectReadbackWords,
        ),
        String,
    > {
        let inputs = execution_inputs(prepare.clone(), None);
        self.uploader_collector().set_prepare_option(prepare);
        let viewport = self
            .create_viewport(RenderViewportDescriptor::new(viewport_size.into()))
            .map_err(|error| error.to_string())?;
        self.set_quality_profile(
            viewport,
            RenderQualityProfile::new("virtual-geometry-compiled-captured-test-frame")
                .with_virtual_geometry(true)
                .with_hybrid_global_illumination(false)
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_reflection_probes(false)
                .with_baked_lighting(false)
                .with_particle_rendering(false)
                .with_async_compute(false),
        )
        .map_err(|error| error.to_string())?;
        self.latest_visible_cluster_words = None;
        self.latest_indirect_gpu_words = None;
        self.latest_native_probe = None;
        self.legacy_cpu_submission_projections = [None, None];
        self.latest_viewport = Some(viewport);
        let snapshot = self
            .framework
            .test_support_submit_compiled_virtual_geometry_frame_with_inputs(
                viewport, extract, pipeline, inputs,
            )
            .map_err(|error| error.to_string())?;
        let rgba = self
            .framework
            .capture_frame(viewport)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "virtual geometry frame capture was unavailable".to_owned())?
            .rgba;
        let outputs = self.visible_cluster_readback.read_latest_outputs();
        self.latest_visible_cluster_words =
            Some(outputs.clone().map(|outputs| outputs.visible_clusters));
        self.latest_cull_gpu_outputs = Some(outputs);
        let words = self.indirect_gpu_readback.read_latest_words()?;
        self.latest_indirect_gpu_words = Some(Ok(words.clone()));
        Ok((snapshot, rgba, words))
    }

    /// Captures two consecutive compiled frames on one framework and viewport. This is used by
    /// regressions whose contract depends on renderer state carrying across the frame boundary.
    pub(crate) fn render_compiled_virtual_geometry_captured_test_frame_pair(
        &mut self,
        viewport_size: impl Into<zircon_runtime::core::math::UVec2>,
        first: (
            zircon_runtime::core::framework::render::RenderFrameExtract,
            Option<crate::virtual_geometry::VirtualGeometryPrepareFrame>,
        ),
        second: (
            zircon_runtime::core::framework::render::RenderFrameExtract,
            Option<crate::virtual_geometry::VirtualGeometryPrepareFrame>,
        ),
        pipeline: &CompiledRenderPipeline,
    ) -> Result<
        (
            (
                Option<RenderVirtualGeometryDebugSnapshot>,
                Vec<u8>,
                IndirectReadbackWords,
            ),
            (
                Option<RenderVirtualGeometryDebugSnapshot>,
                Vec<u8>,
                IndirectReadbackWords,
            ),
        ),
        String,
    > {
        let viewport = self
            .create_viewport(RenderViewportDescriptor::new(viewport_size.into()))
            .map_err(|error| error.to_string())?;
        self.set_quality_profile(
            viewport,
            RenderQualityProfile::new("virtual-geometry-compiled-captured-test-frame-pair")
                .with_virtual_geometry(true)
                .with_hybrid_global_illumination(false)
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_reflection_probes(false)
                .with_baked_lighting(false)
                .with_particle_rendering(false)
                .with_async_compute(false),
        )
        .map_err(|error| error.to_string())?;

        self.latest_visible_cluster_words = None;
        self.latest_indirect_gpu_words = None;
        self.latest_native_probe = None;
        self.legacy_cpu_submission_projections = [None, None];
        self.latest_viewport = Some(viewport);
        let first_inputs = execution_inputs(first.1.clone(), None);
        self.uploader_collector().set_prepare_option(first.1);
        let first_snapshot = self
            .framework
            .test_support_submit_compiled_virtual_geometry_frame_with_inputs(
                viewport,
                first.0,
                pipeline,
                first_inputs,
            )
            .map_err(|error| error.to_string())?;
        let first_rgba = self
            .framework
            .capture_frame(viewport)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "first virtual geometry frame capture was unavailable".to_owned())?
            .rgba;
        let outputs = self.visible_cluster_readback.read_latest_outputs();
        self.latest_visible_cluster_words =
            Some(outputs.clone().map(|outputs| outputs.visible_clusters));
        self.latest_cull_gpu_outputs = Some(outputs);
        let first_words = self.indirect_gpu_readback.read_latest_words()?;
        self.latest_indirect_gpu_words = Some(Ok(first_words.clone()));

        let second_inputs = execution_inputs(second.1.clone(), None);
        self.uploader_collector().set_prepare_option(second.1);
        let second_snapshot = self
            .framework
            .test_support_submit_compiled_virtual_geometry_frame_with_inputs(
                viewport,
                second.0,
                pipeline,
                second_inputs,
            )
            .map_err(|error| error.to_string())?;
        let second_rgba = self
            .framework
            .capture_frame(viewport)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "second virtual geometry frame capture was unavailable".to_owned())?
            .rgba;
        let outputs = self.visible_cluster_readback.read_latest_outputs();
        self.latest_visible_cluster_words =
            Some(outputs.clone().map(|outputs| outputs.visible_clusters));
        self.latest_cull_gpu_outputs = Some(outputs);
        let second_words = self.indirect_gpu_readback.read_latest_words()?;
        self.latest_indirect_gpu_words = Some(Ok(second_words.clone()));

        Ok((
            (first_snapshot, first_rgba, first_words),
            (second_snapshot, second_rgba, second_words),
        ))
    }
}

fn execution_state(
    state: crate::virtual_geometry::VirtualGeometryPrepareClusterState,
) -> zircon_runtime::core::framework::render::RenderVirtualGeometryExecutionState {
    use crate::virtual_geometry::VirtualGeometryPrepareClusterState as Input;
    use zircon_runtime::core::framework::render::RenderVirtualGeometryExecutionState as Output;
    match state {
        Input::Resident => Output::Resident,
        Input::PendingUpload => Output::PendingUpload,
        Input::Missing => Output::Missing,
    }
}

fn execution_inputs(
    prepare: Option<crate::virtual_geometry::VirtualGeometryPrepareFrame>,
    selections: Option<Vec<crate::virtual_geometry::VirtualGeometryClusterSelection>>,
) -> zircon_runtime::core::framework::render::RenderVirtualGeometryExecutionInputs {
    use zircon_runtime::core::framework::render::{
        RenderVirtualGeometryClusterSelectionInput, RenderVirtualGeometryExecutionInputs,
        RenderVirtualGeometryPrepareClusterInput, RenderVirtualGeometryPrepareDrawSegmentInput,
        RenderVirtualGeometryPrepareFrameInput, RenderVirtualGeometryPreparePageInput,
        RenderVirtualGeometryPrepareRequestInput,
    };
    let page = |p: crate::virtual_geometry::VirtualGeometryPreparePage| {
        RenderVirtualGeometryPreparePageInput {
            page_id: p.page_id,
            slot: p.slot,
            size_bytes: p.size_bytes,
        }
    };
    RenderVirtualGeometryExecutionInputs {
        selections: selections.map(|values| {
            values
                .into_iter()
                .map(|s| RenderVirtualGeometryClusterSelectionInput {
                    submission_index: s.submission_index,
                    instance_index: s.instance_index,
                    entity: s.entity,
                    cluster_id: s.cluster_id,
                    cluster_ordinal: s.cluster_ordinal,
                    page_id: s.page_id,
                    lod_level: s.lod_level,
                    submission_page_id: s.submission_page_id,
                    submission_lod_level: s.submission_lod_level,
                    entity_cluster_start_ordinal: s.entity_cluster_start_ordinal,
                    entity_cluster_span_count: s.entity_cluster_span_count,
                    entity_cluster_total_count: s.entity_cluster_total_count,
                    lineage_depth: s.lineage_depth,
                    frontier_rank: s.frontier_rank,
                    resident_slot: s.resident_slot,
                    submission_slot: s.submission_slot,
                    state: execution_state(s.state),
                })
                .collect()
        }),
        prepare: prepare.map(|p| RenderVirtualGeometryPrepareFrameInput {
            visible_entities: p.visible_entities,
            visible_clusters: p
                .visible_clusters
                .into_iter()
                .map(|c| RenderVirtualGeometryPrepareClusterInput {
                    entity: c.entity,
                    cluster_id: c.cluster_id,
                    page_id: c.page_id,
                    lod_level: c.lod_level,
                    resident_slot: c.resident_slot,
                    state: execution_state(c.state),
                })
                .collect(),
            cluster_draw_segments: p
                .cluster_draw_segments
                .into_iter()
                .map(|d| RenderVirtualGeometryPrepareDrawSegmentInput {
                    entity: d.entity,
                    cluster_id: d.cluster_id,
                    page_id: d.page_id,
                    resident_slot: d.resident_slot,
                    cluster_ordinal: d.cluster_ordinal,
                    cluster_span_count: d.cluster_span_count,
                    cluster_count: d.cluster_count,
                    lineage_depth: d.lineage_depth,
                    lod_level: d.lod_level,
                    state: execution_state(d.state),
                })
                .collect(),
            resident_pages: p.resident_pages.into_iter().map(page).collect(),
            pending_page_requests: p
                .pending_page_requests
                .into_iter()
                .map(|r| RenderVirtualGeometryPrepareRequestInput {
                    page_id: r.page_id,
                    size_bytes: r.size_bytes,
                    generation: r.generation,
                    frontier_rank: r.frontier_rank,
                    assigned_slot: r.assigned_slot,
                    recycled_page_id: r.recycled_page_id,
                })
                .collect(),
            available_slots: p.available_slots,
            evictable_pages: p.evictable_pages.into_iter().map(page).collect(),
        }),
    }
}

impl Deref for PluginizedWgpuRenderFrameworkFixture {
    type Target = WgpuRenderFramework;

    fn deref(&self) -> &Self::Target {
        &self.framework
    }
}

impl DerefMut for PluginizedWgpuRenderFrameworkFixture {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.framework
    }
}

pub(crate) fn virtual_geometry_render_feature_descriptor() -> RenderFeatureDescriptor {
    crate::render_feature_descriptor()
}

// 测试夹具先激活项目资产服务，再把插件声明、pass 执行器与 provider 一起注入 WGPU 框架。
pub(crate) fn pluginized_wgpu_render_framework_with_asset_manager(
    asset_manager: Arc<ProjectAssetManager>,
) -> PluginizedWgpuRenderFrameworkFixture {
    pluginized_wgpu_render_framework_with_asset_manager_and_uploader_collector(asset_manager, None)
}

pub(crate) fn pluginized_wgpu_render_framework_with_uploader_collector(
    asset_manager: Arc<ProjectAssetManager>,
    collector: Arc<VirtualGeometryUploaderTestCollector>,
) -> PluginizedWgpuRenderFrameworkFixture {
    pluginized_wgpu_render_framework_with_asset_manager_and_uploader_collector(
        asset_manager,
        Some(collector),
    )
}

pub(crate) fn pluginized_wgpu_render_framework_with_gpu_test_collector(
    asset_manager: Arc<ProjectAssetManager>,
) -> PluginizedWgpuRenderFrameworkFixture {
    pluginized_wgpu_render_framework_with_asset_manager_and_uploader_collector(
        asset_manager,
        Some(Arc::new(VirtualGeometryUploaderTestCollector::default())),
    )
}

pub(crate) fn pluginized_wgpu_render_framework_with_existing_project_and_gpu_test_collector(
    asset_manager: Arc<ProjectAssetManager>,
) -> PluginizedWgpuRenderFrameworkFixture {
    pluginized_wgpu_render_framework_with_collector_and_project_policy(
        asset_manager,
        Some(Arc::new(VirtualGeometryUploaderTestCollector::default())),
        false,
    )
}

fn pluginized_wgpu_render_framework_with_asset_manager_and_uploader_collector(
    asset_manager: Arc<ProjectAssetManager>,
    uploader_collector: Option<Arc<VirtualGeometryUploaderTestCollector>>,
) -> PluginizedWgpuRenderFrameworkFixture {
    pluginized_wgpu_render_framework_with_collector_and_project_policy(
        asset_manager,
        uploader_collector,
        true,
    )
}

fn pluginized_wgpu_render_framework_with_collector_and_project_policy(
    asset_manager: Arc<ProjectAssetManager>,
    uploader_collector: Option<Arc<VirtualGeometryUploaderTestCollector>>,
    initialize_active_project: bool,
) -> PluginizedWgpuRenderFrameworkFixture {
    if initialize_active_project && asset_manager.current_project_snapshot().is_none() {
        open_uploader_active_project(&asset_manager);
    }
    let asset_runtime = CoreRuntime::new();
    asset_runtime
        .register_module(
            ModuleDescriptor::new(TEST_ASSET_MODULE_NAME, "virtual geometry test assets")
                .with_manager(ManagerDescriptor::new(
                    RegistryName::from_parts(
                        TEST_ASSET_MODULE_NAME,
                        ServiceKind::Manager,
                        "ProjectAssetManager",
                    ),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        Ok(
                            Arc::new(RegisteredManagerService::new(Arc::clone(&asset_manager)))
                                as ServiceObject,
                        )
                    }),
                )),
        )
        .expect("virtual geometry test ProjectAssetManager service should register");
    asset_runtime
        .activate_module(TEST_ASSET_MODULE_NAME)
        .expect("virtual geometry test ProjectAssetManager module should activate");
    let core = asset_runtime.handle();
    let handle = manager_service_handle(&core, TEST_ASSET_SERVICE_NAME)
        .expect("virtual geometry test ProjectAssetManager handle should resolve");
    let runtime_prepare_collectors = uploader_collector
        .as_ref()
        .map(|collector| uploader_test_collector_registration(Arc::clone(collector)))
        .into_iter()
        .collect::<Vec<_>>();
    let visible_cluster_readback = NodeClusterCullReadback::new();
    let mut render_pass_executors = crate::render_pass_executor_registrations();
    render_pass_executors.retain(|registration| {
        !matches!(
            registration.executor_id.as_str(),
            "virtual-geometry.node-cluster-cull" | "virtual-geometry.indirect-args"
        )
    });
    render_pass_executors.push(visible_cluster_readback.executor_registration());
    let indirect_gpu_readback = IndirectGpuReadback::new();
    render_pass_executors.push(indirect_gpu_readback.executor_registration());
    let framework = WgpuRenderFramework::new_with_plugin_render_extensions(
        ProjectAssetManagerAccess::new(core, handle),
        [virtual_geometry_render_feature_descriptor()],
        render_pass_executors,
        runtime_prepare_collectors,
        [],
        [crate::virtual_geometry_runtime_provider_registration()],
        asset_runtime.task_graph().worker_pool().clone(),
    )
    .unwrap();
    PluginizedWgpuRenderFrameworkFixture {
        framework,
        latest_viewport: None,
        _asset_runtime: asset_runtime,
        uploader_collector,
        visible_cluster_readback,
        latest_visible_cluster_words: None,
        latest_cull_gpu_outputs: None,
        indirect_gpu_readback,
        latest_indirect_gpu_words: None,
        latest_native_probe: None,
        legacy_cpu_submission_projections: [None, None],
    }
}

fn open_uploader_active_project(manager: &ProjectAssetManager) {
    let output = std::env::temp_dir();
    assert_physical_fixture_root(&output);
    std::fs::create_dir_all(&output).expect("create the registered virtual geometry fixture root");
    assert_physical_fixture_root(&output);
    let requested = output
        .to_str()
        .expect("registered fixture root is UTF-8")
        .replace('\\', "/");
    let requested = requested.trim_end_matches('/');
    let resolved = std::fs::canonicalize(&output)
        .expect("resolve registered virtual geometry fixture root")
        .to_str()
        .expect("resolved fixture root is UTF-8")
        .replace('\\', "/");
    let resolved = resolved.strip_prefix("//?/").unwrap_or(&resolved);
    assert!(
        resolved.eq_ignore_ascii_case(requested),
        "virtual geometry fixture root must retain its registered physical path"
    );

    let project_root = output.join(format!(
        "vg-uploader-active-project-{}-{}",
        std::process::id(),
        NEXT_UPLOADER_PROJECT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&project_root)
        .expect("create an exclusively owned virtual geometry uploader project");
    let paths = ProjectPaths::from_root(&project_root)
        .expect("construct physical virtual geometry project paths");
    let manifest = ProjectManifest::new(
        "VirtualGeometryUploaderActiveProject",
        AssetUri::parse("res://scenes/main.scene.toml").expect("valid project scene URI"),
        1,
    );
    paths
        .ensure_layout(&manifest.asset_roots)
        .expect("create active project asset roots");
    manifest
        .save(paths.manifest_path())
        .expect("save virtual geometry uploader project manifest");
    AssetManager::open_project(
        manager,
        project_root
            .to_str()
            .expect("virtual geometry project path is UTF-8"),
    )
    .expect("activate virtual geometry project before renderer initialization");
}

fn assert_physical_fixture_root(output: &Path) {
    let text = output
        .to_str()
        .expect("fixture path is UTF-8")
        .replace('\\', "/");
    let text = text.trim_end_matches('/');
    let bytes = text.as_bytes();
    let expected = std::path::PathBuf::from(
        std::env::var_os("ZIRCON_RENDERING_SMOKE_OUTPUT_DIR")
            .expect("registered virtual geometry fixture root is required"),
    );
    let expected_text = expected
        .to_str()
        .expect("registered fixture root is UTF-8")
        .replace('\\', "/");
    let expected_text = expected_text.trim_end_matches('/');
    assert!(
        output.is_absolute() && expected.is_absolute() && text.eq_ignore_ascii_case(expected_text),
        "TEMP must equal the explicitly registered rendering fixture root"
    );
    assert!(
        bytes.len() > 3
            && [b'D', b'E', b'F']
                .iter()
                .any(|drive| bytes[0].eq_ignore_ascii_case(drive))
            && bytes[1..3] == *b":/",
        "virtual geometry fixtures require a registered physical D/E/F drive root"
    );
    let segments = text[3..].split('/').collect::<Vec<_>>();
    assert!(
        segments.iter().all(|segment| {
            !segment.is_empty() && *segment != "." && *segment != ".." && !segment.contains(':')
        }),
        "virtual geometry fixture path must not contain aliases"
    );
    let reports_layout = segments.len() == 5
        && segments[2].eq_ignore_ascii_case("reports")
        && segments[4].eq_ignore_ascii_case("fixtures");
    let build_layout = segments.len() == 7
        && segments[2].eq_ignore_ascii_case("win32")
        && segments[4].eq_ignore_ascii_case("build")
        && segments[5].eq_ignore_ascii_case("fixtures");
    assert!(
        (reports_layout || build_layout)
            && segments[0].eq_ignore_ascii_case("cargo-targets")
            && segments[1].eq_ignore_ascii_case("zircon-local"),
        "fixture root must use the registered task reports or build fixture layout"
    );
    for ancestor in output.ancestors() {
        match std::fs::symlink_metadata(ancestor) {
            Ok(metadata) => {
                assert!(
                    !metadata.file_type().is_symlink(),
                    "fixture ancestor is a symlink"
                );
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    assert_eq!(
                        metadata.file_attributes() & 0x400,
                        0,
                        "fixture ancestor is a reparse point"
                    );
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => panic!("inspect virtual geometry fixture ancestor: {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime::graphics::RenderFeatureCapabilityRequirement;

    use super::*;

    #[test]
    fn render_feature_fixture_uses_plugin_virtual_geometry_descriptor() {
        let descriptor = virtual_geometry_render_feature_descriptor();

        assert_eq!(descriptor.name, crate::VIRTUAL_GEOMETRY_FEATURE_NAME);
        assert_eq!(
            descriptor.capability_requirements,
            vec![RenderFeatureCapabilityRequirement::VirtualGeometry]
        );
    }
}

impl PluginizedWgpuRenderFrameworkFixture {
    pub(crate) fn create_virtual_geometry_test_viewport(
        &mut self,
        size: zircon_runtime::core::math::UVec2,
    ) -> Result<zircon_runtime::core::framework::render::RenderViewportHandle, RenderFrameworkError>
    {
        let viewport = self.create_viewport(RenderViewportDescriptor::new(size))?;
        self.set_quality_profile(
            viewport,
            RenderQualityProfile::new("virtual-geometry-history-test")
                .with_virtual_geometry(true)
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_reflection_probes(false)
                .with_baked_lighting(false)
                .with_particle_rendering(false)
                .with_async_compute(false),
        )?;
        Ok(viewport)
    }
    pub(crate) fn submit_compiled_virtual_geometry_at_viewport(
        &mut self,
        viewport: zircon_runtime::core::framework::render::RenderViewportHandle,
        extract: zircon_runtime::core::framework::render::RenderFrameExtract,
        prepare: Option<crate::virtual_geometry::VirtualGeometryPrepareFrame>,
        pipeline: &CompiledRenderPipeline,
        selections: Option<Vec<crate::virtual_geometry::VirtualGeometryClusterSelection>>,
    ) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
        let inputs = execution_inputs(prepare.clone(), selections);
        self.uploader_collector().set_prepare_option(prepare);
        self.latest_visible_cluster_words = None;
        self.latest_cull_gpu_outputs = None;
        self.latest_indirect_gpu_words = None;
        self.latest_native_probe = None;
        self.legacy_cpu_submission_projections = [None, None];
        self.latest_viewport = Some(viewport);
        let submitted = self
            .framework
            .test_support_submit_compiled_virtual_geometry_frame_with_inputs(
                viewport, extract, pipeline, inputs,
            );
        if submitted.is_ok() {
            let outputs = self.visible_cluster_readback.read_latest_outputs();
            self.latest_visible_cluster_words = Some(outputs.clone().map(|v| v.visible_clusters));
            self.latest_cull_gpu_outputs = Some(outputs);
            self.latest_indirect_gpu_words = Some(self.indirect_gpu_readback.read_latest_words());
        }
        if submitted.is_ok() {
            self.latest_native_probe = self
                .framework
                .test_support_virtual_geometry_native_probe_scope(viewport)?
                .map(|scope| (viewport, scope));
        }
        submitted
    }
}
