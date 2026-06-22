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
        }
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
