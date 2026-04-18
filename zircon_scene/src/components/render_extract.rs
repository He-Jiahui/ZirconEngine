use zircon_math::{Real, Transform, Vec3, Vec4};
use zircon_resource::{MaterialMarker, ModelMarker, ResourceHandle};

use crate::EntityId;

use super::{FallbackSkyboxKind, Mobility, RenderOverlayExtract, ViewportCameraSnapshot};

#[derive(Clone, Debug, PartialEq)]
pub struct RenderMeshSnapshot {
    pub node_id: EntityId,
    pub transform: Transform,
    pub model: ResourceHandle<ModelMarker>,
    pub material: ResourceHandle<MaterialMarker>,
    pub tint: Vec4,
    pub mobility: Mobility,
    pub render_layer_mask: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderDirectionalLightSnapshot {
    pub node_id: EntityId,
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: Real,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderBloomSettings {
    pub threshold: Real,
    pub intensity: Real,
    pub radius: Real,
}

impl Default for RenderBloomSettings {
    fn default() -> Self {
        Self {
            threshold: 1.0,
            intensity: 0.0,
            radius: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderColorGradingSettings {
    pub exposure: Real,
    pub contrast: Real,
    pub saturation: Real,
    pub gamma: Real,
    pub tint: Vec3,
}

impl Default for RenderColorGradingSettings {
    fn default() -> Self {
        Self {
            exposure: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            gamma: 1.0,
            tint: Vec3::ONE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderReflectionProbeSnapshot {
    pub position: Vec3,
    pub radius: Real,
    pub color: Vec3,
    pub intensity: Real,
}

impl Default for RenderReflectionProbeSnapshot {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            radius: 0.0,
            color: Vec3::ZERO,
            intensity: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderBakedLightingExtract {
    pub color: Vec3,
    pub intensity: Real,
}

impl Default for RenderBakedLightingExtract {
    fn default() -> Self {
        Self {
            color: Vec3::ZERO,
            intensity: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderVirtualGeometryCluster {
    pub entity: EntityId,
    pub cluster_id: u32,
    pub page_id: u32,
    pub lod_level: u8,
    pub parent_cluster_id: Option<u32>,
    pub bounds_center: Vec3,
    pub bounds_radius: Real,
    pub screen_space_error: Real,
}

impl Default for RenderVirtualGeometryCluster {
    fn default() -> Self {
        Self {
            entity: 0,
            cluster_id: 0,
            page_id: 0,
            lod_level: 0,
            parent_cluster_id: None,
            bounds_center: Vec3::ZERO,
            bounds_radius: 0.0,
            screen_space_error: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryPage {
    pub page_id: u32,
    pub resident: bool,
    pub size_bytes: u64,
}

impl Default for RenderVirtualGeometryPage {
    fn default() -> Self {
        Self {
            page_id: 0,
            resident: false,
            size_bytes: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryExtract {
    pub cluster_budget: u32,
    pub page_budget: u32,
    pub clusters: Vec<RenderVirtualGeometryCluster>,
    pub pages: Vec<RenderVirtualGeometryPage>,
}

impl Default for RenderVirtualGeometryExtract {
    fn default() -> Self {
        Self {
            cluster_budget: 0,
            page_budget: 0,
            clusters: Vec::new(),
            pages: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderHybridGiProbe {
    pub entity: EntityId,
    pub probe_id: u32,
    pub position: Vec3,
    pub radius: Real,
    pub parent_probe_id: Option<u32>,
    pub resident: bool,
    pub ray_budget: u32,
}

impl Default for RenderHybridGiProbe {
    fn default() -> Self {
        Self {
            entity: 0,
            probe_id: 0,
            position: Vec3::ZERO,
            radius: 0.0,
            parent_probe_id: None,
            resident: false,
            ray_budget: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderHybridGiTraceRegion {
    pub entity: EntityId,
    pub region_id: u32,
    pub bounds_center: Vec3,
    pub bounds_radius: Real,
    pub screen_coverage: Real,
    pub rt_lighting_rgb: [u8; 3],
}

impl Default for RenderHybridGiTraceRegion {
    fn default() -> Self {
        Self {
            entity: 0,
            region_id: 0,
            bounds_center: Vec3::ZERO,
            bounds_radius: 0.0,
            screen_coverage: 0.0,
            rt_lighting_rgb: [0, 0, 0],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderHybridGiExtract {
    pub probe_budget: u32,
    pub tracing_budget: u32,
    pub probes: Vec<RenderHybridGiProbe>,
    pub trace_regions: Vec<RenderHybridGiTraceRegion>,
}

impl Default for RenderHybridGiExtract {
    fn default() -> Self {
        Self {
            probe_budget: 0,
            tracing_budget: 0,
            probes: Vec::new(),
            trace_regions: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderParticleSpriteSnapshot {
    pub entity: EntityId,
    pub position: Vec3,
    pub size: Real,
    pub color: Vec4,
    pub intensity: Real,
}

impl Default for RenderParticleSpriteSnapshot {
    fn default() -> Self {
        Self {
            entity: 0,
            position: Vec3::ZERO,
            size: 0.0,
            color: Vec4::ZERO,
            intensity: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreviewEnvironmentExtract {
    pub lighting_enabled: bool,
    pub skybox_enabled: bool,
    pub fallback_skybox: FallbackSkyboxKind,
    pub clear_color: Vec4,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderSceneGeometryExtract {
    pub camera: ViewportCameraSnapshot,
    pub meshes: Vec<RenderMeshSnapshot>,
    pub lights: Vec<RenderDirectionalLightSnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneViewportRenderPacket {
    pub scene: RenderSceneGeometryExtract,
    pub overlays: RenderOverlayExtract,
    pub preview: PreviewEnvironmentExtract,
}

pub type RenderExtractPacket = SceneViewportRenderPacket;
pub type RenderSceneSnapshot = SceneViewportRenderPacket;
