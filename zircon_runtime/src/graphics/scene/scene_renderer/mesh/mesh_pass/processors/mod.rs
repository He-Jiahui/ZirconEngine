mod depth_prepass;
mod hit_proxy;
mod opaque_base;
mod shadow;
mod taa_reactive_mask;
#[cfg(test)]
mod tests;
mod transparent;
mod velocity;

pub(crate) use depth_prepass::DepthPrepassProcessor;
pub(crate) use hit_proxy::HitProxyPassProcessor;
pub(crate) use opaque_base::OpaqueBasePassProcessor;
pub(crate) use shadow::ShadowPassProcessor;
pub(crate) use taa_reactive_mask::TaaReactiveMaskPassProcessor;
pub(crate) use transparent::TransparentPassProcessor;
pub(crate) use velocity::VelocityPassProcessor;
