#![cfg(feature = "backend-zr-vm")]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use woc_protocol::{fnv1a_bytes, FixedTickInput, FNV1A_OFFSET};
use woc_runtime::{
    ClientPresentationProjection, CommittedSnapshot, TickBudgets, VmExecutionTermination,
    VmTickError, WocProjectVm, WocReloadableVm, ZrVmProjectOptions, ZrVmProjectVm,
};
use zr_vm_rust_binding::CancellationToken;

struct Fixture {
    _root: tempfile::TempDir,
    project: PathBuf,
}

impl Fixture {
    fn new(schema: serde_json::Value) -> Self {
        let root = tempfile::tempdir().expect("fixture directory");
        let source = root.path().join("src");
        fs::create_dir_all(&source).unwrap();
        let main = include_str!("fixtures/zr_vm_project_vm/main.zr").replace(
            "__WOC_SCHEMA__",
            &serde_json::to_string(&schema.to_string()).unwrap(),
        );
        fs::write(source.join("main.zr"), main).unwrap();
        fs::write(
            source.join("retained.zr"),
            include_str!("fixtures/zr_vm_project_vm/retained.zr"),
        )
        .unwrap();
        let authored = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../scripts/woc_game/src");
        for module in [
            "protocol/binary.zr",
            "protocol/commands.zr",
            "generated/contracts.zr",
        ] {
            let destination = source.join(module);
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            fs::copy(authored.join(module), destination).unwrap();
        }
        let project = root.path().join("woc_game.zrp");
        fs::write(
            &project,
            serde_json::to_vec(&serde_json::json!({
                "name": "woc_game", "source": "src", "binary": "bin", "entry": "main"
            }))
            .unwrap(),
        )
        .unwrap();
        Self {
            _root: root,
            project,
        }
    }

    fn load(&self) -> Result<ZrVmProjectVm, VmTickError> {
        ZrVmProjectVm::load(
            &self.project,
            ZrVmProjectOptions {
                max_call_instructions: 1_000_000,
                lifecycle_timeout: Duration::from_secs(5),
                ..ZrVmProjectOptions::default()
            },
            [],
        )
    }
}

fn schema() -> serde_json::Value {
    serde_json::json!({
        "schema_version": 3,
        "types": [],
        "package": "woc_game",
        "contract": woc_protocol::SCHEMA_FINGERPRINT_HEX,
        "command_catalog": woc_protocol::COMMAND_CATALOG_SHA256,
        "command_payload_schema": woc_protocol::COMMAND_PAYLOAD_SCHEMA_SHA256,
        "world_state": woc_protocol::WORLD_STATE_FORMAT,
        "simulation_hz": woc_protocol::SIMULATION_HZ,
        "presentation_hz": woc_protocol::PRESENTATION_HZ,
    })
}

fn state(counter: u8) -> Vec<u8> {
    let mut state = b"WOS2".to_vec();
    state.extend_from_slice(&woc_protocol::WORLD_STATE_SCHEMA_VERSION.to_le_bytes());
    state.push(counter);
    state
}

fn input(tick: u64, state: Vec<u8>) -> Vec<u8> {
    FixedTickInput {
        tick,
        commands: vec![],
        wall_time_forbidden: true,
        committed_state_digest: fnv1a_bytes(&state),
        committed_state: state,
        generation: 0,
        movement_frames: vec![],
        offline_bootstrap: None,
    }
    .encode_payload()
    .unwrap()
}

