use crate::core::framework::random::RandomServiceCheckpoint;
use crate::script::VmError;

use super::*;

struct RandomHostFixture {
    core: CoreRuntime,
    level: LevelSystem,
    exports: HostExportRegistry,
    entity: u64,
    other_entity: u64,
}

impl RandomHostFixture {
    fn new(seed: u64) -> Self {
        let core = CoreRuntime::with_random_seed(seed);
        let mut world = World::empty();
        let entity = world.spawn_node(NodeKind::Empty).unwrap();
        let other_entity = world.spawn_node(NodeKind::Empty).unwrap();
        let level = LevelSystem::new(
            WorldHandle::new(91),
            Arc::new(Mutex::new(world)),
            LevelMetadata::default(),
        );
        let exports = HostExportRegistry::new(HostRegistry::default());
        register_gameplay_host_module(&exports).unwrap();
        Self {
            core,
            level,
            exports,
            entity,
            other_entity,
        }
    }

    fn call_at(
        &self,
        level: &LevelSystem,
        entity: u64,
        arguments: Vec<ScriptHostValue>,
        capabilities: &CapabilitySet,
    ) -> Result<ScriptHostValue, VmError> {
        with_script_runtime_call_context(
            ScriptRuntimeCallContext {
                core: self.core.weak(),
                level: level.clone(),
                entity,
                delta_seconds: 0.016,
            },
            || {
                self.exports.call_with_capabilities(
                    GAMEPLAY_MODULE,
                    "random_range",
                    arguments,
                    capabilities,
                )
            },
        )
    }

    fn range_at(&self, level: &LevelSystem, entity: u64, min: f64, max: f64) -> f64 {
        match self
            .call_at(
                level,
                entity,
                vec![ScriptHostValue::Float(min), ScriptHostValue::Float(max)],
                &CapabilitySet::default().with("gameplay.random"),
            )
            .unwrap()
        {
            ScriptHostValue::Float(value) => value,
            value => panic!("random_range returned {value:?}"),
        }
    }

    fn range(&self, min: f64, max: f64) -> f64 {
        self.range_at(&self.level, self.entity, min, max)
    }

    fn checkpoint(&self) -> RandomServiceCheckpoint {
        self.core.random_service().checkpoint().unwrap()
    }
}

#[test]
fn gameplay_host_random_range_replays_seeded_progress_across_host_calls() {
    let first = RandomHostFixture::new(782);
    let replay = RandomHostFixture::new(782);
    let different_seed = RandomHostFixture::new(783);
    let values = (0..8).map(|_| first.range(-4.0, 9.0)).collect::<Vec<_>>();
    assert_eq!(
        values,
        (0..8).map(|_| replay.range(-4.0, 9.0)).collect::<Vec<_>>()
    );
    assert_ne!(
        values,
        (0..8)
            .map(|_| different_seed.range(-4.0, 9.0))
            .collect::<Vec<_>>()
    );
    assert!(values.windows(2).any(|pair| pair[0] != pair[1]));
    let checkpoint = first.checkpoint();
    assert_eq!(checkpoint.streams().len(), 1);
    assert_eq!(checkpoint.streams()[0].state().draw_index(), 16);
    assert_eq!(first.core.random_service().active_lease_count(), 0);
}

#[test]
fn gameplay_host_random_range_keeps_finite_f64_bounds_and_equal_ranges() {
    let fixture = RandomHostFixture::new(37);
    for value in [0.0, -4.0, f64::MAX, -f64::MAX, f64::MIN_POSITIVE] {
        assert_eq!(fixture.range(value, value), value);
    }
    assert!(fixture.checkpoint().streams().is_empty());

    for (min, max) in [
        (1.0, 10.0),
        (-10.0, -1.0),
        (-10.0, 10.0),
        (-f64::MAX, f64::MAX),
        (f64::MAX / 2.0, f64::MAX),
        (-f64::MAX, -f64::MAX / 2.0),
        (1.0, 1.0 + f64::EPSILON),
        (-f64::from_bits(1), f64::from_bits(1)),
        (0.0, f64::from_bits(1)),
    ] {
        for _ in 0..32 {
            let value = fixture.range(min, max);
            assert!(
                value.is_finite() && min <= value && value < max,
                "{value} outside [{min}, {max})"
            );
        }
    }
}

#[test]
fn gameplay_host_random_range_rejects_invalid_arguments_without_consuming_progress() {
    let fixture = RandomHostFixture::new(44);
    let replay = RandomHostFixture::new(44);
    let capabilities = CapabilitySet::default().with("gameplay.random");
    let first = fixture.range(-1.0, 1.0);
    assert_eq!(first, replay.range(-1.0, 1.0));
    let before = fixture.checkpoint();
    for (min, max) in [
        (4.0, -2.0),
        (f64::NAN, 2.0),
        (1.0, f64::NAN),
        (f64::NEG_INFINITY, 2.0),
        (1.0, f64::INFINITY),
        (f64::INFINITY, f64::INFINITY),
    ] {
        let error = fixture
            .call_at(
                &fixture.level,
                fixture.entity,
                vec![ScriptHostValue::Float(min), ScriptHostValue::Float(max)],
                &capabilities,
            )
            .unwrap_err();
        assert!(error.to_string().contains("finite bounds"));
        assert_eq!(fixture.checkpoint(), before);
    }
    for arguments in [
        vec![
            ScriptHostValue::String("0".into()),
            ScriptHostValue::Float(1.0),
        ],
        vec![ScriptHostValue::Float(0.0)],
        vec![
            ScriptHostValue::Float(0.0),
            ScriptHostValue::Float(1.0),
            ScriptHostValue::Float(2.0),
        ],
    ] {
        assert!(fixture
            .call_at(&fixture.level, fixture.entity, arguments, &capabilities)
            .is_err());
        assert_eq!(fixture.checkpoint(), before);
    }
    assert_eq!(fixture.range(2.0, 2.0), 2.0);
    assert_eq!(fixture.checkpoint(), before);
    assert_eq!(fixture.range(-1.0, 1.0), replay.range(-1.0, 1.0));
}

