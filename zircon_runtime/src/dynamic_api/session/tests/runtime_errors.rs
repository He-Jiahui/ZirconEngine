use crate::core::CoreError;
use std::time::Duration;

use crate::scene::{
    FixedStepFailurePhase, FixedStepFailureReceipt, LevelTickError, SimulationTickId, SystemStage,
    WorldTimeAdvanceError,
};

use super::super::error::{RuntimeDynamicSessionError, RuntimeProjectError};

#[test]
fn runtime_session_error_preserves_step_and_typed_source() {
    assert_eq!(
        RuntimeDynamicSessionError::ProjectStep {
            step: "load default level",
            source: RuntimeProjectError::EmptyProjectRoot,
        }
        .to_string(),
        "load default level: runtime project root cannot be empty"
    );
    assert_eq!(
        RuntimeDynamicSessionError::CoreStep {
            step: "activate runtime module",
            source: CoreError::MissingModule("script".to_string()),
        }
        .to_string(),
        "activate runtime module: module not found: script"
    );
    assert_eq!(
        RuntimeDynamicSessionError::LevelTick {
            source: LevelTickError::from(CoreError::MissingModule("scene".to_string())),
        }
        .to_string(),
        "tick loaded level: module not found: scene"
    );
}

#[test]
fn runtime_session_error_preserves_fixed_step_failure_receipt() {
    let expected_receipt = FixedStepFailureReceipt::new(
        FixedStepFailurePhase::Stage(SystemStage::FixedUpdate),
        SimulationTickId::new(7, 3, 11),
        Some("runtime22.fixed.failure".to_string()),
        2,
        Duration::from_millis(15),
        7,
    );
    let error = RuntimeDynamicSessionError::LevelTick {
        source: LevelTickError::fixed_step(
            expected_receipt.clone(),
            CoreError::Initialization(
                "runtime22.fixed.failure".to_string(),
                "injected".to_string(),
            ),
        ),
    };

    let RuntimeDynamicSessionError::LevelTick { source } = error else {
        unreachable!("constructed LevelTick error must retain its typed source");
    };
    assert_eq!(source.fixed_step_receipt(), Some(&expected_receipt));
}

#[test]
fn runtime_session_error_preserves_outer_frame_consumption_rejection() {
    let expected = WorldTimeAdvanceError::OutOfOrderOuterFrame {
        last_consumed: 9,
        submitted: 7,
    };
    let error = RuntimeDynamicSessionError::LevelTick {
        source: LevelTickError::from(expected),
    };

    let RuntimeDynamicSessionError::LevelTick { source } = error else {
        unreachable!("constructed LevelTick error must retain its typed source");
    };
    assert_eq!(source.world_time_advance_error(), Some(&expected));
}
