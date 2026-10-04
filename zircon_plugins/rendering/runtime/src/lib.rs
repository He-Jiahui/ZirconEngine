//! 渲染可选特性的身份和清单归属中心；具体图贡献及设备资源由特性 crate 与图形宿主负责。
pub const RENDERING_MODULE_NAME: &str = "rendering.runtime";

mod capability;
mod plugin;

pub use capability::{
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST, PLUGIN_ID, RENDERING_DECLARATION,
    RENDERING_RUNTIME_CAPABILITY, RUNTIME_CAPABILITIES,
};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_plugin,
    runtime_plugin_descriptor, RenderingRuntimePlugin, RENDERING_DIST_CRATE_NAME,
    RENDERING_DIST_RUNTIME_ENTRY,
};

/// 主包认识的特性集合，用于一致生成跨端 crate 名称、能力键和默认项目选择。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderingFeatureKind {
    PostProcess,
    Ssao,
    ContactShadow,
    VolumetricFog,
    Oit,
    LightCookies,
    IrradianceVolumes,
    PlanarReflections,
    SubsurfaceScattering,
    Decals,
    ReflectionProbes,
    BakedLighting,
    RayTracingPolicy,
    ShaderGraph,
    VfxGraph,
}

/// 主包描述符遍历的声明集合；新增特性还需同步内建目录与对应模块的注册入口。
pub const RENDERING_FEATURES: &[RenderingFeatureKind] = &[
    RenderingFeatureKind::PostProcess,
    RenderingFeatureKind::Ssao,
    RenderingFeatureKind::ContactShadow,
    RenderingFeatureKind::VolumetricFog,
    RenderingFeatureKind::Oit,
    RenderingFeatureKind::LightCookies,
    RenderingFeatureKind::IrradianceVolumes,
    RenderingFeatureKind::PlanarReflections,
    RenderingFeatureKind::SubsurfaceScattering,
    RenderingFeatureKind::Decals,
    RenderingFeatureKind::ReflectionProbes,
    RenderingFeatureKind::BakedLighting,
    RenderingFeatureKind::RayTracingPolicy,
    RenderingFeatureKind::ShaderGraph,
    RenderingFeatureKind::VfxGraph,
];

// 身份生成规则供清单与链接式导出消费；后缀变化会同时改变包选择、能力和 crate 定位。
impl RenderingFeatureKind {
    pub const fn id_suffix(self) -> &'static str {
        match self {
            Self::PostProcess => "post_process",
            Self::Ssao => "ssao",
            Self::ContactShadow => "contact_shadow",
            Self::VolumetricFog => "volumetric_fog",
            Self::Oit => "oit",
            Self::LightCookies => "light_cookies",
            Self::IrradianceVolumes => "irradiance_volumes",
            Self::PlanarReflections => "planar_reflections",
            Self::SubsurfaceScattering => "subsurface_scattering",
            Self::Decals => "decals",
            Self::ReflectionProbes => "reflection_probes",
            Self::BakedLighting => "baked_lighting",
            Self::RayTracingPolicy => "ray_tracing_policy",
            Self::ShaderGraph => "shader_graph",
            Self::VfxGraph => "vfx_graph",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::PostProcess => "Post Process",
            Self::Ssao => "SSAO",
            Self::ContactShadow => "Contact Shadow",
            Self::VolumetricFog => "Volumetric Fog",
            Self::Oit => "Order Independent Transparency",
            Self::LightCookies => "Light Cookies",
            Self::IrradianceVolumes => "Irradiance Volumes",
            Self::PlanarReflections => "Planar Reflections",
            Self::SubsurfaceScattering => "Subsurface Scattering",
            Self::Decals => "Decals",
            Self::ReflectionProbes => "Reflection Probes",
            Self::BakedLighting => "Baked Lighting",
            Self::RayTracingPolicy => "Ray Tracing Policy",
            Self::ShaderGraph => "Shader Graph",
            Self::VfxGraph => "VFX Graph",
        }
    }

    /// 返回项目选择的默认值；它不保证特性实现、后端能力或每帧数据已满足执行条件。
    pub const fn enabled_by_default(self) -> bool {
        matches!(
            self,
            Self::PostProcess | Self::ReflectionProbes | Self::BakedLighting
        )
    }

    pub fn feature_id(self) -> String {
        format!("rendering.{}", self.id_suffix())
    }

    pub fn runtime_capability(self) -> String {
        format!("runtime.feature.rendering.{}", self.id_suffix())
    }

    pub fn editor_capability(self) -> String {
        format!("editor.feature.rendering.{}", self.id_suffix())
    }

    pub fn runtime_crate(self) -> String {
        format!("zircon_plugin_rendering_{}_runtime", self.id_suffix())
    }

    pub fn editor_crate(self) -> String {
        format!("zircon_plugin_rendering_{}_editor", self.id_suffix())
    }
}

/// 提供核心运行时所需的主包模块身份；可选特性贡献通过各自注册报告安装。
pub fn module_descriptor() -> zircon_runtime::core::ModuleDescriptor {
    zircon_runtime::core::ModuleDescriptor::new(
        RENDERING_MODULE_NAME,
        "Rendering umbrella plugin and feature owner",
    )
}

/// 为跨端发现与导出生成完整特性契约；运行时模块仅允许客户端及编辑器宿主目标。
pub fn feature_manifest(
    feature: RenderingFeatureKind,
) -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    let feature_id = feature.feature_id();
    let capability = feature.runtime_capability();
    let editor_capability = feature.editor_capability();
    let mut manifest = zircon_runtime::plugin::PluginFeatureBundleManifest::new(
        feature_id.clone(),
        feature.display_name(),
        PLUGIN_ID,
    )
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::primary(
        PLUGIN_ID,
        RENDERING_RUNTIME_CAPABILITY,
    ))
    .with_capability(capability.clone())
    .with_runtime_module(
        zircon_runtime::plugin::PluginModuleManifest::runtime(
            format!("{feature_id}.runtime"),
            feature.runtime_crate(),
        )
        .with_target_modes([
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities([capability]),
    )
    .with_editor_module(
        zircon_runtime::plugin::PluginModuleManifest::editor(
            format!("{feature_id}.editor"),
            feature.editor_crate(),
        )
        .with_capabilities([editor_capability]),
    )
    .enabled_by_default(feature.enabled_by_default());

    // 此特性的资源含粒子与着色器图引用，目录必须先解析两个额外能力依赖。
    if feature == RenderingFeatureKind::VfxGraph {
        manifest = manifest
            .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::required(
                "particles",
                "runtime.plugin.particles",
            ))
            .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::required(
                PLUGIN_ID,
                RenderingFeatureKind::ShaderGraph.runtime_capability(),
            ));
    }

    manifest
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