#[test]
fn gameplay_host_random_range_requires_random_capability_and_active_context_entity() {
    let fixture = RandomHostFixture::new(48);
    let arguments = vec![ScriptHostValue::Float(0.0), ScriptHostValue::Float(1.0)];
    for capabilities in [
        CapabilitySet::default(),
        CapabilitySet::default().with("gameplay.entity"),
    ] {
        let error = fixture
            .call_at(
                &fixture.level,
                fixture.entity,
                arguments.clone(),
                &capabilities,
            )
            .unwrap_err();
        assert!(error.to_string().contains("missing capability"));
        assert!(error.to_string().contains("gameplay.random"));
    }
    let capabilities = CapabilitySet::default().with("gameplay.random");
    let missing_context = fixture
        .exports
        .call_with_capabilities(
            GAMEPLAY_MODULE,
            "random_range",
            arguments.clone(),
            &capabilities,
        )
        .unwrap_err();
    assert!(missing_context
        .to_string()
        .contains("context is not active"));
    let missing_entity = fixture
        .call_at(&fixture.level, 999, arguments, &capabilities)
        .unwrap_err();
    assert!(missing_entity.to_string().contains("entity 999 is missing"));
    assert!(fixture.checkpoint().streams().is_empty());
}

#[test]
fn gameplay_host_random_range_isolates_entities_and_worlds_without_resetting_on_mutations() {
    let fixture = RandomHostFixture::new(64);
    let replay = RandomHostFixture::new(64);
    let other_level = LevelSystem::new(
        WorldHandle::new(92),
        Arc::new(Mutex::new(fixture.level.snapshot())),
        LevelMetadata::default(),
    );
    assert_eq!(fixture.range(0.0, 1.0), replay.range(0.0, 1.0));
    let second_entity = fixture.range_at(&fixture.level, fixture.other_entity, 0.0, 1.0);
    assert_eq!(
        second_entity,
        replay.range_at(&replay.level, replay.other_entity, 0.0, 1.0)
    );
    fixture.range_at(&fixture.level, fixture.other_entity, 0.0, 1.0);
    fixture.range_at(&other_level, fixture.entity, 0.0, 1.0);
    fixture.level.with_world_mut(|world| {
        world
            .set_dynamic_component(fixture.entity, "gameplay.score", serde_json::json!(1))
            .unwrap();
    });
    assert_eq!(fixture.range(0.0, 1.0), replay.range(0.0, 1.0));
    let checkpoint = fixture.checkpoint();
    assert_eq!(checkpoint.streams().len(), 3);
    assert!(checkpoint.streams().iter().any(|stream| {
        stream.key().world().id() == 92 && stream.key().entity().unwrap().id() == fixture.entity
    }));
}

#[test]
fn gameplay_host_random_range_separates_recreated_entities_and_replaced_worlds() {
    let fixture = RandomHostFixture::new(68);
    fixture.range(0.0, 1.0);
    let original_key = fixture.checkpoint().streams()[0].key();
    fixture.level.with_world_mut(|world| {
        let record = world.node_record(fixture.entity).unwrap();
        world.remove_entity(fixture.entity).unwrap();
        world.insert_node_record(record).unwrap();
    });
    fixture.range(0.0, 1.0);
    let after_recreation = fixture.checkpoint();
    assert_eq!(after_recreation.streams().len(), 2);
    let recreated_key = after_recreation
        .streams()
        .iter()
        .map(|stream| stream.key())
        .find(|key| *key != original_key)
        .unwrap();
    assert_eq!(recreated_key.world(), original_key.world());
    assert_eq!(
        recreated_key.entity().unwrap().id(),
        original_key.entity().unwrap().id()
    );
    assert_ne!(
        recreated_key.entity().unwrap().generation(),
        original_key.entity().unwrap().generation()
    );

    // A different entity can take the retired slot before the stable ID is reinserted.
    fixture.level.with_world_mut(|world| {
        let record = world.node_record(fixture.entity).unwrap();
        world.remove_entity(fixture.entity).unwrap();
        world.spawn_node(NodeKind::Empty).unwrap();
        world.insert_node_record(record).unwrap();
    });
    fixture.range(0.0, 1.0);
    assert_eq!(fixture.checkpoint().streams().len(), 3);

    let replacement = fixture.level.snapshot();
    fixture.level.replace(replacement);
    fixture.range(0.0, 1.0);
    let after_replacement = fixture.checkpoint();
    assert_eq!(after_replacement.streams().len(), 4);
    assert!(after_replacement.streams().iter().any(|stream| {
        stream.key().world().id() == original_key.world().id()
            && stream.key().world().generation() != original_key.world().generation()
    }));
}
