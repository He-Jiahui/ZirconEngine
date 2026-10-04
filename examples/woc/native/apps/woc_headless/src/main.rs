use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use woc_protocol::{
    OfflineSessionBootstrap, OfflineWeaponSkinAccount, OFFLINE_SESSION_BOOTSTRAP_VERSION,
    STANDARD_OFFLINE_WORLD_SEED,
};
use woc_runtime::{
    load_engine_project_vm, RuntimeRole, TickBudgets, WocHostRole, WocProjectIdentity,
    WocReloadableVm, WocTransactionalRuntime, ZrVmProjectOptions, ZrVmProjectVm,
};

fn main() -> ExitCode {
    match Options::parse(std::env::args_os().skip(1))
        .and_then(|options| options.map(run_headless).transpose())
    {
        Ok(Some(report)) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Ok(None) => {
            println!("woc_headless [--project PATH] [--ticks COUNT] [--class ID] [--player-name NAME] [--verify-replay]");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("woc_headless: {error}");
            ExitCode::FAILURE
        }
    }
}

struct Options {
    project: PathBuf,
    ticks: u64,
    bootstrap: OfflineSessionBootstrap,
    verify_replay: bool,
}

impl Options {
    fn parse(args: impl Iterator<Item = OsString>) -> Result<Option<Self>, String> {
        let mut args = args;
        let mut options = Self {
            project: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."),
            ticks: u64::from(woc_protocol::SIMULATION_HZ),
            bootstrap: OfflineSessionBootstrap {
                launch_version: OFFLINE_SESSION_BOOTSTRAP_VERSION,
                world_seed: STANDARD_OFFLINE_WORLD_SEED,
                player_class: 1,
                player_name: "Adventurer".into(),
                skin_variant: 0,
                weapon_skin_account: OfflineWeaponSkinAccount::default(),
            },
            verify_replay: false,
        };
        while let Some(argument) = args.next() {
            let flag = argument.to_str().ok_or("argument must be UTF-8")?;
            if matches!(flag, "--help" | "-h") {
                return Ok(None);
            }
            if flag == "--verify-replay" {
                options.verify_replay = true;
                continue;
            }
            if !matches!(flag, "--project" | "--ticks" | "--class" | "--player-name") {
                return Err(format!("unknown argument {flag}"));
            }
            let value = args
                .next()
                .ok_or_else(|| format!("{flag} requires a value"))?;
            if flag == "--project" {
                options.project = PathBuf::from(value);
                continue;
            }
            let value = value
                .to_str()
                .ok_or_else(|| format!("{flag} requires UTF-8"))?;
            match flag {
                "--ticks" => {
                    options.ticks = value
                        .parse()
                        .map_err(|_| "--ticks requires an unsigned count")?
                }
                "--class" => {
                    options.bootstrap.player_class = value
                        .parse()
                        .map_err(|_| "--class requires an unsigned ID")?
                }
                "--player-name" => options.bootstrap.player_name = value.to_owned(),
                _ => unreachable!(),
            }
        }
        if options.ticks == 0 {
            return Err("--ticks must be positive".into());
        }
        options
            .bootstrap
            .validate()
            .map_err(|error| format!("offline session: {error:?}"))?;
        Ok(Some(options))
    }
}

fn run_headless(options: Options) -> Result<String, String> {
    let (identity, mut vm) = load_engine_project_vm(
        &options.project,
        WocHostRole::Headless,
        ZrVmProjectOptions::default(),
    )
    .map_err(|error| error.to_string())?;
    if let Err(error) = vm.activate() {
        let cleanup = vm.deactivate().err();
        return Err(match cleanup {
            Some(cleanup) => format!("activate: {error:?}; cleanup: {cleanup:?}"),
            None => format!("activate: {error:?}"),
        });
    }
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
    runtime: &mut WocTransactionalRuntime<ZrVmProjectVm>,
    identity: WocProjectIdentity,
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
