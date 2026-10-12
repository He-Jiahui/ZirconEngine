#[path = "../../processors/depth_prepass.rs"]
mod depth_prepass;
#[path = "../../processors/hit_proxy.rs"]
mod hit_proxy;
#[path = "../../processors/opaque_base.rs"]
mod opaque_base;
#[path = "shadow.rs"]
mod shadow;
#[path = "../../processors/taa_reactive_mask.rs"]
mod taa_reactive_mask;
#[cfg(test)]
#[path = "../../tests/processors_cases.rs"]
mod tests;
#[path = "../../processors/transparent.rs"]
mod transparent;
#[path = "velocity.rs"]
mod velocity;

pub(crate) use depth_prepass::DepthPrepassProcessor;
pub(crate) use hit_proxy::HitProxyPassProcessor;
pub(crate) use opaque_base::OpaqueBasePassProcessor;
pub(crate) use shadow::ShadowPassProcessor;
pub(crate) use taa_reactive_mask::TaaReactiveMaskPassProcessor;
pub(crate) use transparent::TransparentPassProcessor;
pub(crate) use velocity::VelocityPassProcessor;
