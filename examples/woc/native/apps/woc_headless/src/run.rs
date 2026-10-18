use woc_runtime::{
    load_engine_project_vm, RuntimeRole, TickBudgets, WocHostRole, WocReloadableVm,
    WocTransactionalRuntime, ZrVmProjectOptions,
};

use super::options::Options;

pub(super) fn run(options: Options) -> Result<String, String> {
    let (identity, mut vm) = load_engine_project_vm(
        &options.project,
        WocHostRole::Headless,
        ZrVmProjectOptions::default(),
    )
    .map_err(|error| error.to_string())?;
    vm.activate()
        .map_err(|error| format!("activate: {error:?}"))?;
    let mut runtime =
        WocTransactionalRuntime::new(RuntimeRole::Offline, vm, TickBudgets::default());
    let result = run_ticks(&mut runtime, identity, options);
    let cleanup = runtime.into_vm().deactivate();
    match (result, cleanup) {
        (Ok(report), Ok(())) => Ok(report),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(error)) => Err(format!("deactivate: {error:?}")),
        (Err(error), Err(cleanup)) => Err(format!("{error}; deactivate: {cleanup:?}")),
    }
}

fn run_ticks(
    runtime: &mut WocTransactionalRuntime<woc_runtime::ZrVmProjectVm>,
    identity: woc_runtime::WocProjectIdentity,
    options: Options,
) -> Result<String, String> {
    runtime
        .install_offline_bootstrap(options.bootstrap)
        .map_err(|error| format!("bootstrap: {error:?}"))?;
    for _ in 0..options.ticks {
        runtime
            .tick(Vec::new())
            .map_err(|error| format!("fixed tick: {error:?}"))?;
    }
    if options.verify_replay {
        let checkpoint = runtime
            .checkpoint()
            .map_err(|error| format!("checkpoint: {error:?}"))?;
        let expected = runtime
            .tick(Vec::new())
            .map_err(|error| format!("replay first tick: {error:?}"))?
            .clone();
        runtime
            .rollback_checkpoint(checkpoint)
            .map_err(|error| format!("rollback: {error:?}"))?;
        let actual = runtime
            .tick(Vec::new())
            .map_err(|error| format!("replay second tick: {error:?}"))?;
        if *actual != expected {
            return Err("checkpoint replay changed the committed snapshot".into());
        }
    }
    let committed = runtime.committed();
    serde_json::to_string_pretty(&serde_json::json!({
        "identity": identity,
        "role": "headless",
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
