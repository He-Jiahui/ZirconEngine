use crate::core::framework::scene::EntityId;
use crate::core::math::UVec2;
use std::sync::Arc;

use super::super::{
    AntiAliasSettings, CameraRenderDescriptor, CorePipelineKind, RenderCameraOrderReport,
    RenderCameraTarget, RenderLayerSet, RenderResolutionPolicy, RenderUpscalerKind,
    RenderViewFamilyPipeline, RenderViewportRect, ViewportCameraSnapshot,
};
use super::camera_target_size_from_descriptor;

#[derive(Clone, Debug, PartialEq)]
pub struct RenderViewExtract {
    pub camera: ViewportCameraSnapshot,
    pub cameras: Vec<CameraRenderDescriptor>,
    pub scene_camera_entity: Option<EntityId>,
    pub scene_camera_order_report: Option<Arc<RenderCameraOrderReport>>,
    pub core_pipeline: CorePipelineKind,
    pub anti_alias: AntiAliasSettings,
    pub target_size: Option<UVec2>,
    view_family_pipeline: RenderViewFamilyPipeline,
}

impl RenderViewExtract {
    pub fn from_camera(camera: ViewportCameraSnapshot) -> Self {
        let core_pipeline = camera.core_pipeline_kind();
        let anti_alias = AntiAliasSettings::from_camera_msaa_samples(camera.msaa_samples);
        let descriptor = CameraRenderDescriptor::from_camera_payload(None, camera.clone());
        let target_size = camera_target_size_from_descriptor(Some(&descriptor));
        let mut view = Self {
            cameras: vec![descriptor],
            camera,
            scene_camera_entity: None,
            scene_camera_order_report: None,
            core_pipeline,
            anti_alias,
            target_size,
            view_family_pipeline: RenderViewFamilyPipeline::resolve(
                UVec2::new(1, 1),
                RenderResolutionPolicy::default(),
                RenderUpscalerKind::Spatial,
            ),
        };
        view.refresh_spatial_view_family_pipeline();
        view
    }

    pub fn with_scene_camera_order_report(
        mut self,
        scene_camera_entity: EntityId,
        camera_order_report: RenderCameraOrderReport,
    ) -> Self {
        self.scene_camera_entity = Some(scene_camera_entity);
        self.scene_camera_order_report = Some(Arc::new(camera_order_report));
        self
    }

    pub fn with_cameras(mut self, cameras: Vec<CameraRenderDescriptor>) -> Self {
        self.cameras = cameras;
        self.refresh_spatial_view_family_pipeline();
        self
    }

    pub fn select_camera_descriptor(&mut self, mut descriptor: CameraRenderDescriptor) {
        descriptor.apply_target_size(
            self.target_size
                .or_else(|| camera_target_size_from_descriptor(Some(&descriptor)))
                .unwrap_or_else(|| UVec2::new(1, 1)),
        );
        self.core_pipeline = descriptor.camera.core_pipeline_kind();
        self.camera = descriptor.camera.clone();
        self.scene_camera_entity = descriptor.entity;
        self.cameras = vec![descriptor];
        self.refresh_spatial_view_family_pipeline();
    }

    pub(super) fn for_camera_submission(&self, descriptor: CameraRenderDescriptor) -> Self {
        let mut submission = Self {
            camera: self.camera.clone(),
            cameras: Vec::with_capacity(1),
            scene_camera_entity: self.scene_camera_entity,
            scene_camera_order_report: self.scene_camera_order_report.clone(),
            core_pipeline: self.core_pipeline,
            anti_alias: self.anti_alias,
            target_size: self.target_size,
            view_family_pipeline: self.view_family_pipeline,
        };
        submission.select_camera_descriptor(descriptor);
        submission
    }

    pub fn with_selected_camera_descriptor(mut self, descriptor: CameraRenderDescriptor) -> Self {
        self.select_camera_descriptor(descriptor);
        self
    }

    pub fn selected_camera_descriptor(&self) -> Option<&CameraRenderDescriptor> {
        self.scene_camera_entity
            .and_then(|entity| {
                self.cameras
                    .iter()
                    .find(|camera| camera.entity == Some(entity))
            })
            .or_else(|| self.cameras.first())
    }

