//! 高级光照的帧级数据和纯选择规则位于这里；资源驻留、通道编排及降级由 graphics 层消费后决定。
//! 保持此边界可让场景提取和资产序列化不依赖 WGPU 资源生命周期。
#[path = "cookie.rs"]
mod cookie;
#[path = "extract.rs"]
mod extract;
#[path = "irradiance_volume.rs"]
mod irradiance_volume;
#[path = "material_features.rs"]
mod material_features;
#[path = "material_usage.rs"]
mod material_usage;
#[path = "oit.rs"]
mod oit;
#[path = "planar.rs"]
mod planar;
#[path = "../../../render/advanced_lighting/screen_space_transmission.rs"]
mod screen_space_transmission;
#[path = "subsurface.rs"]
mod subsurface;
#[path = "volumetric.rs"]
mod volumetric;

pub use cookie::{CookieProjection, CookieWrapMode, LightCookieData};
pub use extract::AdvancedLightingExtract;
pub use irradiance_volume::{
    select_irradiance_volume, select_irradiance_volume_for_view, IrradianceVolumeData,
};
pub use material_features::{
    StandardPbrMaterialFeatures, STANDARD_PBR_DEFAULT_CLEARCOAT_ROUGHNESS,
    STANDARD_PBR_DEFAULT_IOR, STANDARD_PBR_NO_ATTENUATION_DISTANCE,
    STANDARD_PBR_TRANSMISSION_RENDER_QUEUE,
};
pub use material_usage::AdvancedPbrMaterialFrameUsage;
pub use oit::{
    oit_support, resolve_oit_fragments, OitBufferPlan, OitCapabilityProfile, OitFragment,
    OitResolveResult, OitSettings, OitSupport, OIT_GPU_COUNT_SIZE_BYTES, OIT_GPU_LAYER_SIZE_BYTES,
    OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
};
pub use planar::{
    derive_planar_reflection_camera, planar_oblique_near_clip_projection, planar_reflection_matrix,
    PlanarReflectionProbeData, PlanarReflectionQuality, PlanarReflectionUpdateState,
    PlanarUpdateMode,
};
pub use screen_space_transmission::{
    ScreenSpaceTransmissionSettings, MAX_SCREEN_SPACE_TRANSMISSION_STEPS,
};
pub use subsurface::{
    burley_radial_pdf, resolve_subsurface_profile_table, SubsurfaceProfileData,
    SubsurfaceProfileDiagnostic, SubsurfaceProfileTable, ZR_SSS_BURLEY_SAMPLE_COUNT,
    ZR_SSS_MAX_PROFILES,
};
pub use volumetric::{
    henyey_greenstein_phase, integrate_volumetric_step, FogVolumeData, FroxelGridParams,
    FroxelGridQuality, VolumetricFogSettings, VolumetricIntegrationStep,
    VOLUMETRIC_FOG_COMPONENT_ID, VOLUMETRIC_FOG_VOLUME_COMPONENT,
};
