use crate::virtual_geometry::renderer::{
    VirtualGeometryGpuReadback, VirtualGeometryGpuReadbackCompletionParts,
};
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryNodeClusterCullReadbackOutputs, RenderVirtualGeometryReadbackOutputs,
};

#[derive(Default)]
pub(super) struct VirtualGeometryReadbackOutputs {
    gpu_readback: Option<VirtualGeometryGpuReadback>,
}

impl VirtualGeometryReadbackOutputs {
    pub(in crate::virtual_geometry::renderer) fn store_gpu_readback(
        &mut self,
        readback: Option<VirtualGeometryGpuReadback>,
    ) {
        self.gpu_readback = readback;
    }

    pub(in crate::virtual_geometry::renderer) fn store_node_cluster_cull_readback(
        &mut self,
        node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs,
    ) {
        self.gpu_readback
            .get_or_insert_with(|| {
                VirtualGeometryGpuReadback::new(Vec::new(), Vec::new(), Vec::new(), Vec::new())
            })
            .replace_node_cluster_cull_readback(node_cluster_cull);
    }

    pub(in crate::virtual_geometry::renderer) fn has_gpu_readback(&self) -> bool {
        self.gpu_readback.is_some()
    }

    pub(in crate::virtual_geometry::renderer) fn gpu_readback_mut(
        &mut self,
    ) -> Option<&mut VirtualGeometryGpuReadback> {
        self.gpu_readback.as_mut()
    }

    pub(in crate::virtual_geometry::renderer) fn gpu_readback(
        &self,
    ) -> Option<&VirtualGeometryGpuReadback> {
        self.gpu_readback.as_ref()
    }

    pub(in crate::virtual_geometry::renderer) fn take_gpu_completion_parts(
        &mut self,
    ) -> Option<VirtualGeometryGpuReadbackCompletionParts> {
        self.gpu_readback
            .take()
            .map(VirtualGeometryGpuReadback::into_completion_parts)
    }

    pub(in crate::virtual_geometry::renderer) fn take_neutral_readback_outputs(
        &mut self,
    ) -> RenderVirtualGeometryReadbackOutputs {
        self.gpu_readback
            .take()
            .map(RenderVirtualGeometryReadbackOutputs::from)
            .unwrap_or_default()
    }

    #[cfg(test)]
    pub(crate) fn take_gpu_readback(&mut self) -> Option<VirtualGeometryGpuReadback> {
        self.gpu_readback.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_neutral_readback_outputs_projects_and_consumes_gpu_readback() {
        let mut outputs = VirtualGeometryReadbackOutputs::default();
        outputs.store_gpu_readback(Some(VirtualGeometryGpuReadback::new(
            vec![(44, 6)],
            vec![44],
            vec![(44, 6)],
            vec![(44, 11)],
        )));

        let neutral = outputs.take_neutral_readback_outputs();

        assert_eq!(neutral.page_table_entries, vec![44, 6]);
        assert_eq!(neutral.completed_page_assignments[0].page_id, 44);
        assert_eq!(neutral.completed_page_assignments[0].physical_slot, 6);
        assert_eq!(neutral.page_replacements[0].old_page_id, 11);
        assert_eq!(neutral.page_replacements[0].new_page_id, 44);
        assert_eq!(neutral.page_replacements[0].physical_slot, 6);
        assert_eq!(
            outputs.take_neutral_readback_outputs(),
            RenderVirtualGeometryReadbackOutputs::default()
        );
    }

    #[test]
    fn take_neutral_readback_outputs_projects_node_cluster_cull_without_gpu_completion() {
        let mut outputs = VirtualGeometryReadbackOutputs::default();
        outputs.store_node_cluster_cull_readback(
            RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                page_request_ids: vec![300, 301],
                ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
            },
        );

        let neutral = outputs.take_neutral_readback_outputs();

        assert_eq!(neutral.node_cluster_cull.page_request_ids, vec![300, 301]);
        assert!(neutral.page_table_entries.is_empty());
        assert_eq!(
            outputs.take_neutral_readback_outputs(),
            RenderVirtualGeometryReadbackOutputs::default()
        );
    }
}