    pub fn selected_camera_descriptor_mut(&mut self) -> Option<&mut CameraRenderDescriptor> {
        if let Some(entity) = self.scene_camera_entity {
            if let Some(index) = self
                .cameras
                .iter()
                .position(|camera| camera.entity == Some(entity))
            {
                return self.cameras.get_mut(index);
            }
        }
        self.cameras.first_mut()
    }

    pub fn selected_camera_target(&self) -> &RenderCameraTarget {
        self.selected_camera_descriptor()
            .map(|camera| &camera.target)
            .expect("render view extract must carry a selected camera descriptor")
    }

    pub fn selected_camera_layers(&self) -> &RenderLayerSet {
        self.selected_camera_descriptor()
            .map(|camera| &camera.culling_mask)
            .expect("render view extract must carry a selected camera descriptor")
    }

    pub fn selected_camera_volume_layers(&self) -> &RenderLayerSet {
        self.selected_camera_descriptor()
            .map(|camera| &camera.volume_mask)
            .expect("render view extract must carry a selected camera descriptor")
    }

    pub fn selected_effective_camera(&self) -> ViewportCameraSnapshot {
        self.selected_camera_descriptor()
            .map(CameraRenderDescriptor::as_effective_camera)
            .unwrap_or_else(|| self.camera.clone())
    }

    pub fn sync_selected_descriptor_camera_payload(&mut self) {
        let camera_payload = self.camera.clone();
        if let Some(camera) = self.selected_camera_descriptor_mut() {
            camera.camera = camera_payload;
            self.camera = camera.camera.clone();
        }
    }

    pub fn apply_target_size(&mut self, target_size: UVec2) {
        self.target_size = Some(target_size);
        self.sync_selected_descriptor_camera_payload();
        if let Some(camera) = self.selected_camera_descriptor_mut() {
            camera.apply_target_size(target_size);
            self.camera = camera.camera.clone();
        } else {
            self.camera.apply_viewport_size(target_size);
        }
        self.refresh_spatial_view_family_pipeline();
    }

    pub fn effective_view_size(&self) -> UVec2 {
        let target_size = self
            .target_size
            .or_else(|| camera_target_size_from_descriptor(self.selected_camera_descriptor()))
            .unwrap_or_else(|| UVec2::new(1, 1));
        self.selected_camera_descriptor()
            .map(|camera| camera.effective_viewport_size(target_size))
            .unwrap_or_else(|| self.camera.effective_viewport_size(target_size))
    }

    pub fn effective_render_size(&self) -> UVec2 {
        let target_size = self
            .target_size
            .or_else(|| camera_target_size_from_descriptor(self.selected_camera_descriptor()))
            .unwrap_or_else(|| UVec2::new(1, 1));
        self.selected_camera_descriptor()
            .map(|camera| camera.effective_render_size(target_size))
            .unwrap_or_else(|| self.camera.effective_render_size(target_size))
    }

    /// Installs the renderer-owned resolution decision used by graph allocation and execution.
    pub fn apply_view_family_pipeline(&mut self, pipeline: RenderViewFamilyPipeline) {
        self.view_family_pipeline = pipeline;
    }

    pub const fn view_family_pipeline(&self) -> &RenderViewFamilyPipeline {
        &self.view_family_pipeline
    }

    fn refresh_spatial_view_family_pipeline(&mut self) {
        let target_size = self
            .target_size
            .or_else(|| camera_target_size_from_descriptor(self.selected_camera_descriptor()))
            .unwrap_or_else(|| UVec2::new(1, 1));
        let descriptor = self.selected_camera_descriptor();
        let camera = descriptor
            .map(CameraRenderDescriptor::as_effective_camera)
            .unwrap_or_else(|| self.camera.clone());
        let display_viewport = descriptor
            .and_then(|descriptor| descriptor.viewport_rect)
            .map(|viewport| viewport.clamped_to_size(target_size))
            .unwrap_or_else(|| RenderViewportRect::new(UVec2::ZERO, target_size));
        self.view_family_pipeline = RenderViewFamilyPipeline::resolve_for_viewport(
            target_size,
            display_viewport,
            RenderResolutionPolicy::with_spatial_primary_fraction(
                camera.dynamic_resolution.clamped_scale(),
            ),
            RenderUpscalerKind::Spatial,
        );
    }
}

impl From<ViewportCameraSnapshot> for RenderViewExtract {
    fn from(camera: ViewportCameraSnapshot) -> Self {
        Self::from_camera(camera)
    }
}