// The SDK owns process-global native registries, so this integration binary
// deliberately runs its project scenarios serially in one test.
#[test]
fn real_retained_project_adapter_contract() {
    let startup_fixture = Fixture::new(schema());
    fs::write(
        startup_fixture
            .project
            .parent()
            .unwrap()
            .join("src/main.zr"),
        "module main;\nwhile (true) { }\nreturn 0;\n",
    )
    .unwrap();
    let startup_options = ZrVmProjectOptions {
        startup_max_instructions: 256,
        startup_timeout: Duration::from_secs(3),
        ..ZrVmProjectOptions::default()
    };
    let error = ZrVmProjectVm::load(&startup_fixture.project, startup_options.clone(), [])
        .err()
        .expect("infinite top-level entry must fail before Ready");
    assert!(matches!(
        error,
        VmTickError::Terminated {
            reason: VmExecutionTermination::InstructionLimit,
            executed_instructions: 256,
            ..
        }
    ));
    let error = ZrVmProjectVm::load(
        &startup_fixture.project,
        ZrVmProjectOptions {
            startup_timeout: Duration::ZERO,
            ..startup_options.clone()
        },
        [],
    )
    .err()
    .expect("expired startup deadline must fail before Ready");
    assert!(matches!(
        error,
        VmTickError::Terminated {
            reason: VmExecutionTermination::Deadline,
            ..
        }
    ));
    let cancellation = CancellationToken::new().unwrap();
    cancellation.cancel();
    let error = ZrVmProjectVm::load(
        &startup_fixture.project,
        ZrVmProjectOptions {
            cancellation: Some(cancellation),
            ..startup_options
        },
        [],
    )
    .err()
    .expect("pre-cancelled startup must fail before Ready");
    assert!(matches!(
        error,
        VmTickError::Terminated {
            reason: VmExecutionTermination::Cancelled,
            ..
        }
    ));

    let fixture = Fixture::new(schema());
    let mut vm = fixture
        .load()
        .expect("compile and load a retained .zr project");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&vm.state_schema().unwrap()).unwrap(),
        schema()
    );
    vm.activate().unwrap();
    assert_eq!(vm.save_state().unwrap(), state(0));
    assert_eq!(
        vm.save_state().unwrap(),
        state(0),
        "save's hidden heap mutation must roll back"
    );
    let outer = vm.checkpoint().unwrap();
    for byte in [0, 1, 127, 128, 255] {
        vm.restore_state(&state(byte)).unwrap();
        assert_eq!(vm.save_state().unwrap(), state(byte));
    }
    vm.rollback(&outer).unwrap();
    assert_eq!(vm.save_state().unwrap(), state(0));
    vm.restore_state(&state(7)).unwrap();
    assert!(matches!(
        vm.restore_state(&state(254)),
        Err(VmTickError::Trap(_))
    ));
    assert_eq!(vm.save_state().unwrap(), state(7));
    assert!(matches!(
        vm.restore_state(b"{\"schema_version\":3}"),
        Err(VmTickError::Transport(_))
    ));
    assert_eq!(vm.save_state().unwrap(), state(7));
    let snapshot = CommittedSnapshot {
        generation: 2,
        tick: 9,
        state: state(9),
        state_digest: fnv1a_bytes(&state(9)),
        event_digest: FNV1A_OFFSET,
        presentation_digest: FNV1A_OFFSET,
        presentation_payload: vec![],
    };
    vm.install_full_snapshot(&snapshot).unwrap();
    assert_eq!(vm.save_state().unwrap(), snapshot.state);
    let budgets = TickBudgets {
        max_execution_micros: 5_000_000,
        ..TickBudgets::default()
    };
    assert!(matches!(
        vm.fixed_tick(&input(1, state(8)), budgets),
        Err(VmTickError::Trap(_))
    ));
    assert_eq!(vm.save_state().unwrap(), state(9));
    let output = vm.fixed_tick(&input(1, state(9)), budgets).unwrap();
    let snapshot = woc_protocol::WorldSnapshot::decode_payload(&output.output_payload).unwrap();
    assert_eq!(snapshot.state, state(10));
    let presentation = ClientPresentationProjection::decode_json(&output.presentation_payload)
        .expect("adapter must return a validated client presentation payload");
    assert_eq!(presentation.world.viewer.id, 1);
    assert_eq!(presentation.hud.player.display_name, "fixture");
    assert!(output.usage.memory_bytes > 0);
    assert!(output.usage.host_calls > 0);
    assert!(vm.last_call_usage().unwrap().executed_instructions > 0);
    assert_eq!(vm.save_state().unwrap(), state(10));
    vm.restore_state(&state(9)).unwrap();
    assert!(matches!(
        vm.fixed_tick(&input(103, state(9)), budgets),
        Err(VmTickError::Transport(_))
    ));
    assert_eq!(
        vm.save_state().unwrap(),
        state(9),
        "an invalid presentation payload must roll back the fixed tick"
    );
    for (budgets, expected) in [
        (
            TickBudgets {
                max_memory_bytes: 0,
                ..budgets
            },
            VmExecutionTermination::HeapLimit,
        ),
        (
            TickBudgets {
                max_host_calls: 0,
                ..budgets
            },
            VmExecutionTermination::NativeCallLimit,
        ),
    ] {
        assert!(
            matches!(vm.fixed_tick(&input(1, state(9)), budgets), Err(VmTickError::Terminated { reason, .. }) if reason == expected)
        );
        assert_eq!(vm.save_state().unwrap(), state(9));
    }
    assert!(matches!(
        vm.fixed_tick(
            &input(100, state(9)),
            TickBudgets {
                max_host_calls: u64::MAX,
                ..budgets
            }
        ),
        Err(VmTickError::Terminated {
            reason: VmExecutionTermination::InstructionLimit,
            executed_instructions: 1_000_000,
            ..
        })
    ));
    assert_eq!(vm.save_state().unwrap(), state(9));
    assert!(matches!(
        vm.fixed_tick(&input(101, state(9)), budgets),
        Err(VmTickError::Transport(_))
    ));
    assert!(matches!(
        vm.fixed_tick(&input(102, state(9)), budgets),
        Err(VmTickError::Trap(_))
    ));
    assert_eq!(vm.save_state().unwrap(), state(9));
    let cancellation = CancellationToken::new().unwrap();
    cancellation.cancel();
    vm.set_cancellation(Some(cancellation));
    assert!(matches!(
        vm.activate(),
        Err(VmTickError::Terminated {
            reason: VmExecutionTermination::Cancelled,
            executed_instructions: 0,
            ..
        })
    ));
    vm.set_cancellation(None);
    vm.activate().unwrap();
    assert_eq!(vm.save_state().unwrap(), state(9));
    vm.deactivate().unwrap();
    assert!(matches!(
        vm.fixed_tick(&input(1, state(9)), budgets),
        Err(VmTickError::Trap(_))
    ));
    drop(vm);
    drop(outer);

    let mut wrong_schema = schema();
    wrong_schema["simulation_hz"] = serde_json::json!(21);
    let incompatible = Fixture::new(wrong_schema);
    assert!(matches!(incompatible.load(), Err(VmTickError::Limited(_))));
}
