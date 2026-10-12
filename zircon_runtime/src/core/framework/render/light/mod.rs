#[path = "gpu_light.rs"]
mod gpu_light;
#[path = "readiness.rs"]
mod readiness;
#[path = "shadow_settings.rs"]
mod shadow_settings;
#[path = "snapshots.rs"]
mod snapshots;

pub use gpu_light::{GpuLightData, GpuLightType, GPU_LIGHT_DATA_STRIDE, SHADOW_SLOT_NONE};
pub use readiness::{RenderLightFamilyReadiness, RenderLightReadinessReport};
pub use shadow_settings::{LightShadowSettings, ShadowPcfQuality, ShadowResolutionTier};
pub use snapshots::{
    RenderAmbientLightSnapshot, RenderBakedLightingExtract, RenderDirectionalLightSnapshot,
    RenderPointLightSnapshot, RenderRectLightSnapshot, RenderSpotLightSnapshot,
};
