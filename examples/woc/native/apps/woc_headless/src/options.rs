use std::ffi::OsString;
use std::path::PathBuf;

use woc_protocol::{
    OfflineSessionBootstrap, OfflineWeaponSkinAccount, OFFLINE_SESSION_BOOTSTRAP_VERSION,
    STANDARD_OFFLINE_WORLD_SEED,
};

pub(super) struct Options {
    pub(super) project: PathBuf,
    pub(super) ticks: u64,
    pub(super) bootstrap: OfflineSessionBootstrap,
    pub(super) verify_replay: bool,
}

impl Options {
    pub(super) fn parse(args: impl Iterator<Item = OsString>) -> Result<Option<Self>, String> {
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
