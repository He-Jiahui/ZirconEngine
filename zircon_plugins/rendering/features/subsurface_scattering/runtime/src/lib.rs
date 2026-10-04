//! 次表面散射的运行时公共契约；特性提供者将此处元数据提交到目录与图编译。
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

/// 复用宿主的延迟散射图契约；配置表为空、前向管线或不兼容采样模式时由编译器回退。
pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    zircon_runtime::graphics::subsurface_render_feature_descriptor()
}

/// 提供与特性图匹配的执行实现；宿主负责实际 GPU 资源与这些句柄的设备生命周期。
pub fn render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    zircon_runtime::graphics::subsurface_render_pass_executor_registrations()
}

// 此测试边界覆盖声明与注册约束；GPU 效果证据需由对应产品测试另行提供。
#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
