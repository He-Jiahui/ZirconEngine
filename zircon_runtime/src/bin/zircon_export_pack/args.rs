use std::ffi::OsString;
use std::path::PathBuf;

use super::error::{ExportPackError, ExportPackResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackArgs {
    pub profile: String,
    pub manifest: PathBuf,
    pub pack: PathBuf,
    pub previous_pack: Option<PathBuf>,
    pub delta_pack: Option<PathBuf>,
    pub report: Option<PathBuf>,
    pub stage_output: Option<PathBuf>,
    pub pretty: bool,
    pub determinism_check: bool,
}

pub fn parse(args: impl IntoIterator<Item = OsString>) -> ExportPackResult<Option<PackArgs>> {
    let mut profile = None;
    let mut manifest = None;
    let mut pack = None;
    let mut previous_pack = None;
    let mut delta_pack = None;
    let mut report = None;
    let mut stage_output = None;
    let mut pretty = false;
    let mut determinism_check = false;

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let arg_text = arg.to_str().ok_or_else(|| {
            ExportPackError::Usage(usage("zircon_export_pack expects UTF-8 command arguments"))
        })?;
        match arg_text {
            "-h" | "--help" => return Ok(None),
            "--profile" => profile = Some(next_string(&mut args, "--profile")?),
            "--manifest" => manifest = Some(next_path(&mut args, "--manifest")?),
            "--pack" => pack = Some(next_path(&mut args, "--pack")?),
            "--previous-pack" => previous_pack = Some(next_path(&mut args, "--previous-pack")?),
            "--delta-pack" => delta_pack = Some(next_path(&mut args, "--delta-pack")?),
            "--report" => report = Some(next_path(&mut args, "--report")?),
            "--stage-output" => stage_output = Some(next_path(&mut args, "--stage-output")?),
            "--pretty" => pretty = true,
            "--determinism-check" => determinism_check = true,
            unknown => {
                return Err(ExportPackError::Usage(usage(&format!(
                    "unknown argument {unknown}"
                ))))
            }
        }
    }

    let profile = profile.ok_or_else(|| ExportPackError::Usage(usage("missing --profile")))?;
    let manifest = manifest.ok_or_else(|| ExportPackError::Usage(usage("missing --manifest")))?;
    let pack = pack.ok_or_else(|| ExportPackError::Usage(usage("missing --pack")))?;
    if previous_pack.is_some() != delta_pack.is_some() {
        return Err(ExportPackError::Usage(usage(
            "--previous-pack and --delta-pack must be supplied together",
        )));
    }
    Ok(Some(PackArgs {
        profile,
        manifest,
        pack,
        previous_pack,
        delta_pack,
        report,
        stage_output,
        pretty,
        determinism_check,
    }))
}

pub fn usage(message: &str) -> String {
    format!(
        "{message}\nusage: zircon_export_pack --profile <name> --manifest <assets.json> --pack <assets.zrpack> [--previous-pack <old.zrpack> --delta-pack <delta.zrpd>] [--report <path>] [--stage-output <dir>] [--pretty] [--determinism-check]"
    )
}

fn next_string(
    args: &mut impl Iterator<Item = OsString>,
    flag: &'static str,
) -> ExportPackResult<String> {
    args.next()
        .ok_or_else(|| ExportPackError::Usage(usage(&format!("missing value for {flag}"))))?
        .into_string()
        .map_err(|_| ExportPackError::Usage(usage(&format!("{flag} value must be UTF-8"))))
}

fn next_path(
    args: &mut impl Iterator<Item = OsString>,
    flag: &'static str,
) -> ExportPackResult<PathBuf> {
    Ok(PathBuf::from(next_string(args, flag)?))
}
