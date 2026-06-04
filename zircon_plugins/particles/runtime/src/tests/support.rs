use zircon_runtime::core::math::{Vec3, Vec4};
use zircon_runtime::render_graph::RenderGraphResourceKind;

use crate::{
    ParticleColorKey, ParticleEmitterAsset, ParticleScalarKey, ParticleScalarRange,
    ParticleSystemAsset,
};

pub(super) fn assert_approx_eq(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 1.0e-4,
        "expected {actual} to be approximately {expected}"
    );
}

pub(super) fn spawn_rate_asset(spawn_rate: f32, max_particles: u32) -> ParticleSystemAsset {
    ParticleSystemAsset::new("spawn_rate").with_emitters(vec![ParticleEmitterAsset::sprite(
        "sparks",
    )
    .with_spawn_rate(spawn_rate)
    .with_max_particles(max_particles)
    .with_lifetime(ParticleScalarRange::constant(2.0))
    .with_size_over_lifetime(vec![
        ParticleScalarKey::new(0.0, 1.0),
        ParticleScalarKey::new(1.0, 0.5),
    ])
    .with_color_over_lifetime(vec![
        ParticleColorKey::new(0.0, Vec4::new(1.0, 0.5, 0.1, 1.0)),
        ParticleColorKey::new(1.0, Vec4::new(1.0, 0.1, 0.0, 0.0)),
    ])])
}

pub(super) fn graph_resource_kind(
    kind: zircon_runtime::graphics::RenderFeatureResourceKind,
) -> RenderGraphResourceKind {
    match kind {
        zircon_runtime::graphics::RenderFeatureResourceKind::Texture => {
            RenderGraphResourceKind::TransientTexture
        }
        zircon_runtime::graphics::RenderFeatureResourceKind::Buffer => {
            RenderGraphResourceKind::TransientBuffer
        }
        zircon_runtime::graphics::RenderFeatureResourceKind::External => {
            RenderGraphResourceKind::External
        }
    }
}
