use crate::core::framework::render::RenderVirtualGeometryExtract;
use crate::graphics::{
    runtime_provider::RuntimeProviderPrepareInput, VisibilityVirtualGeometryCluster,
    VisibilityVirtualGeometryDrawSegment, VisibilityVirtualGeometryPageUploadPlan,
};

pub struct VirtualGeometryRuntimePrepareInput<'a> {
    input: RuntimeProviderPrepareInput<'a, RenderVirtualGeometryExtract>,
    page_upload_plan: Option<&'a VisibilityVirtualGeometryPageUploadPlan>,
    visible_clusters: &'a [VisibilityVirtualGeometryCluster],
    visibility_draw_segments: &'a [VisibilityVirtualGeometryDrawSegment],
    camera: Option<&'a crate::core::framework::render::ViewportCameraSnapshot>,
    viewport_size: [u32; 2],
    cull_input: Option<crate::core::framework::render::RenderVirtualGeometryCullInputSnapshot>,
    execution_inputs:
        Option<&'a crate::core::framework::render::RenderVirtualGeometryExecutionInputs>,
}

impl<'a> VirtualGeometryRuntimePrepareInput<'a> {
    pub fn new(
        extract: Option<&'a RenderVirtualGeometryExtract>,
        page_upload_plan: Option<&'a VisibilityVirtualGeometryPageUploadPlan>,
        visible_clusters: &'a [VisibilityVirtualGeometryCluster],
        visibility_draw_segments: &'a [VisibilityVirtualGeometryDrawSegment],
        generation: u64,
    ) -> Self {
        Self {
            input: RuntimeProviderPrepareInput::new(extract, generation),
            page_upload_plan,
            visible_clusters,
            visibility_draw_segments,
            camera: None,
            viewport_size: [0, 0],
            cull_input: None,
            execution_inputs: None,
        }
    }

    pub fn with_cpu_frame_context(
        mut self,
        camera: &'a crate::core::framework::render::ViewportCameraSnapshot,
        viewport_size: [u32; 2],
        cull_input: Option<crate::core::framework::render::RenderVirtualGeometryCullInputSnapshot>,
        execution_inputs: &'a crate::core::framework::render::RenderVirtualGeometryExecutionInputs,
    ) -> Self {
        self.camera = Some(camera);
        self.viewport_size = viewport_size;
        self.cull_input = cull_input;
        self.execution_inputs = Some(execution_inputs);
        self
    }
    pub fn camera(&self) -> Option<&crate::core::framework::render::ViewportCameraSnapshot> {
        self.camera
    }
    pub fn viewport_size(&self) -> [u32; 2] {
        self.viewport_size
    }
    pub fn cull_input(
        &self,
    ) -> Option<&crate::core::framework::render::RenderVirtualGeometryCullInputSnapshot> {
        self.cull_input.as_ref()
    }
    pub fn execution_inputs(
        &self,
    ) -> Option<&crate::core::framework::render::RenderVirtualGeometryExecutionInputs> {
        self.execution_inputs
    }

    pub fn extract(&self) -> Option<&RenderVirtualGeometryExtract> {
        self.input.extract()
    }

    pub fn page_upload_plan(&self) -> Option<&VisibilityVirtualGeometryPageUploadPlan> {
        self.page_upload_plan
    }

    pub fn visible_clusters(&self) -> &[VisibilityVirtualGeometryCluster] {
        self.visible_clusters
    }

    pub fn visibility_draw_segments(&self) -> &[VisibilityVirtualGeometryDrawSegment] {
        self.visibility_draw_segments
    }

    pub fn generation(&self) -> u64 {
        self.input.generation()
    }
}
