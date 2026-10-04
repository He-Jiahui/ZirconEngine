use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use woc_runtime::{
    load_engine_project_vm, TickBudgets, WocHostRole, WocProjectIdentity, WocProjectVm,
    WocReloadableVm, ZrVmProjectOptions,
};
use woc_server::{FixedServerTickDriver, SERVER_TICK_NS};

const DEFAULT_MAX_CATCH_UP_TICKS: u32 = 4;
const DEFAULT_MAX_PENDING_COMMANDS: usize = 256;
const DEFAULT_MAX_PENDING_MOVEMENT: usize = 256;

fn main() -> ExitCode {
    match Options::parse(std::env::args_os().skip(1))
        .and_then(|options| options.map(run_server).transpose())
    {
        Ok(Some(report)) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Ok(None) => {
            println!(
                "woc_server [--project PATH] [--ticks COUNT] [--max-catch-up COUNT] [--verify-replay]"
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("woc_server: {error}");
            ExitCode::FAILURE
        }
    }
}

struct Options {
    project: PathBuf,
    ticks: u64,
    max_catch_up_ticks: u32,
    verify_replay: bool,
}

impl Options {
    fn parse(args: impl Iterator<Item = OsString>) -> Result<Option<Self>, String> {
        let mut args = args;
        let mut options = Self {
            project: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."),
            ticks: u64::from(woc_protocol::SIMULATION_HZ),
            max_catch_up_ticks: DEFAULT_MAX_CATCH_UP_TICKS,
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
            if !matches!(flag, "--project" | "--ticks" | "--max-catch-up") {
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
                "--max-catch-up" => {
                    options.max_catch_up_ticks = value
                        .parse()
                        .map_err(|_| "--max-catch-up requires an unsigned count")?
                }
                _ => unreachable!(),
            }
        }
        if options.ticks == 0 {
            return Err("--ticks must be positive".into());
        }
        if options.max_catch_up_ticks == 0 {
            return Err("--max-catch-up must be positive".into());
        }
        Ok(Some(options))
    }
}

fn run_server(options: Options) -> Result<String, String> {
    let (identity, mut vm) = load_engine_project_vm(
        &options.project,
        WocHostRole::Server,
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
    let mut driver = match FixedServerTickDriver::new_with_vm_recovery(
        vm,
        TickBudgets::default(),
        options.max_catch_up_ticks,
        DEFAULT_MAX_PENDING_COMMANDS,
        DEFAULT_MAX_PENDING_MOVEMENT,
    ) {
        Ok(driver) => driver,
        Err((error, mut vm)) => {
            let cleanup = vm.deactivate().err();
            return Err(match cleanup {
                Some(cleanup) => format!("server scheduler: {error:?}; cleanup: {cleanup:?}"),
                None => format!("server scheduler: {error:?}"),
            });
        }
    };

    let result = run_ticks(&mut driver, identity, &options);
    let cleanup = driver.into_vm().deactivate();
    match (result, cleanup) {
        (Ok(report), Ok(())) => Ok(report),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(error)) => Err(format!("deactivate: {error:?}")),
        (Err(error), Err(cleanup)) => Err(format!("{error}; deactivate: {cleanup:?}")),
    }
}

fn run_ticks<V: WocProjectVm>(
    driver: &mut FixedServerTickDriver<V>,
    identity: WocProjectIdentity,
    options: &Options,
) -> Result<String, String> {
    // Server sessions begin from the canonical empty world; client-only
    // offline bootstrap is deliberately rejected by RuntimeRole::Server.
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
