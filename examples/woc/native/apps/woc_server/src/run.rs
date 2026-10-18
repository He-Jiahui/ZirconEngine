use woc_runtime::{
    load_engine_project_vm, TickBudgets, WocHostRole, WocReloadableVm, ZrVmProjectOptions,
};

use super::options::{Options, DEFAULT_MAX_PENDING_COMMANDS, DEFAULT_MAX_PENDING_MOVEMENT};
use woc_server::{FixedServerTickDriver, SERVER_TICK_NS};

pub(super) fn run(options: Options) -> Result<String, String> {
    let (identity, mut vm) = load_engine_project_vm(
        &options.project,
        WocHostRole::Server,
        ZrVmProjectOptions::default(),
    )
    .map_err(|error| error.to_string())?;
    vm.activate()
        .map_err(|error| format!("activate: {error:?}"))?;
    let mut driver = FixedServerTickDriver::new(
        vm,
        TickBudgets::default(),
        options.max_catch_up_ticks,
        DEFAULT_MAX_PENDING_COMMANDS,
        DEFAULT_MAX_PENDING_MOVEMENT,
    )
    .map_err(|error| format!("server scheduler: {error:?}"))?;

    let result = run_ticks(&mut driver, identity, options);
    let cleanup = driver.into_vm().deactivate();
    match (result, cleanup) {
        (Ok(report), Ok(())) => Ok(report),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(error)) => Err(format!("deactivate: {error:?}")),
        (Err(error), Err(cleanup)) => Err(format!("{error}; deactivate: {cleanup:?}")),
    }
}

fn run_ticks<V: woc_runtime::WocProjectVm>(
    driver: &mut FixedServerTickDriver<V>,
    identity: woc_runtime::WocProjectIdentity,
    options: Options,
) -> Result<String, String> {
    for _ in 0..options.ticks {
        driver
            .advance(SERVER_TICK_NS)
            .map_err(|error| format!("fixed tick: {error:?}"))?;
    }

    if options.verify_replay {
        let checkpoint = driver
            .runtime_mut()
            .checkpoint()
            .map_err(|error| format!("checkpoint: {error:?}"))?;
        driver
            .advance(SERVER_TICK_NS)
            .map_err(|error| format!("replay first tick: {error:?}"))?;
        let expected = driver.runtime().committed().clone();
        driver
            .runtime_mut()
            .rollback_checkpoint(checkpoint)
            .map_err(|error| format!("rollback: {error:?}"))?;
        driver
            .advance(SERVER_TICK_NS)
            .map_err(|error| format!("replay second tick: {error:?}"))?;
        if *driver.runtime().committed() != expected {
            return Err("checkpoint replay changed the committed snapshot".into());
        }
    }

    let committed = driver.runtime().committed();
    serde_json::to_string_pretty(&serde_json::json!({
        "identity": identity,
        "role": "server",
        "committedTick": committed.tick,
        "stateBytes": committed.state.len(),
        "stateDigest": committed.state_digest,
        "eventDigest": committed.event_digest,
        "presentationBytes": committed.presentation_payload.len(),
        "presentationDigest": committed.presentation_digest,
        "presentationReady": false,
        "renderedFrames": 0,
        "replayVerified": options.verify_replay,
    }))
    .map_err(|error| error.to_string())
}
