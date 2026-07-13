use zircon_runtime::core::framework::render::{
    GBufferChannelMask, ShadingModelDescriptor, ShadingModelId,
};
use zircon_runtime::graphics::{RenderFeatureDescriptor, RenderPassExecutorRegistration};

mod capability;
mod plugin;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    RenderingSubsurfaceScatteringRuntimeFeature,
};

pub const FEATURE_ID: &str = "rendering.subsurface_scattering";
pub const FEATURE_NAME: &str = "subsurface_scattering";
pub const SHADING_MODEL_ID: ShadingModelId = ShadingModelId::new(16);

pub const SETUP_PASS: &str = zircon_runtime::graphics::SSS_SETUP_EXECUTOR_ID;
pub const SCATTER_PASS: &str = zircon_runtime::graphics::SSS_SCATTER_EXECUTOR_ID;
pub const RECOMBINE_PASS: &str = zircon_runtime::graphics::SSS_RECOMBINE_EXECUTOR_ID;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubsurfacePipelineResolution {
    DeferredScattering,
    ForwardStandardPbrFallback { diagnostic: String },
}

pub fn resolve_subsurface_pipeline(deferred_enabled: bool) -> SubsurfacePipelineResolution {
    if deferred_enabled {
        SubsurfacePipelineResolution::DeferredScattering
    } else {
        SubsurfacePipelineResolution::ForwardStandardPbrFallback {
            diagnostic: "subsurface scattering requires deferred rendering; StandardPBR forward fallback selected"
                .to_string(),
        }
    }
}

pub fn shading_model_descriptor() -> ShadingModelDescriptor {
    ShadingModelDescriptor::new(
        SHADING_MODEL_ID,
        "custom:subsurface",
        "zr_shading_standard_pbr.wgsl",
        "zr_gbuffer_encode_subsurface.wgsl",
        "zr_shade_deferred_subsurface.wgsl",
        GBufferChannelMask::standard_lit(),
    )
}

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    zircon_runtime::graphics::subsurface_render_feature_descriptor()
}

pub fn render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    zircon_runtime::graphics::subsurface_render_pass_executor_registrations()
}

#[cfg(test)]
mod tests;
