use super::*;

fn level() -> LevelSystem {
    LevelSystem::new(
        WorldHandle::new(130_030),
        Arc::new(Mutex::new(World::empty())),
        LevelMetadata::default(),
    )
}

fn fault(entity: EntityId, key: &str, message: String) -> ScriptBindingFault {
    ScriptBindingFault {
        entity,
        binding_key: Some(key.into()),
        package: "probe".into(),
        module: "main".into(),
        export: "onStart".into(),
        kind: ScriptBindingFaultKind::Resolution,
        message,
        host_fault_count: 0,
        outer_frame_index: 7,
    }
}

#[test]
fn script_binding_fault_ledger_is_bounded_and_does_not_change_world_generations() {
    let level = level();
    let before = level.with_world(|world| {
        (
            world.world_generation(),
            world.dynamic_component_generation("script.bindings"),
        )
    });
    for entry in 0..257 {
        assert!(level.record_script_binding_fault(fault(
            entry,
            "probe::main#0",
            entry.to_string()
        )));
    }
    let faults = level.script_binding_faults();
    assert_eq!(faults.len(), 256);
    assert_eq!(faults[0].entity, 1);
    assert_eq!(faults[255].entity, 256);
    assert_eq!(level.script_binding_fault_total(), 257);
    assert_eq!(
        level.with_world(|world| (
            world.world_generation(),
            world.dynamic_component_generation("script.bindings")
        )),
        before
    );
    assert!(!level.record_script_binding_fault(fault(
        0,
        "probe::main#0",
        "already faulted despite eviction".into()
    )));
    assert_eq!(level.script_binding_fault_total(), 257);
}

#[test]
fn script_binding_fault_message_is_bounded_on_a_utf8_boundary() {
    let level = level();
    let message = format!("{}界suffix", "a".repeat(1023));
    level.record_script_binding_fault(fault(1, "probe::main#0", message));
    level.record_script_binding_fault(fault(2, "probe::main#0", "é".repeat(512)));
    let faults = level.script_binding_faults();
    assert_eq!(faults[0].message, "a".repeat(1023));
    assert_eq!(faults[1].message.len(), 1024);
}

#[test]
fn script_binding_fault_identity_exit_prunes_faulted_and_started_together() {
    let level = level();
    let key = "probe::main#0";
    let other = "probe::other#1";
    for key in [key, other] {
        level.mark_script_binding_started(1, key);
        assert!(level.record_script_binding_fault(fault(1, key, "failure".into())));
    }
    let live = BTreeMap::from([(1, BTreeSet::from([Box::<str>::from(other)]))]);
    level.retain_script_binding_lifecycle(&live);
    assert!(!level.script_binding_faulted(1, key));
    assert!(!level.script_binding_started(1, key));
    assert!(level.script_binding_faulted(1, other));
    assert!(level.script_binding_started(1, other));
    assert!(level.record_script_binding_fault(fault(1, key, "new identity lifetime".into())));
    assert_eq!(level.script_binding_fault_total(), 3);
}

#[test]
fn script_binding_fault_world_replacement_clears_lifecycle_and_keeps_history() {
    let level = level();
    let key = "probe::main#0";
    level.mark_script_binding_started(1, key);
    level.record_script_binding_fault(fault(1, key, "old World".into()));
    let history = level.script_binding_faults();
    level.replace_world_and_reset_runtime_state(World::empty());
    assert!(!level.script_binding_started(1, key));
    assert!(!level.script_binding_faulted(1, key));
    assert_eq!(level.script_binding_faults(), history);
    assert_eq!(level.script_binding_fault_total(), 1);
    assert!(level.record_script_binding_fault(fault(1, key, "new World".into())));
    assert_eq!(level.script_binding_fault_total(), 2);
}
