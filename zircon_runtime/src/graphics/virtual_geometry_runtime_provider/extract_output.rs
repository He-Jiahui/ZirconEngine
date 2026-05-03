use crate::core::framework::render::{
    RenderVirtualGeometryBvhVisualizationInstance, RenderVirtualGeometryCpuReferenceInstance,
    RenderVirtualGeometryExtract,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VirtualGeometryRuntimeExtractOutput {
    extract: RenderVirtualGeometryExtract,
    cpu_reference_instances: Vec<RenderVirtualGeometryCpuReferenceInstance>,
    bvh_visualization_instances: Vec<RenderVirtualGeometryBvhVisualizationInstance>,
}

impl VirtualGeometryRuntimeExtractOutput {
    pub fn new(
        extract: RenderVirtualGeometryExtract,
        cpu_reference_instances: Vec<RenderVirtualGeometryCpuReferenceInstance>,
        bvh_visualization_instances: Vec<RenderVirtualGeometryBvhVisualizationInstance>,
    ) -> Self {
        Self {
            extract,
            cpu_reference_instances,
            bvh_visualization_instances,
        }
    }

    pub fn extract(&self) -> &RenderVirtualGeometryExtract {
        &self.extract
    }

    pub fn cpu_reference_instances(&self) -> &[RenderVirtualGeometryCpuReferenceInstance] {
        &self.cpu_reference_instances
    }

    pub fn bvh_visualization_instances(&self) -> &[RenderVirtualGeometryBvhVisualizationInstance] {
        &self.bvh_visualization_instances
    }
}
