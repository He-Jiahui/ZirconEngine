use std::ffi::OsString;
use std::path::PathBuf;

use zircon_runtime::core::framework::render::{ShaderIdePreviewVariant, ShaderPassType};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderIdeEnvArgs {
    pub project_root: PathBuf,
    pub output_dir: Option<PathBuf>,
    pub pretty: bool,
    pub preview_variants: Vec<ShaderIdePreviewVariant>,
}

pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Option<ShaderIdeEnvArgs>, String> {
    let mut project_root = PathBuf::from(".");
    let mut output_dir = None;
    let mut pretty = false;
    let mut preview_variants = Vec::new();

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let arg_text = arg
            .to_str()
            .ok_or_else(|| "zircon_shader_ide_env expects UTF-8 command arguments".to_string())?;
        match arg_text {
            "-h" | "--help" => return Ok(None),
            "--project-root" => project_root = next_path(&mut args, "--project-root")?,
            "--out-dir" => output_dir = Some(next_path(&mut args, "--out-dir")?),
            "--pretty" => pretty = true,
            "--variants" => preview_variants.push(ShaderIdePreviewVariant::default_forward()),
            "--variant" => {
                let spec = next_string(&mut args, "--variant")?;
                preview_variants.push(parse_preview_variant_spec(&spec)?);
            }
            unknown => return Err(usage(&format!("unknown argument {unknown}"))),
        }
    }

    Ok(Some(ShaderIdeEnvArgs {
        project_root,
        output_dir,
        pretty,
        preview_variants,
    }))
}

pub fn usage(message: &str) -> String {
    format!(
        "{message}\nusage: zircon_shader_ide_env [--project-root <dir>] [--out-dir <dir>] [--pretty] [--variants] [--variant <pass[:options=bits]>]"
    )
}

fn next_path(
    args: &mut impl Iterator<Item = OsString>,
    flag: &'static str,
) -> Result<PathBuf, String> {
    args.next()
        .ok_or_else(|| usage(&format!("missing value for {flag}")))?
        .into_string()
        .map(PathBuf::from)
        .map_err(|_| usage(&format!("{flag} value must be UTF-8")))
}

fn next_string(
    args: &mut impl Iterator<Item = OsString>,
    flag: &'static str,
) -> Result<String, String> {
    args.next()
        .ok_or_else(|| usage(&format!("missing value for {flag}")))?
        .into_string()
        .map_err(|_| usage(&format!("{flag} value must be UTF-8")))
}

fn parse_preview_variant_spec(spec: &str) -> Result<ShaderIdePreviewVariant, String> {
    let spec = spec.trim();
    if spec.eq_ignore_ascii_case("default") {
        return Ok(ShaderIdePreviewVariant::default_forward());
    }
    let (pass_token, option_bits) = match spec.split_once(':') {
        Some((pass_token, suffix)) => (pass_token, parse_preview_variant_suffix(suffix)?),
        None => (spec, 0),
    };
    let pass_type = parse_shader_pass_type(pass_token)
        .ok_or_else(|| usage(&format!("unknown shader preview pass {pass_token}")))?;
    Ok(ShaderIdePreviewVariant::new(pass_type, option_bits))
}

fn parse_preview_variant_suffix(suffix: &str) -> Result<u32, String> {
    let (key, value) = suffix
        .split_once('=')
        .ok_or_else(|| usage("shader preview variant suffix must be options=<bits>"))?;
    if key.trim() != "options" {
        return Err(usage(&format!(
            "unknown shader preview variant suffix {}",
            key.trim()
        )));
    }
    parse_u32_bits(value.trim())
}

fn parse_u32_bits(value: &str) -> Result<u32, String> {
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u32::from_str_radix(hex, 16)
    } else {
        value.parse()
    }
    .map_err(|_| usage(&format!("invalid material option bits {value}")))
}

fn parse_shader_pass_type(token: &str) -> Option<ShaderPassType> {
    match token.trim().to_ascii_lowercase().as_str() {
        "forward" => Some(ShaderPassType::Forward),
        "gbuffer" => Some(ShaderPassType::GBuffer),
        "depth_prepass" | "depth" => Some(ShaderPassType::DepthPrepass),
        "shadow" => Some(ShaderPassType::Shadow),
        "velocity" => Some(ShaderPassType::Velocity),
        "taa_reactive_mask" | "taa" => Some(ShaderPassType::TaaReactiveMask),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_accepts_variants_flag() {
        let args = parse([
            OsString::from("--project-root"),
            OsString::from("project"),
            OsString::from("--out-dir"),
            OsString::from("out"),
            OsString::from("--variants"),
        ])
        .unwrap()
        .expect("parsed args");

        assert_eq!(args.project_root, PathBuf::from("project"));
        assert_eq!(args.output_dir, Some(PathBuf::from("out")));
        assert_eq!(
            args.preview_variants,
            vec![ShaderIdePreviewVariant::default_forward()]
        );
    }

    #[test]
    fn parse_accepts_non_default_preview_variant_specs() {
        let args = parse([
            OsString::from("--variant"),
            OsString::from("gbuffer:options=0x1"),
            OsString::from("--variant"),
            OsString::from("shadow"),
        ])
        .unwrap()
        .expect("parsed args");

        assert_eq!(
            args.preview_variants,
            vec![
                ShaderIdePreviewVariant::new(ShaderPassType::GBuffer, 1),
                ShaderIdePreviewVariant::new(ShaderPassType::Shadow, 0),
            ]
        );
    }
}
