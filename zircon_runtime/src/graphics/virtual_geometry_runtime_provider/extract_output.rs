use crate::core::framework::render::{
    RenderVirtualGeometryBvhVisualizationInstance, RenderVirtualGeometryCpuReferenceInstance,
    RenderVirtualGeometryExtract, RenderVirtualGeometryPagePayload,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VirtualGeometryRuntimeExtractOutput {
    extract: RenderVirtualGeometryExtract,
    cpu_reference_instances: Vec<RenderVirtualGeometryCpuReferenceInstance>,
    bvh_visualization_instances: Vec<RenderVirtualGeometryBvhVisualizationInstance>,
    resident_page_payloads: Vec<RenderVirtualGeometryPagePayload>,
}

impl VirtualGeometryRuntimeExtractOutput {
    pub fn new(
        extract: RenderVirtualGeometryExtract,
        cpu_reference_instances: Vec<RenderVirtualGeometryCpuReferenceInstance>,
        bvh_visualization_instances: Vec<RenderVirtualGeometryBvhVisualizationInstance>,
        resident_page_payloads: Vec<RenderVirtualGeometryPagePayload>,
    ) -> Self {
        Self {
            extract,
            cpu_reference_instances,
            bvh_visualization_instances,
            resident_page_payloads,
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

    pub fn resident_page_payloads(&self) -> &[RenderVirtualGeometryPagePayload] {
        &self.resident_page_payloads
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        RenderVirtualGeometryExtract,
        Vec<RenderVirtualGeometryCpuReferenceInstance>,
        Vec<RenderVirtualGeometryBvhVisualizationInstance>,
        Vec<RenderVirtualGeometryPagePayload>,
    ) {
        (
            self.extract,
            self.cpu_reference_instances,
            self.bvh_visualization_instances,
            self.resident_page_payloads,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::VirtualGeometryRuntimeExtractOutput;

    #[test]
    fn runtime_extract_output_moves_all_parts_without_clone_projection() {
        let (extract, cpu_references, bvh_instances, resident_payloads) =
            VirtualGeometryRuntimeExtractOutput::default().into_parts();

        assert_eq!(extract, Default::default());
        assert!(cpu_references.is_empty());
        assert!(bvh_instances.is_empty());
        assert!(resident_payloads.is_empty());
    }
}
