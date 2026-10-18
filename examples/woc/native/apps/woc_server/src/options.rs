use std::ffi::OsString;
use std::path::PathBuf;

pub(super) const DEFAULT_MAX_CATCH_UP_TICKS: u32 = 4;
pub(super) const DEFAULT_MAX_PENDING_COMMANDS: usize = 256;
pub(super) const DEFAULT_MAX_PENDING_MOVEMENT: usize = 256;

pub(super) struct Options {
    pub(super) project: PathBuf,
    pub(super) ticks: u64,
    pub(super) max_catch_up_ticks: u32,
    pub(super) verify_replay: bool,
}

impl Options {
    pub(super) fn parse(args: impl Iterator<Item = OsString>) -> Result<Option<Self>, String> {
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
