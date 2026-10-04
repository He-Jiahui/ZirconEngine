use crate::core::framework::render::{
    RenderVirtualGeometryBvhVisualizationInstance, RenderVirtualGeometryCpuReferenceInstance,
    RenderVirtualGeometryExtract, RenderVirtualGeometryPagePayload,
};

#[derive(Clone, Debug, Default, PartialEq)]
/// 自动提取的一次性结果；几何、CPU 参考实例、BVH 可视化与驻留页必须来自同一批网格资产。
/// 框架在构建帧上下文时消费全部字段，避免只更新提取内容而丢失随附资源。
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
#[path = "tests/extract_output.rs"]
mod tests;
