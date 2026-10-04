use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use woc_client::{
    ClientProductHost, OfflinePlayerClass, OfflineSessionDraft, OfflineSessionLaunch,
};

struct Options {
    project: PathBuf,
    ticks: u64,
    launch: OfflineSessionLaunch,
    verify_replay: bool,
}

impl Options {
    fn parse(args: impl Iterator<Item = OsString>) -> Result<Option<Self>, String> {
        let mut args = args;
        // `load_engine_project_vm` accepts the WOC project root and resolves
        // the authored scripts/woc_game package from its manifest.
        let mut project = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let mut ticks = u64::from(woc_protocol::SIMULATION_HZ);
        let mut player_class = OfflinePlayerClass::Mage;
        let mut player_name = String::from("Adventurer");
        let mut skin_variant: u16 = 0;
        let mut verify_replay = false;

        while let Some(argument) = args.next() {
            let flag = argument.to_str().ok_or("argument must be UTF-8")?;
            if matches!(flag, "--help" | "-h") {
                return Ok(None);
            }
            if flag == "--verify-replay" {
                verify_replay = true;
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
                project = PathBuf::from(value);
                continue;
            }
            let value = value
                .to_str()
                .ok_or_else(|| format!("{flag} requires UTF-8"))?;
            match flag {
                "--ticks" => {
                    ticks = value
                        .parse()
                        .map_err(|_| "--ticks requires an unsigned count")?
                }
                "--class" => {
                    player_class = parse_class(value).ok_or_else(|| {
                        format!("--class must be a class id or bootstrap index: {value}")
                    })?
                }
                "--player-name" => player_name = value.to_owned(),
                "--skin" => {
                    skin_variant = value
                        .parse()
                        .map_err(|_| "--skin requires an unsigned index")?
                }
                _ => unreachable!(),
            }
        }
        if ticks == 0 {
            return Err("--ticks must be positive".into());
        }
        let mut draft = OfflineSessionDraft::default();
        draft.set_player_class(player_class);
        draft.set_skin_variant(skin_variant);
        draft.set_raw_name(player_name);
        let launch = draft
            .launch()
            .map_err(|error| format!("offline session: {error:?}"))?;
        Ok(Some(Self {
            project,
            ticks,
            launch,
            verify_replay,
        }))
    }
}

fn main() -> ExitCode {
    match Options::parse(std::env::args_os().skip(1)).and_then(|options| {
        options
            .map(|options| {
                ClientProductHost::start(&options.project, &options.launch)?
                    .run(options.ticks, options.verify_replay)
            })
            .transpose()
    }) {
        Ok(Some(report)) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Ok(None) => {
            println!(
                "woc_client [--project PATH] [--ticks COUNT] [--class ID|NAME] [--skin INDEX] [--player-name NAME] [--verify-replay]"
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("woc_client: {error}");
            ExitCode::FAILURE
        }
    }
}

fn parse_class(value: &str) -> Option<OfflinePlayerClass> {
    OfflinePlayerClass::parse(value).or_else(|| {
        let bootstrap_index = value.parse::<u8>().ok()?;
        OfflinePlayerClass::ALL
            .into_iter()
            .find(|player_class| player_class.bootstrap_index() == bootstrap_index)
    })
}
