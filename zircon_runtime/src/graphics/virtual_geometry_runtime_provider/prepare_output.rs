use crate::core::framework::render::RenderPluginRendererOutputs;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VirtualGeometryRuntimePrepareOutput {
    evictable_page_ids: Vec<u32>,
    renderer_outputs: RenderPluginRendererOutputs,
}

impl VirtualGeometryRuntimePrepareOutput {
    pub fn new(evictable_page_ids: Vec<u32>) -> Self {
        Self {
            evictable_page_ids,
            renderer_outputs: RenderPluginRendererOutputs::default(),
        }
    }

    pub fn with_renderer_outputs(mut self, renderer_outputs: RenderPluginRendererOutputs) -> Self {
        self.renderer_outputs = renderer_outputs;
        self
    }

    pub fn renderer_outputs(&self) -> &RenderPluginRendererOutputs {
        &self.renderer_outputs
    }

    pub fn into_evictable_page_ids(self) -> Vec<u32> {
        self.evictable_page_ids
    }

    pub(crate) fn into_parts(self) -> (Vec<u32>, RenderPluginRendererOutputs) {
        (self.evictable_page_ids, self.renderer_outputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        RenderVirtualGeometryNodeClusterCullReadbackOutputs, RenderVirtualGeometryReadbackOutputs,
    };

    #[test]
    fn prepare_output_carries_neutral_virtual_geometry_renderer_outputs() {
        let output = VirtualGeometryRuntimePrepareOutput::new(vec![3]).with_renderer_outputs(
            RenderPluginRendererOutputs {
                virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                    node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                        page_request_ids: vec![300, 301],
                        ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
                    },
                    ..RenderVirtualGeometryReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            },
        );

        assert_eq!(
            output
                .renderer_outputs()
                .virtual_geometry
                .node_cluster_cull
                .page_request_ids,
            vec![300, 301]
        );

        let (evictable_page_ids, renderer_outputs) = output.into_parts();
        assert_eq!(evictable_page_ids, vec![3]);
        assert_eq!(
            renderer_outputs
                .virtual_geometry
                .node_cluster_cull
                .page_request_ids,
            vec![300, 301]
        );
    }
}
