use zircon_render_graph::{QueueLane, RenderGraphBuilder};
use zircon_scene::RenderFrameExtract;

use crate::pipeline::RenderPassStage;
use crate::{FrameHistoryBinding, FrameHistorySlot};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFeaturePassDescriptor {
    pub stage: RenderPassStage,
    pub pass_name: String,
    pub queue: QueueLane,
}

impl RenderFeaturePassDescriptor {
    pub fn new(stage: RenderPassStage, pass_name: impl Into<String>, queue: QueueLane) -> Self {
        Self {
            stage,
            pass_name: pass_name.into(),
            queue,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFeatureDescriptor {
    pub name: String,
    pub required_extract_sections: Vec<String>,
    pub history_bindings: Vec<FrameHistoryBinding>,
    pub stage_passes: Vec<RenderFeaturePassDescriptor>,
}

impl RenderFeatureDescriptor {
    pub fn new(
        name: impl Into<String>,
        required_extract_sections: Vec<String>,
        history_bindings: Vec<FrameHistoryBinding>,
        stage_passes: Vec<RenderFeaturePassDescriptor>,
    ) -> Self {
        Self {
            name: name.into(),
            required_extract_sections,
            history_bindings,
            stage_passes,
        }
    }
}

pub trait RenderFeature: Send + Sync {
    fn descriptor(&self) -> RenderFeatureDescriptor;

    fn register_passes(
        &self,
        _graph: &mut RenderGraphBuilder,
        _extract: &RenderFrameExtract,
    ) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltinRenderFeature {
    Mesh,
    DeferredGeometry,
    DeferredLighting,
    ClusteredLighting,
    ScreenSpaceAmbientOcclusion,
    Bloom,
    ColorGrading,
    ReflectionProbes,
    BakedLighting,
    HistoryResolve,
    Shadows,
    PostProcess,
    DebugOverlay,
    Particle,
    GlobalIllumination,
    RayTracing,
    VirtualGeometry,
}

impl BuiltinRenderFeature {
    pub const fn requires_explicit_opt_in(self) -> bool {
        matches!(
            self,
            Self::GlobalIllumination | Self::RayTracing | Self::VirtualGeometry
        )
    }

    pub fn descriptor(self) -> RenderFeatureDescriptor {
        match self {
            Self::Mesh => RenderFeatureDescriptor::new(
                "mesh",
                vec![
                    "view".to_string(),
                    "geometry".to_string(),
                    "visibility".to_string(),
                ],
                Vec::new(),
                vec![
                    RenderFeaturePassDescriptor::new(
                        RenderPassStage::DepthPrepass,
                        "depth-prepass",
                        QueueLane::Graphics,
                    ),
                    RenderFeaturePassDescriptor::new(
                        RenderPassStage::Opaque,
                        "opaque-mesh",
                        QueueLane::Graphics,
                    ),
                    RenderFeaturePassDescriptor::new(
                        RenderPassStage::Transparent,
                        "transparent-mesh",
                        QueueLane::Graphics,
                    ),
                ],
            ),
            Self::DeferredGeometry => RenderFeatureDescriptor::new(
                "deferred_geometry",
                vec![
                    "view".to_string(),
                    "geometry".to_string(),
                    "visibility".to_string(),
                ],
                Vec::new(),
                vec![
                    RenderFeaturePassDescriptor::new(
                        RenderPassStage::DepthPrepass,
                        "depth-prepass",
                        QueueLane::Graphics,
                    ),
                    RenderFeaturePassDescriptor::new(
                        RenderPassStage::GBuffer,
                        "gbuffer-mesh",
                        QueueLane::Graphics,
                    ),
                    RenderFeaturePassDescriptor::new(
                        RenderPassStage::Transparent,
                        "transparent-mesh",
                        QueueLane::Graphics,
                    ),
                ],
            ),
            Self::DeferredLighting => RenderFeatureDescriptor::new(
                "deferred_lighting",
                vec![
                    "view".to_string(),
                    "geometry".to_string(),
                    "lighting".to_string(),
                    "visibility".to_string(),
                ],
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::Lighting,
                    "deferred-lighting",
                    QueueLane::Graphics,
                )],
            ),
            Self::ClusteredLighting => RenderFeatureDescriptor::new(
                "clustered_lighting",
                vec![
                    "view".to_string(),
                    "lighting".to_string(),
                    "visibility".to_string(),
                ],
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::Lighting,
                    "clustered-light-culling",
                    QueueLane::AsyncCompute,
                )],
            ),
            Self::ScreenSpaceAmbientOcclusion => RenderFeatureDescriptor::new(
                "screen_space_ambient_occlusion",
                vec![
                    "view".to_string(),
                    "geometry".to_string(),
                    "visibility".to_string(),
                ],
                vec![FrameHistoryBinding::read_write(
                    FrameHistorySlot::AmbientOcclusion,
                )],
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::AmbientOcclusion,
                    "ssao-evaluate",
                    QueueLane::AsyncCompute,
                )],
            ),
            Self::Bloom => RenderFeatureDescriptor::new(
                "bloom",
                vec!["view".to_string(), "post_process".to_string()],
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "bloom-extract",
                    QueueLane::Graphics,
                )],
            ),
            Self::ColorGrading => RenderFeatureDescriptor::new(
                "color_grading",
                vec!["view".to_string(), "post_process".to_string()],
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "color-grade",
                    QueueLane::Graphics,
                )],
            ),
            Self::ReflectionProbes => RenderFeatureDescriptor::new(
                "reflection_probes",
                vec![
                    "view".to_string(),
                    "lighting".to_string(),
                    "post_process".to_string(),
                ],
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "reflection-probe-composite",
                    QueueLane::Graphics,
                )],
            ),
            Self::BakedLighting => RenderFeatureDescriptor::new(
                "baked_lighting",
                vec!["lighting".to_string(), "post_process".to_string()],
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "baked-lighting-composite",
                    QueueLane::Graphics,
                )],
            ),
            Self::HistoryResolve => RenderFeatureDescriptor::new(
                "history_resolve",
                vec!["view".to_string(), "post_process".to_string()],
                vec![FrameHistoryBinding::read_write(
                    FrameHistorySlot::SceneColor,
                )],
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "history-resolve",
                    QueueLane::Graphics,
                )],
            ),
            Self::Shadows => RenderFeatureDescriptor::new(
                "shadows",
                vec![
                    "view".to_string(),
                    "geometry".to_string(),
                    "lighting".to_string(),
                    "visibility".to_string(),
                ],
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::Shadow,
                    "shadow-map",
                    QueueLane::Graphics,
                )],
            ),
            Self::PostProcess => RenderFeatureDescriptor::new(
                "post_process",
                vec!["view".to_string(), "post_process".to_string()],
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "post-process",
                    QueueLane::Graphics,
                )],
            ),
            Self::DebugOverlay => RenderFeatureDescriptor::new(
                "debug_overlay",
                vec!["view".to_string(), "debug".to_string()],
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::Overlay,
                    "overlay-gizmo",
                    QueueLane::Graphics,
                )],
            ),
            Self::Particle => RenderFeatureDescriptor::new(
                "particle",
                vec![
                    "view".to_string(),
                    "particles".to_string(),
                    "visibility".to_string(),
                ],
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::Transparent,
                    "particle-render",
                    QueueLane::Graphics,
                )],
            ),
            Self::GlobalIllumination => RenderFeatureDescriptor::new(
                "global_illumination",
                vec![
                    "view".to_string(),
                    "lighting".to_string(),
                    "visibility".to_string(),
                ],
                vec![FrameHistoryBinding::read_write(
                    FrameHistorySlot::GlobalIllumination,
                )],
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::Lighting,
                    "hybrid-gi-resolve",
                    QueueLane::Graphics,
                )],
            ),
            Self::RayTracing => RenderFeatureDescriptor::new(
                "ray_tracing",
                vec![
                    "view".to_string(),
                    "geometry".to_string(),
                    "visibility".to_string(),
                ],
                Vec::new(),
                Vec::new(),
            ),
            Self::VirtualGeometry => RenderFeatureDescriptor::new(
                "virtual_geometry",
                vec![
                    "view".to_string(),
                    "geometry".to_string(),
                    "visibility".to_string(),
                ],
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::DepthPrepass,
                    "virtual-geometry-prepare",
                    QueueLane::Graphics,
                )],
            ),
        }
    }
}
