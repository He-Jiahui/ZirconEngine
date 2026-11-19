use zircon_plugin_particles_editor::ParticleSystemSource;
use zircon_plugin_particles_runtime::{
    ParticleSimulationBackend, ParticleSystemComponent, ParticlesManager,
};
use zircon_runtime::core::framework::render::{
    ParticleExtract, RenderParticlePreviousSpriteSnapshot,
};
use zircon_runtime::core::math::{Real, Vec3};

const ENTITY: u64 = 17;

fn authored_cpu(times: &[Real], lifetime: Real) -> ParticlesManager {
    let bursts = times
        .iter()
        .enumerate()
        .map(|(i, time)| format!(r#"{{ key = "burst-{i}", time_seconds = {time:?}, count = 1 }}"#))
        .collect::<Vec<_>>()
        .join(", ");
    let size_at_end = 1.0 + lifetime;
    let document = format!(
        r#"
[system]
id = "birth-check"
backend = "Cpu"
seed = 7
looped = false

[[emitters]]
key = "point-emitter"
max_particles = 8
spawn_rate_per_second = 0.0
bursts = [{bursts}]
lifetime = {{ min = {lifetime:?}, max = {lifetime:?} }}
initial_size = {{ min = 1.0, max = 1.0 }}
initial_velocity = {{ min = [8.0, 0.0, 0.0], max = [8.0, 0.0, 0.0] }}
gravity = [0.0, 0.0, 0.0]
drag = 0.0
shape = {{ kind = "Point" }}
color_over_lifetime = []
size_over_lifetime = [
    {{ key = "size-start", t = 0.0, value = 1.0 }},
    {{ key = "size-end", t = 1.0, value = {size_at_end:?} }},
]
"#
    );
    let decoded = ParticleSystemSource::decode_toml(&document).expect("valid authored source");
    assert_eq!(decoded.assigned_keys, 0);
    let asset = decoded.source.to_cpu_asset().expect("validated CPU asset");
    let manager = ParticlesManager::default();
    manager
        .instantiate(ParticleSystemComponent::new(ENTITY, asset))
        .expect("CPU system instance");
    manager
}

fn frame(manager: &ParticlesManager) -> ParticleExtract {
    let snapshot = manager.snapshot();
    let [emitter] = snapshot.emitters.as_slice() else {
        panic!("expected one emitter in the runtime snapshot");
    };
    assert_eq!(emitter.backend, ParticleSimulationBackend::Cpu);
    assert!(!emitter.fallback_to_cpu);
    assert_eq!(emitter.live_particles, snapshot.sprites.len());

    let extract = manager.build_extract(Some(Vec3::ZERO));
    assert_eq!(extract.emitters.as_slice(), &[ENTITY]);
    assert!(extract.gpu_frame.is_none());
    assert_eq!(extract.sprites.len(), snapshot.sprites.len());
    for sprite in &extract.sprites {
        assert!(snapshot.sprites.iter().any(|current| current == sprite));
        assert_eq!(sprite.entity, ENTITY);
        assert!(sprite.size > 0.0 && sprite.color.w > 0.0);
    }
    let bound_count = usize::from(!extract.sprites.is_empty());
    assert_eq!(extract.bounds.len(), bound_count);
    if let [sprite] = extract.sprites.as_slice() {
        assert_eq!(extract.bounds[0].center, sprite.position);
    }
    extract
}

// The authored size curve is 1 + lifetime * normalized_age and initial size is 1.
// Read age back from the production curve output, not from distance or the system clock.
fn x_and_age(extract: &ParticleExtract) -> Vec<(Real, Real)> {
    extract
        .sprites
        .iter()
        .map(|sprite| {
            assert_eq!(sprite.position, Vec3::new(sprite.position.x, 0.0, 0.0));
            (sprite.position.x, sprite.size - 1.0)
        })
        .collect()
}

#[test]
fn particle_birth_mid_frame_only_advances_for_its_remaining_time() {
    let manager = authored_cpu(&[0.375], 1.0);
    manager.tick(0.5).unwrap();
    // Speed 8 for 0.5 - 0.375 seconds: age 0.125, x = 1, not x = 4.
    assert_eq!(x_and_age(&frame(&manager)), [(1.0, 0.125)]);
}

#[test]
fn particle_birth_preserves_old_step_and_two_newborn_times_in_one_emitter() {
    let manager = authored_cpu(&[0.625, 0.375, 0.0], 1.0);
    manager.tick(0.25).unwrap();
    let before = frame(&manager);
    assert_eq!(x_and_age(&before), [(2.0, 0.25)]);
    let previous = RenderParticlePreviousSpriteSnapshot::from_current(&before.sprites[0]);

    manager.tick(0.5).unwrap();
    let after = frame(&manager);
    assert_eq!(after.previous_state_sprite_count_with(&[previous]), 1);
    assert_eq!(x_and_age(&after), [(6.0, 0.75), (3.0, 0.375), (1.0, 0.125)]);
    assert_eq!(after.sprites[0].identity(), previous.identity());
}

#[test]
fn particle_birth_matches_single_step_when_split_around_the_burst() {
    let whole = authored_cpu(&[0.375], 1.0);
    whole.tick(0.5).unwrap();
    let split = authored_cpu(&[0.375], 1.0);
    split.tick(0.25).unwrap();
    assert!(frame(&split).sprites.is_empty());
    split.tick(0.1875).unwrap();
    split.tick(0.0625).unwrap();

    let expected = vec![(1.0, 0.125)];
    assert_eq!(
        [x_and_age(&frame(&whole)), x_and_age(&frame(&split))],
        [expected.clone(), expected]
    );
}

#[test]
fn particle_birth_short_lifetime_reaches_render_extract_before_expiring() {
    let manager = authored_cpu(&[0.375], 0.25);
    manager.tick(0.5).unwrap();
    assert_eq!(x_and_age(&frame(&manager)), [(1.0, 0.125)]);
    manager.tick(0.125).unwrap();
    assert!(frame(&manager).sprites.is_empty());
}

#[test]
fn particle_birth_on_positive_step_end_is_visible_once_at_age_zero() {
    let manager = authored_cpu(&[0.5], 1.0);
    manager.tick(0.5).unwrap();
    let edge = frame(&manager);
    manager.tick(0.125).unwrap();
    let next = frame(&manager);
    assert_eq!(x_and_age(&edge), [(0.0, 0.0)]);
    assert_eq!(x_and_age(&next), [(1.0, 0.125)]);
    assert_eq!(edge.sprites[0].identity(), next.sprites[0].identity());
}

#[test]
fn particle_birth_at_system_start_receives_the_first_positive_step() {
    let manager = authored_cpu(&[0.0], 1.0);
    manager.tick(0.5).unwrap();
    assert_eq!(x_and_age(&frame(&manager)), [(4.0, 0.5)]);
}
