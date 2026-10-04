use zircon_runtime::core::framework::script::ScriptHostValue;
use zircon_runtime::script::{VmBackend, VmGcBudget, VmPluginInstance, VmStateBlob};

use crate::{register_zr_vm_backend, ZrVmBackend};

use super::support::build_real_backend_host;
use super::woc_codec::woc_project_fixture;

#[test]
fn real_backend_woc_lifecycle_retains_actual_world_and_rejects_stale_tick_state() {
    let fixture = woc_project_fixture("main");
    let manager = zircon_runtime::script::VmPluginManager::mock();
    let packages = manager.discover_packages(&fixture.root).unwrap();
    let host = build_real_backend_host(&manager, &packages[0]);
    let mut instance = ZrVmBackend::default()
        .load_package(&packages[0].package, &host)
        .unwrap();
    instance.activate(&host).unwrap();
    assert!(instance.save_state().unwrap().payload.is_empty());

    let first_input = tick_input(1, &[]);
    let first_output = call_bytes(instance.as_mut(), "fixedTick", first_input.clone());
    let first = VmStateBlob::from_payload(snapshot_state(&first_output, 1));
    assert_eq!(instance.save_state().unwrap(), first);
    assert_eq!(&first.payload[..6], b"WOS2\x76\x00");
    let second_input = tick_input(2, &first.payload);
    let second_output = call_bytes(instance.as_mut(), "fixedTick", second_input.clone());
    let second = VmStateBlob::from_payload(snapshot_state(&second_output, 2));
    assert_ne!(second.payload, first.payload);
    assert_eq!(instance.save_state().unwrap(), second);

    for invalid in [vec![], first_input, {
        let mut input = tick_input(3, &second.payload);
        // The state digest follows the u32 length and committed state bytes.
        input[17 + second.payload.len()] ^= 1;
        input
    }] {
        assert!(instance
            .call_export("main", "fixedTick", &[ScriptHostValue::Bytes(invalid)])
            .is_err());
        assert_eq!(instance.save_state().unwrap(), second);
    }
    let mut truncated = first.payload.clone();
    truncated.pop();
    let mut trailing = first.payload.clone();
    trailing.push(0);
    for invalid in [truncated, trailing] {
        assert!(instance
            .restore_state(&VmStateBlob::from_payload(invalid))
            .is_err());
        assert_eq!(instance.save_state().unwrap(), second);
    }
    let mut unsupported = first.payload.clone();
    unsupported[4] = 119;
    assert!(instance
        .restore_state(&VmStateBlob::from_payload(unsupported))
        .is_err());
    assert_eq!(instance.save_state().unwrap(), second);

    instance.restore_state(&first).unwrap();
    instance
        .gc_step(VmGcBudget {
            max_micros_per_frame: 1_000,
        })
        .unwrap();
    assert_eq!(instance.save_state().unwrap(), first);
    assert_eq!(
        call_bytes(instance.as_mut(), "fixedTick", second_input),
        second_output
    );
    instance.deactivate().unwrap();
    assert_eq!(instance.save_state().unwrap(), second);
    assert!(instance
        .call_export(
            "main",
            "fixedTick",
            &[ScriptHostValue::Bytes(tick_input(3, &second.payload))]
        )
        .is_err());
    instance.activate(&host).unwrap();
    assert_eq!(instance.save_state().unwrap(), second);
}

#[test]
fn real_backend_woc_canonical_state_migrates_and_survives_actual_hot_reload() {
    let fixture = woc_project_fixture("main");
    let manager = zircon_runtime::script::VmPluginManager::mock();
    register_zr_vm_backend(&manager);
    let packages = manager.discover_packages(&fixture.root).unwrap();
    let slot = manager.load_discovered_package(&packages[0]).unwrap();
    assert_eq!(
        manager
            .call_slot_export(slot, "kernel/lifecycle_state_tests", "selfTest", &[])
            .unwrap(),
        Some(ScriptHostValue::Int(1))
    );
    let output = manager
        .call_slot_export(
            slot,
            "main",
            "fixedTick",
            &[ScriptHostValue::Bytes(tick_input(1, &[]))],
        )
        .unwrap()
        .unwrap();
    let ScriptHostValue::Bytes(output) = output else {
        panic!("tick output must be binary")
    };
    let state = snapshot_state(&output, 1);
    manager
        .hot_reload_discovered_slot(slot, &packages[0])
        .unwrap();
    assert_eq!(manager.slot(slot).unwrap().generation, 2);
    assert_eq!(
        manager
            .call_slot_export(slot, "main", "saveState", &[])
            .unwrap(),
        Some(ScriptHostValue::Bytes(state.clone()))
    );
    let output = manager
        .call_slot_export(
            slot,
            "main",
            "fixedTick",
            &[ScriptHostValue::Bytes(tick_input(2, &state))],
        )
        .unwrap()
        .unwrap();
    let ScriptHostValue::Bytes(output) = output else {
        panic!("tick output must be binary")
    };
    assert_ne!(snapshot_state(&output, 2), state);
    manager.unload_slot(slot).unwrap();
}

fn call_bytes(instance: &mut dyn VmPluginInstance, export: &str, input: Vec<u8>) -> Vec<u8> {
    match instance
        .call_export("main", export, &[ScriptHostValue::Bytes(input)])
        .unwrap()
        .unwrap()
    {
        ScriptHostValue::Bytes(bytes) => bytes,
        value => panic!("{export} must return bytes, got {value:?}"),
    }
}

// Independent protocol fixture; no gameplay state is reconstructed in Rust.
fn tick_input(tick: u64, committed: &[u8]) -> Vec<u8> {
    let mut bytes = tick.to_le_bytes().to_vec();
    bytes.extend_from_slice(&0_u32.to_le_bytes()); // Commands.
    bytes.push(1); // Wall time forbidden.
    bytes.extend_from_slice(&(committed.len() as u32).to_le_bytes());
    bytes.extend_from_slice(committed);
    bytes.extend_from_slice(&digest(committed).to_le_bytes());
    bytes.extend_from_slice(&1_u64.to_le_bytes()); // Generation.
    bytes.extend_from_slice(&0_u32.to_le_bytes()); // Movement frames.
    bytes.extend_from_slice(&0_u32.to_le_bytes()); // Offline bootstrap.
    bytes
}

fn snapshot_state(payload: &[u8], tick: u64) -> Vec<u8> {
    assert_eq!(u64::from_le_bytes(payload[..8].try_into().unwrap()), tick);
    assert_eq!(
        u32::from_le_bytes(payload[12..16].try_into().unwrap()),
        digest(&[])
    );
    let length = u32::from_le_bytes(payload[16..20].try_into().unwrap()) as usize;
    assert_eq!(payload.len(), 24 + length);
    assert_eq!(&payload[20 + length..], &[0; 4]);
    let state = payload[20..20 + length].to_vec();
    assert_eq!(
        u32::from_le_bytes(payload[8..12].try_into().unwrap()),
        digest(&state)
    );
    state
}

fn digest(bytes: &[u8]) -> u32 {
    bytes.iter().fold(2_166_136_261_u32, |hash, byte| {
        (hash ^ u32::from(*byte)).wrapping_mul(16_777_619)
    })
}
