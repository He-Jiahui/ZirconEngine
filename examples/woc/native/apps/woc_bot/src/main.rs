use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::json;
use woc_protocol::{
    OfflineSessionBootstrap, OfflineWeaponSkinAccount, OFFLINE_SESSION_BOOTSTRAP_VERSION,
    STANDARD_OFFLINE_WORLD_SEED,
};
use woc_runtime::{
    load_engine_project_vm, RuntimeRole, TickBudgets, WocHostRole, WocProjectIdentity,
    WocReloadableVm, WocTransactionalRuntime, ZrVmProjectOptions, ZrVmProjectVm,
};

fn main() -> ExitCode {
    match Options::parse(std::env::args_os().skip(1)).and_then(|options| {
        options
            .map(|options| BotProductHost::start(&options.project, &options)?.run(&options))
            .transpose()
    }) {
        Ok(Some(report)) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Ok(None) => {
            println!("woc_bot [--project PATH] [--ticks COUNT] [--class ID] [--player-name NAME] [--skin INDEX] [--verify-replay]");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("woc_bot: {error}");
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
            if !matches!(
                flag,
                "--project" | "--ticks" | "--class" | "--player-name" | "--skin"
            ) {
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
                "--skin" => {
                    options.bootstrap.skin_variant = value
                        .parse()
                        .map_err(|_| "--skin requires an unsigned index")?
                }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BotProductPhase {
    Constructing,
    Ready,
    Running,
    Stopping,
    Stopped,
}

impl BotProductPhase {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Constructing => "constructing",
            Self::Ready => "ready",
            Self::Running => "running",
            Self::Stopping => "stopping",
            Self::Stopped => "stopped",
        }
    }
}

type BotRuntime = WocTransactionalRuntime<ZrVmProjectVm>;

/// Engine-owned lifecycle for a local deterministic bot episode. Network,
/// persistence, and render providers are intentionally not fabricated here.
struct BotProductHost {
    identity: WocProjectIdentity,
    phase: BotProductPhase,
    transitions: Vec<BotProductPhase>,
    runtime: Option<BotRuntime>,
}

#[derive(Clone, Copy)]
struct RunMetrics {
    committed_tick: u64,
    state_bytes: usize,
    state_digest: u32,
    event_digest: u32,
    observations: u64,
    actions: u64,
    replay_verified: bool,
}

impl BotProductHost {
    fn start(project: impl AsRef<Path>, options: &Options) -> Result<Self, String> {
        let mut transitions = vec![BotProductPhase::Constructing];
        let (identity, mut vm) =
            load_engine_project_vm(project, WocHostRole::Bot, ZrVmProjectOptions::default())
                .map_err(|error| format!("load bot project VM: {error}"))?;
        if let Err(error) = vm.activate() {
            let cleanup = vm.deactivate().err();
            return Err(match cleanup {
                Some(cleanup) => {
                    format!("activate bot project VM: {error:?}; cleanup: {cleanup:?}")
                }
                None => format!("activate bot project VM: {error:?}"),
            });
        }

        let mut runtime =
            WocTransactionalRuntime::new(RuntimeRole::Offline, vm, TickBudgets::default());
        if let Err(error) = runtime.install_offline_bootstrap(options.bootstrap.clone()) {
            let cleanup = deactivate_runtime(runtime);
            return Err(match cleanup {
                Some(cleanup) => format!("prepare bot session: {error:?}; cleanup: {cleanup}"),
                None => format!("prepare bot session: {error:?}"),
            });
        }
        transitions.push(BotProductPhase::Ready);
        Ok(Self {
            identity,
            phase: BotProductPhase::Ready,
            transitions,
            runtime: Some(runtime),
        })
    }

    fn run(mut self, options: &Options) -> Result<String, String> {
        self.transition(BotProductPhase::Running);
        let run_result = self.run_episode(options.ticks, options.verify_replay);
        self.transition(BotProductPhase::Stopping);
        let cleanup_result = self.shutdown();
        let metrics = match (run_result, cleanup_result) {
            (Ok(metrics), Ok(())) => metrics,
            (Err(error), Ok(())) => return Err(error),
            (Ok(_), Err(error)) => return Err(error),
            (Err(error), Err(cleanup)) => return Err(format!("{error}; shutdown: {cleanup}")),
        };

        serde_json::to_string_pretty(&json!({
            "identity": self.identity,
            "role": "bot",
            "mode": "local_deterministic",
            "lifecycle": self.transitions.iter().map(|phase| phase.as_str()).collect::<Vec<_>>(),
            "phase": self.phase.as_str(),
            "realVm": true,
            "observations": metrics.observations,
            "actions": metrics.actions,
            "committedTick": metrics.committed_tick,
            "stateBytes": metrics.state_bytes,
            "stateDigest": metrics.state_digest,
            "eventDigest": metrics.event_digest,
            "replayVerified": metrics.replay_verified,
            "networkReady": false,
            "persistenceReady": false,
            "renderReady": false,
        }))
        .map_err(|error| format!("serialize bot report: {error}"))
    }

    fn run_episode(&mut self, ticks: u64, verify_replay: bool) -> Result<RunMetrics, String> {
        let runtime = self
            .runtime
            .as_mut()
            .ok_or_else(|| "bot runtime is not available".to_owned())?;
        let mut observations = 0;
        let mut actions = 0;
        for _ in 0..ticks {
            // The local policy is deliberately deterministic: observe the
            // committed state, then emit one empty action batch.
            let _observation = runtime.committed().state_digest;
            observations += 1;
            runtime
                .tick(Vec::new())
                .map_err(|error| format!("bot fixed tick: {error:?}"))?;
            actions += 1;
        }
        if verify_replay {
            verify_replay_snapshot(runtime)?;
        }
        let committed = runtime.committed();
        Ok(RunMetrics {
            committed_tick: committed.tick,
            state_bytes: committed.state.len(),
            state_digest: committed.state_digest,
            event_digest: committed.event_digest,
            observations,
            actions,
            replay_verified: verify_replay,
        })
    }

    fn shutdown(&mut self) -> Result<(), String> {
        let runtime = self
            .runtime
            .take()
            .ok_or_else(|| "bot runtime was already stopped".to_owned())?;
        let mut vm = runtime.into_vm();
        vm.deactivate()
            .map_err(|error| format!("deactivate bot project VM: {error:?}"))?;
        self.transition(BotProductPhase::Stopped);
        Ok(())
    }

    fn transition(&mut self, phase: BotProductPhase) {
        self.phase = phase;
        self.transitions.push(phase);
    }
}

fn verify_replay_snapshot(runtime: &mut BotRuntime) -> Result<(), String> {
    let checkpoint = runtime
        .checkpoint()
        .map_err(|error| format!("bot replay checkpoint: {error:?}"))?;
    let expected = runtime
        .tick(Vec::new())
        .map_err(|error| format!("bot replay first tick: {error:?}"))?
        .clone();
    runtime
        .rollback_checkpoint(checkpoint)
        .map_err(|error| format!("bot replay rollback: {error:?}"))?;
    let actual = runtime
        .tick(Vec::new())
        .map_err(|error| format!("bot replay second tick: {error:?}"))?
        .clone();
    if actual != expected {
        return Err("bot checkpoint replay changed the committed snapshot".into());
    }
    Ok(())
}

fn deactivate_runtime(runtime: BotRuntime) -> Option<String> {
    let mut vm = runtime.into_vm();
    vm.deactivate()
        .err()
        .map(|error| format!("deactivate bot project VM: {error:?}"))
}
