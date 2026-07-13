use std::error::Error;
use std::path::{Path, PathBuf};

pub(crate) const MIN_FACE_SIZE: u32 = 64;
pub(crate) const MAX_FACE_SIZE: u32 = 1024;

#[derive(Clone, Debug)]
pub(crate) struct ViewerConfig {
    pub(crate) hdri_path: PathBuf,
    // None keeps import sizing tied to the decoded HDRI instead of a viewer-only default.
    pub(crate) face_size: Option<u32>,
    // None gives PMREM the resolved source face size while retaining an independent override.
    pub(crate) pmrem_face_size: Option<u32>,
    pub(crate) renderdoc_capture_once: bool,
    pub(crate) exit_after_capture: bool,
    pub(crate) initial_yaw_degrees: f32,
    pub(crate) initial_pitch_degrees: f32,
    pub(crate) help_requested: bool,
}

impl ViewerConfig {
    pub(crate) fn from_args(
        args: impl IntoIterator<Item = String>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut hdri_path = default_hdri_path();
        let mut face_size = None;
        let mut pmrem_face_size = None;
        let mut renderdoc_capture_once = false;
        let mut exit_after_capture = false;
        let mut initial_yaw_degrees = 0.0;
        let mut initial_pitch_degrees = 0.0;
        let mut help_requested = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => help_requested = true,
                "--renderdoc-capture-once" => renderdoc_capture_once = true,
                "--exit-after-capture" => exit_after_capture = true,
                "--yaw" => {
                    initial_yaw_degrees = parse_angle("--yaw", args.next())?;
                }
                "--pitch" => {
                    initial_pitch_degrees = parse_angle("--pitch", args.next())?;
                }
                "--hdri" => {
                    let Some(path) = args.next() else {
                        return Err("--hdri requires a file path".into());
                    };
                    hdri_path = PathBuf::from(path);
                }
                "--face-size" => {
                    let Some(value) = args.next() else {
                        return Err("--face-size requires a pixel value".into());
                    };
                    face_size = Some(parse_face_size(&value)?);
                }
                "--pmrem-face-size" => {
                    let Some(value) = args.next() else {
                        return Err("--pmrem-face-size requires a pixel value".into());
                    };
                    pmrem_face_size = Some(parse_face_size_named("--pmrem-face-size", &value)?);
                }
                _ if arg.starts_with('-') => {
                    return Err(format!("unknown argument `{arg}`").into());
                }
                _ => {
                    hdri_path = PathBuf::from(arg);
                }
            }
        }

        Ok(Self {
            hdri_path,
            face_size,
            pmrem_face_size,
            renderdoc_capture_once,
            exit_after_capture,
            initial_yaw_degrees,
            initial_pitch_degrees,
            help_requested,
        })
    }
}

fn parse_angle(name: &str, value: Option<String>) -> Result<f32, Box<dyn Error>> {
    let Some(value) = value else {
        return Err(format!("{name} requires a finite degree value").into());
    };
    let value = value.parse::<f32>()?;
    if !value.is_finite() {
        return Err(format!("{name} requires a finite degree value").into());
    }
    Ok(value)
}

fn parse_face_size(value: &str) -> Result<u32, Box<dyn Error>> {
    parse_face_size_named("--face-size", value)
}

fn parse_face_size_named(name: &str, value: &str) -> Result<u32, Box<dyn Error>> {
    let parsed = value.parse::<u32>()?;
    if !(MIN_FACE_SIZE..=MAX_FACE_SIZE).contains(&parsed) || !parsed.is_power_of_two() {
        return Err(format!(
            "{name} must be a power of two between {MIN_FACE_SIZE} and {MAX_FACE_SIZE}, got {parsed}"
        )
        .into());
    }
    Ok(parsed)
}

pub(crate) fn default_hdri_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or(manifest_dir);
    workspace_root
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("shader")
        .join("assets")
        .join("polyhaven_lakes_2k.hdr")
}

pub(crate) fn print_help() {
    println!(
        "zircon_shader_pbr_viewer [--hdri <path>]\n\
         Optional: --face-size <64|128|256|512|1024>\n\
         Optional: --pmrem-face-size <64|128|256|512|1024>\n\
         Optional: --renderdoc-capture-once [--exit-after-capture]\n\
         Optional: --yaw <degrees> --pitch <degrees>\n\
         Left mouse drag: orbit camera\n\
         Mouse wheel: zoom\n\
         Default HDRI: {}\n\
         Default source face size: automatic from HDRI height (64..1024)\n\
         Default PMREM face size: resolved source face size",
        default_hdri_path().display()
    );
}

#[cfg(test)]
mod tests {
    use super::ViewerConfig;

    #[test]
    fn default_face_size_uses_hdri_native_angular_resolution() {
        let config = ViewerConfig::from_args([]).expect("default viewer arguments should parse");

        assert_eq!(config.face_size, None);
    }

    #[test]
    fn explicit_face_size_accepts_plan_maximum() {
        let config = ViewerConfig::from_args(["--face-size".to_owned(), "1024".to_owned()])
            .expect("the Shader 06 source cubemap maximum should parse");

        assert_eq!(config.face_size, Some(1024));
    }

    #[test]
    fn default_pmrem_face_size_follows_resolved_source_size() {
        let config = ViewerConfig::from_args([]).expect("default viewer arguments should parse");

        assert_eq!(config.pmrem_face_size, None);
    }

    #[test]
    fn explicit_pmrem_face_size_accepts_plan_maximum() {
        let config = ViewerConfig::from_args(["--pmrem-face-size".to_owned(), "1024".to_owned()])
            .expect("the Shader 06 PMREM result-size maximum should parse");

        assert_eq!(config.pmrem_face_size, Some(1024));
    }

    #[test]
    fn exact_multiview_angles_accept_signed_degrees() {
        let config = ViewerConfig::from_args([
            "--yaw".to_owned(),
            "-120".to_owned(),
            "--pitch".to_owned(),
            "120".to_owned(),
        ])
        .expect("signed finite viewer angles should parse");

        assert_eq!(config.initial_yaw_degrees, -120.0);
        assert_eq!(config.initial_pitch_degrees, 120.0);
    }

    #[test]
    fn exact_multiview_angles_reject_non_finite_values() {
        let error = ViewerConfig::from_args(["--yaw".to_owned(), "NaN".to_owned()])
            .expect_err("non-finite viewer angles must be rejected");

        assert!(error.to_string().contains("finite degree value"));
    }
}
