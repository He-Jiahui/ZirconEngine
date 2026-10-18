use std::sync::Arc;

use crate::virtual_geometry::types::{
    VirtualGeometryNodeAndClusterCullChildWorkItem,
    VirtualGeometryNodeAndClusterCullClusterWorkItem,
    VirtualGeometryNodeAndClusterCullTraversalRecord,
};
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
};
use zircon_runtime::graphics::{RenderPassGpuRecordingContext, RenderPassGpuResourceFactory};

use super::super::packed_words::collect_fixed_packed_words;

fn create_prepared_words_buffer(
    factory: &(impl RenderPassGpuResourceFactory + ?Sized),
    label: &str,
    words: &[u32],
) -> Option<Arc<wgpu::Buffer>> {
    if words.is_empty() {
        return None;
    }
    Some(Arc::new(factory.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}

pub(super) fn create_node_and_cluster_cull_dispatch_setup_buffer(
    device: &(impl RenderPassGpuResourceFactory + ?Sized),
    dispatch_setup: &RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
) -> Option<Arc<wgpu::Buffer>> {
    create_prepared_words_buffer(
        device,
        "zircon-vg-node-and-cluster-cull-dispatch-setup",
        &dispatch_setup.packed_words(),
    )
}

pub(super) fn create_node_and_cluster_cull_launch_worklist_buffer(
    device: &(impl RenderPassGpuResourceFactory + ?Sized),
    launch_worklist: &RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
) -> Option<Arc<wgpu::Buffer>> {
    create_prepared_words_buffer(
        device,
        "zircon-vg-node-and-cluster-cull-launch-worklist",
        &launch_worklist.packed_words(),
    )
}

pub(super) fn create_node_and_cluster_cull_instance_seed_buffer(
    device: &(impl RenderPassGpuResourceFactory + ?Sized),
    instance_seeds: &[RenderVirtualGeometryNodeAndClusterCullInstanceSeed],
) -> Option<Arc<wgpu::Buffer>> {
    create_prepared_words_buffer(
        device,
        "zircon-vg-node-and-cluster-cull-instance-seeds",
        &collect_fixed_packed_words(
            instance_seeds,
            RenderVirtualGeometryNodeAndClusterCullInstanceSeed::packed_words,
        ),
    )
}

pub(super) fn create_node_and_cluster_cull_hierarchy_child_id_buffer(
    device: &(impl RenderPassGpuResourceFactory + ?Sized),
    hierarchy_child_ids: &[u32],
) -> Option<Arc<wgpu::Buffer>> {
    create_prepared_words_buffer(
        device,
        "zircon-vg-node-and-cluster-cull-hierarchy-child-ids",
        &hierarchy_child_ids.to_vec(),
    )
}

pub(super) fn create_node_and_cluster_cull_page_request_buffer(
    device: &(impl RenderPassGpuResourceFactory + ?Sized),
    page_request_ids: &[u32],
) -> Option<Arc<wgpu::Buffer>> {
    create_prepared_words_buffer(
        device,
        "zircon-vg-node-and-cluster-cull-page-requests",
        &page_request_ids.to_vec(),
    )
}

/// CPU policy uploads plus the real GPU-generated startup work-item buffer.
/// These resources are not candidate GPU readback outputs.
pub(crate) struct MaterializedPreparedCull {
    inspection: zircon_runtime::core::framework::render::RenderVirtualGeometryPreparedCpuInspection,
    buffer: Option<Arc<wgpu::Buffer>>,
    dispatch_setup_buffer: Option<Arc<wgpu::Buffer>>,
    launch_worklist_buffer: Option<Arc<wgpu::Buffer>>,
    instance_seed_buffer: Option<Arc<wgpu::Buffer>>,
    instance_work_item_buffer: Option<Arc<wgpu::Buffer>>,
    cluster_work_item_buffer: Option<Arc<wgpu::Buffer>>,
    hierarchy_child_id_buffer: Option<Arc<wgpu::Buffer>>,
    child_work_item_buffer: Option<Arc<wgpu::Buffer>>,
    traversal_record_buffer: Option<Arc<wgpu::Buffer>>,
    page_request_buffer: Option<Arc<wgpu::Buffer>>,
}

pub(crate) fn materialize_prepared_cull(
    recording: &mut impl RenderPassGpuRecordingContext,
    inspection: &zircon_runtime::core::framework::render::RenderVirtualGeometryPreparedCpuInspection,
) -> MaterializedPreparedCull {
    let factory = recording.resource_factory();
    let buffer = inspection.global_state.as_ref().and_then(|state| {
        create_prepared_words_buffer(
            factory,
            "zircon-vg-node-and-cluster-cull-buffer",
            &state.packed_words(),
        )
    });
    let dispatch_setup_buffer = inspection
        .dispatch_setup
        .as_ref()
        .and_then(|setup| create_node_and_cluster_cull_dispatch_setup_buffer(factory, setup));
    let launch_worklist_buffer = inspection.launch_worklist.as_ref().and_then(|worklist| {
        create_node_and_cluster_cull_launch_worklist_buffer(factory, worklist)
    });
    let instance_seed_buffer =
        create_node_and_cluster_cull_instance_seed_buffer(factory, &inspection.instance_seeds);
    let cluster_work_item_buffer = create_prepared_words_buffer(
        factory,
        "zircon-vg-node-and-cluster-cull-cluster-work-items",
        &collect_fixed_packed_words(&inspection.cluster_work_items, |row| row.packed_words()),
    );
    let hierarchy_child_id_buffer = create_node_and_cluster_cull_hierarchy_child_id_buffer(
        factory,
        &inspection.hierarchy_child_ids,
    );
    let child_work_item_buffer = create_prepared_words_buffer(
        factory,
        "zircon-vg-node-and-cluster-cull-child-work-items",
        &collect_fixed_packed_words(&inspection.child_work_items, |row| row.packed_words()),
    );
    let traversal_record_buffer = create_prepared_words_buffer(
        factory,
        "zircon-vg-node-and-cluster-cull-traversal-records",
        &collect_fixed_packed_words(&inspection.traversal_records, |row| row.packed_words()),
    );
    let page_request_buffer =
        create_node_and_cluster_cull_page_request_buffer(factory, &inspection.page_request_ids);
    let instance_work_item_buffer = inspection.dispatch_setup.and_then(|setup| {
        let count = u32::try_from(inspection.instance_work_items.len()).ok()?;
        if count == 0 || launch_worklist_buffer.is_none() { return None; }
        let (layout, pipeline) = crate::virtual_geometry::renderer::VirtualGeometryGpuResources::create_prepared_cull_startup_pipeline(recording.resource_factory());
        crate::virtual_geometry::renderer::VirtualGeometryGpuResources::record_prepared_cull_instance_work_items(
            recording, &layout, &pipeline, launch_worklist_buffer.as_ref(), setup, count)
    });
    MaterializedPreparedCull {
        inspection: inspection.clone(),
        buffer,
        dispatch_setup_buffer,
        launch_worklist_buffer,
        instance_seed_buffer,
        instance_work_item_buffer,
        cluster_work_item_buffer,
        hierarchy_child_id_buffer,
        child_work_item_buffer,
        traversal_record_buffer,
        page_request_buffer,
    }
}

impl MaterializedPreparedCull {
    /// Returns leases to the actual uploaded policy and computed startup buffers.
    /// Schema IDs are stable within the current-frame inspection envelope only.
    pub(crate) fn native_inspection_sections(
        &self,
    ) -> Result<Vec<(u32, wgpu::Buffer, std::ops::Range<u64>)>, String> {
        use zircon_runtime::core::framework::render::*;
        let words = [
            self.inspection.global_state.as_ref().map_or(0, |_| {
                RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot::GPU_WORD_COUNT
            }),
            self.inspection.dispatch_setup.as_ref().map_or(0, |_| {
                RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot::GPU_WORD_COUNT
            }),
            self.inspection
                .launch_worklist
                .as_ref()
                .map_or(0, |worklist| worklist.packed_words().len()),
            self.inspection
                .instance_seeds
                .len()
                .checked_mul(RenderVirtualGeometryNodeAndClusterCullInstanceSeed::GPU_WORD_COUNT)
                .ok_or("seed schema length overflow")?,
            self.inspection
                .instance_work_items
                .len()
                .checked_mul(
                    RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem::GPU_WORD_COUNT,
                )
                .ok_or("instance schema length overflow")?,
            self.inspection
                .cluster_work_items
                .len()
                .checked_mul(RenderVirtualGeometryNodeAndClusterCullClusterWorkItem::GPU_WORD_COUNT)
                .ok_or("cluster schema length overflow")?,
            self.inspection.hierarchy_child_ids.len(),
            self.inspection
                .child_work_items
                .len()
                .checked_mul(RenderVirtualGeometryNodeAndClusterCullChildWorkItem::GPU_WORD_COUNT)
                .ok_or("child schema length overflow")?,
            self.inspection
                .traversal_records
                .len()
                .checked_mul(RenderVirtualGeometryNodeAndClusterCullTraversalRecord::GPU_WORD_COUNT)
                .ok_or("traversal schema length overflow")?,
            self.inspection.page_request_ids.len(),
        ];
        let buffers = [
            (1, &self.buffer),
            (2, &self.dispatch_setup_buffer),
            (3, &self.launch_worklist_buffer),
            (4, &self.instance_seed_buffer),
            (5, &self.instance_work_item_buffer),
            (6, &self.cluster_work_item_buffer),
            (7, &self.hierarchy_child_id_buffer),
            (8, &self.child_work_item_buffer),
            (9, &self.traversal_record_buffer),
            (10, &self.page_request_buffer),
        ];
        let mut sections = Vec::new();
        for ((id, buffer), word_count) in buffers.into_iter().zip(words) {
            let expected = u64::try_from(word_count)
                .map_err(|_| "native schema does not fit u64")?
                .checked_mul(4)
                .ok_or("native schema byte length overflow")?;
            match (buffer, expected) {
                (None, 0) => {}
                (Some(buffer), size) if size != 0 && buffer.size() == size => {
                    sections.push((id, buffer.as_ref().clone(), 0..size));
                }
                _ => return Err("materialized startup buffer is absent or disagrees with its actual upload schema".into()),
            }
        }
        Ok(sections)
    }

    pub(super) fn into_pass_output(
        self,
    ) -> super::output::VirtualGeometryNodeAndClusterCullPassOutput {
        use super::store_parts::VirtualGeometryNodeAndClusterCullPassStoreParts;
        use crate::virtual_geometry::types::{
            VirtualGeometryNodeAndClusterCullTraversalChildSource as LegacyChildSource,
            VirtualGeometryNodeAndClusterCullTraversalOp as LegacyOp,
        };
        use zircon_runtime::core::framework::render::{
            RenderVirtualGeometryNodeAndClusterCullTraversalChildSource as ChildSource,
            RenderVirtualGeometryNodeAndClusterCullTraversalOp as Op,
        };
        let inspection = self.inspection;
        let cluster_work_items = inspection
            .cluster_work_items
            .into_iter()
            .map(|row| VirtualGeometryNodeAndClusterCullClusterWorkItem {
                instance_index: row.instance_index,
                entity: row.entity,
                cluster_array_index: row.cluster_array_index,
                hierarchy_node_id: row.hierarchy_node_id,
                cluster_budget: row.cluster_budget,
                page_budget: row.page_budget,
                forced_mip: row.forced_mip,
            })
            .collect::<Vec<_>>();
        let child_work_items = inspection
            .child_work_items
            .into_iter()
            .map(|row| VirtualGeometryNodeAndClusterCullChildWorkItem {
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
            })
            .collect::<Vec<_>>();
        let traversal_records = inspection
            .traversal_records
            .into_iter()
            .map(|row| VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: match row.op {
                    Op::VisitNode => LegacyOp::VisitNode,
                    Op::StoreCluster => LegacyOp::StoreCluster,
                    Op::EnqueueChild => LegacyOp::EnqueueChild,
                },
                child_source: match row.child_source {
                    ChildSource::None => LegacyChildSource::None,
                    ChildSource::FixedFanout => LegacyChildSource::FixedFanout,
                    ChildSource::AuthoredHierarchy => LegacyChildSource::AuthoredHierarchy,
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
            })
            .collect::<Vec<_>>();
        super::output::VirtualGeometryNodeAndClusterCullPassOutput::from_store_parts(
            VirtualGeometryNodeAndClusterCullPassStoreParts {
                source: inspection.source,
                record_count: inspection.record_count,
                global_state: inspection.global_state,
                dispatch_setup: inspection.dispatch_setup,
                launch_worklist: inspection.launch_worklist,
                instance_seed_count: u32::try_from(inspection.instance_seeds.len())
                    .unwrap_or(u32::MAX),
                instance_seeds: inspection.instance_seeds,
                instance_work_item_count: u32::try_from(inspection.instance_work_items.len())
                    .unwrap_or(u32::MAX),
                instance_work_items: inspection.instance_work_items,
                cluster_work_item_count: u32::try_from(cluster_work_items.len())
                    .unwrap_or(u32::MAX),
                cluster_work_items,
                hierarchy_child_ids: inspection.hierarchy_child_ids,
                child_work_item_count: u32::try_from(child_work_items.len()).unwrap_or(u32::MAX),
                child_work_items,
                traversal_record_count: u32::try_from(traversal_records.len()).unwrap_or(u32::MAX),
                traversal_records,
                page_request_count: u32::try_from(inspection.page_request_ids.len())
                    .unwrap_or(u32::MAX),
                page_request_ids: inspection.page_request_ids,
                buffer: self.buffer,
                dispatch_setup_buffer: self.dispatch_setup_buffer,
                launch_worklist_buffer: self.launch_worklist_buffer,
                instance_seed_buffer: self.instance_seed_buffer,
                instance_work_item_buffer: self.instance_work_item_buffer,
                cluster_work_item_buffer: self.cluster_work_item_buffer,
                hierarchy_child_id_buffer: self.hierarchy_child_id_buffer,
                child_work_item_buffer: self.child_work_item_buffer,
                traversal_record_buffer: self.traversal_record_buffer,
                page_request_buffer: self.page_request_buffer,
            },
        )
    }
}
