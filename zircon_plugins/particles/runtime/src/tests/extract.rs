use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::core::resource::{MaterialMarker, ResourceHandle, ResourceId, TextureMarker};

use crate::{
    ParticleBurst, ParticleEmitterAsset, ParticleScalarRange, ParticleShape, ParticleSystemAsset,
    ParticleSystemComponent, ParticlesManager,
};

use super::support::assert_approx_eq;

#[test]
fn extract_sorts_sprites_back_to_front_when_camera_is_known() {
    let manager = ParticlesManager::default();
    let asset =
        ParticleSystemAsset::new("burst").with_emitters(vec![ParticleEmitterAsset::sprite(
            "burst",
        )
        .with_spawn_rate(0.0)
        .with_burst(ParticleBurst::new(0.0, 2))
        .with_shape(ParticleShape::Box {
            half_extents: Vec3::new(0.0, 0.0, 2.0),
        })]);
    manager
        .instantiate(
            ParticleSystemComponent::new(9, asset)
                .with_transform(Transform::from_translation(Vec3::new(0.0, 0.0, 5.0))),
        )
        .unwrap();
    manager.tick(0.001).unwrap();

    let extract = manager.build_extract(Some(Vec3::ZERO));

    assert_eq!(extract.emitters, vec![9]);
    assert_eq!(extract.sprites.len(), 2);
    let first_distance = extract.sprites[0].position.length_squared();
    let second_distance = extract.sprites[1].position.length_squared();
    assert!(first_distance >= second_distance);
}

#[test]
fn cpu_extract_preserves_material_texture_rotation_bounds_and_sort_metadata() {
    let material = ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
        "particles/material/spark",
    ));
    let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "particles/texture/spark",
    ));
    let mut emitter = ParticleEmitterAsset::sprite("metadata")
        .with_spawn_rate(0.0)
        .with_burst(ParticleBurst::new(0.0, 1))
        .with_initial_rotation(ParticleScalarRange::constant(0.5))
        .with_initial_angular_velocity(ParticleScalarRange::constant(2.0))
        .with_material(material)
        .with_texture(texture);
    emitter.initial_size = ParticleScalarRange::constant(2.0);
    let asset = ParticleSystemAsset::new("metadata").with_emitters(vec![emitter]);
    let manager = ParticlesManager::default();
    manager
        .instantiate(ParticleSystemComponent::new(33, asset))
        .unwrap();

    manager.tick(0.25).unwrap();
    let snapshot = manager.snapshot();
    let sprite = snapshot.sprites.first().expect("one sprite should spawn");

    assert_eq!(sprite.material, Some(material));
    assert_eq!(sprite.texture, Some(texture));
    assert_approx_eq(sprite.rotation, 1.0);

    let extract = manager.build_extract(Some(Vec3::new(0.0, 0.0, -8.0)));
    assert_eq!(
        extract.sort_camera_position,
        Some(Vec3::new(0.0, 0.0, -8.0))
    );
    assert_eq!(extract.bounds.len(), 1);
    assert_eq!(extract.bounds[0].entity, 33);
    assert_eq!(extract.bounds[0].center, sprite.position);
    assert_approx_eq(extract.bounds[0].radius, 3.0_f32.sqrt());
    assert_eq!(extract.sprites[0].material, Some(material));
    assert_eq!(extract.sprites[0].texture, Some(texture));
    assert!(extract.sprites[0].stable_sprite_key > 0);
    assert_eq!(extract.sprites[0].aspect_ratio, 1.0);
    assert_eq!(
        extract.sprites[0].billboard_offset,
        zircon_runtime::core::math::Vec2::ZERO
    );
    assert_eq!(extract.sprites[0].sort_order, 0);
    assert!(extract.sprites[0].depth_test);
    assert!(extract.previous_sprites.is_empty());
}
