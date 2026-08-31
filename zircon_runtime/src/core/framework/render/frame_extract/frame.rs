use std::collections::BTreeSet;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use crate::core::framework::scene::Mobility;
use crate::core::math::UVec2;

use super::super::{
    AdvancedLightingExtract, CameraRenderDescriptor, RenderBloomSettings,
    RenderColorGradingSettings, RenderExposureSettings, RenderFramePhaseQueueSummary,
    RenderSceneGeometryExtract, RenderSceneSnapshot, SpriteExtract,
};
use super::{
    DebugOverlayExtract, GeometryExtract, LightingExtract, ParticleExtract, PostProcessExtract,
    RenderFrameScenePayload, RenderViewExtract, RenderWorldSnapshotHandle, VisibilityInput,
    VisibilityRenderableInput,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderFrameTiming {
    outer_frame_index: u64,
    raw_real_delta_seconds: f32,
}

impl RenderFrameTiming {
    pub fn new(outer_frame_index: u64, raw_real_delta_seconds: f32) -> Self {
        Self {
            outer_frame_index,
            raw_real_delta_seconds: if raw_real_delta_seconds.is_finite() {
                raw_real_delta_seconds.max(0.0)
            } else {
                0.0
            },
        }
    }

    pub const fn outer_frame_index(self) -> u64 {
        self.outer_frame_index
    }

    pub const fn raw_real_delta_seconds(self) -> f32 {
        self.raw_real_delta_seconds
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderFrameExtract {
    scene: Arc<RenderFrameScenePayload>,
    pub timing: RenderFrameTiming,
    pub view: RenderViewExtract,
}

impl RenderFrameExtract {
    pub fn new(
        scene: RenderFrameScenePayload,
        view: RenderViewExtract,
        timing: RenderFrameTiming,
    ) -> Self {
        Self {
            scene: Arc::new(scene),
            timing,
            view,
        }
    }

    pub fn from_shared_scene(
        scene: Arc<RenderFrameScenePayload>,
        view: RenderViewExtract,
        timing: RenderFrameTiming,
    ) -> Self {
        Self {
            scene,
            timing,
            view,
        }
    }

    pub fn shared_scene(&self) -> Arc<RenderFrameScenePayload> {
        Arc::clone(&self.scene)
    }

    pub fn shares_scene_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.scene, &other.scene)
    }

    /// Builds a frame DTO from the scene viewport snapshot packet for preview,
    /// roundtrip, and synthetic validation paths. Scene production producers
    /// should build `RenderFrameScenePayload` and combine it with the owned
    /// view/timing overlay because this adapter cannot recover advanced
    /// sidebands such as sprites, particles, VG payloads, or level-owned
    /// animation poses from a `SceneViewportRenderPacket`.
    pub fn from_snapshot(world: RenderWorldSnapshotHandle, snapshot: RenderSceneSnapshot) -> Self {
        let RenderSceneGeometryExtract {
            camera,
            meshes,
            directional_lights,
            point_lights,
            spot_lights,
            ambient_lights,
            rect_lights,
        } = snapshot.scene;
        let camera_core_pipeline = camera.core_pipeline_kind();
        let camera_exposure_ev100 = camera.exposure_ev100;
        let renderables = meshes
            .iter()
            .map(|mesh| VisibilityRenderableInput {
                entity: mesh.node_id,
                stable_instance_key: mesh.stable_instance_key,
                mobility: mesh.mobility,
                render_layer_mask: mesh.common.layer_mask.clone(),
            })
            .collect::<Vec<_>>();
        let renderable_entities = renderables
            .iter()
            .map(|entry| entry.entity)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let static_entities = renderables
            .iter()
            .filter(|entry| entry.mobility == Mobility::Static)
            .map(|entry| entry.entity)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let dynamic_entities = renderables
            .iter()
            .filter(|entry| entry.mobility == Mobility::Dynamic)
            .map(|entry| entry.entity)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        let view = RenderViewExtract::from_camera(camera);
        let scene = RenderFrameScenePayload::new(
            world,
            {
                let mut geometry = GeometryExtract::from_meshes(camera_core_pipeline, meshes);
                geometry.virtual_geometry_debug = snapshot.virtual_geometry_debug;
                geometry
            },
            Vec::new(),
            LightingExtract {
                directional_lights,
                point_lights,
                spot_lights,
                ambient_lights,
                rect_lights,
                hybrid_global_illumination: None,
                advanced_lighting: AdvancedLightingExtract::default(),
            },
            snapshot.environment,
            {
                let mut post_process = PostProcessExtract::from_parts(
                    snapshot.preview,
                    snapshot.overlays.display_mode,
                    RenderBloomSettings::default(),
                    RenderColorGradingSettings::default(),
                    false,
                    false,
                );
                post_process.exposure = RenderExposureSettings::manual_ev100(camera_exposure_ev100);
                post_process
            },
            DebugOverlayExtract {
                overlays: snapshot.overlays,
            },
            SpriteExtract::default(),
            ParticleExtract::default(),
            VisibilityInput {
                renderable_entities,
                static_entities,
                dynamic_entities,
                renderables,
            },
        );
        Self::new(scene, view, RenderFrameTiming::default())
    }

    pub fn to_scene_snapshot(&self) -> RenderSceneSnapshot {
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: self.view.camera.clone(),
                meshes: self.geometry.meshes.clone(),
                directional_lights: self.lighting.directional_lights.clone(),
                point_lights: self.lighting.point_lights.clone(),
                spot_lights: self.lighting.spot_lights.clone(),
                ambient_lights: self.lighting.ambient_lights.clone(),
                rect_lights: self.lighting.rect_lights.clone(),
            },
            overlays: self.debug.overlays.clone(),
            environment: self.environment.as_ref().clone(),
            preview: self.post_process.preview.clone(),
            virtual_geometry_debug: self.geometry.virtual_geometry_debug,
        }
    }

    pub fn apply_viewport_size(&mut self, viewport_size: UVec2) {
        self.view.apply_target_size(viewport_size);
    }

    pub fn with_viewport_size(mut self, viewport_size: UVec2) -> Self {
        self.apply_viewport_size(viewport_size);
        self
    }

    pub fn with_timing(mut self, timing: RenderFrameTiming) -> Self {
        self.timing = timing;
        self
    }

    pub fn set_timing(&mut self, timing: RenderFrameTiming) {
        self.timing = timing;
    }

    pub fn with_selected_camera_descriptor(mut self, descriptor: CameraRenderDescriptor) -> Self {
        self.select_camera_descriptor(descriptor);
        self
    }

    pub fn select_camera_descriptor(&mut self, descriptor: CameraRenderDescriptor) {
        self.view.select_camera_descriptor(descriptor);
    }

    pub fn for_camera_submission(&self, descriptor: CameraRenderDescriptor) -> Self {
        Self {
            scene: Arc::clone(&self.scene),
            timing: self.timing,
            view: self.view.for_camera_submission(descriptor),
        }
    }

    /// Builds a diagnostics summary for the frame's mesh and sprite phase queues.
    pub fn phase_queue_summary(&self) -> RenderFramePhaseQueueSummary {
        RenderFramePhaseQueueSummary::new(
            self.geometry.phase_queue_summary(),
            self.sprites.phase_queue_summary(),
        )
    }
}

impl Deref for RenderFrameExtract {
    type Target = RenderFrameScenePayload;

    fn deref(&self) -> &Self::Target {
        self.scene.as_ref()
    }
}

impl DerefMut for RenderFrameExtract {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.scene)
    }
}
