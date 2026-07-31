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
    // None uses the stable system-temporary cache shared by independent viewer launches.
    pub(crate) ibl_cache_dir: Option<PathBuf>,
    pub(crate) screenshot_path: Option<PathBuf>,
    pub(crate) renderdoc_capture_once: bool,
    pub(crate) renderdoc_dll: Option<PathBuf>,
    pub(crate) renderdoc_capture_path: Option<PathBuf>,
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
        let mut ibl_cache_dir = None;
        let mut screenshot_path = None;
        let mut renderdoc_capture_once = false;
        let mut renderdoc_dll = None;
        let mut renderdoc_capture_path = None;
        let mut exit_after_capture = false;
        let mut initial_yaw_degrees = 0.0;
        let mut initial_pitch_degrees = 0.0;
        let mut help_requested = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => help_requested = true,
                "--renderdoc-capture-once" => renderdoc_capture_once = true,
                "--renderdoc-dll" => {
                    let Some(path) = args.next() else {
                        return Err("--renderdoc-dll requires a DLL path".into());
                    };
                    let path = PathBuf::from(path);
                    if path.as_os_str().is_empty() {
                        return Err("--renderdoc-dll requires a DLL path".into());
                    }
                    renderdoc_dll = Some(path);
                }
                "--renderdoc-capture-path" => {
                    let Some(path) = args.next() else {
                        return Err("--renderdoc-capture-path requires a file template".into());
                    };
                    let path = PathBuf::from(path);
                    if path.as_os_str().is_empty() {
                        return Err("--renderdoc-capture-path requires a file template".into());
                    }
                    renderdoc_capture_path = Some(path);
                }
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
                "--ibl-cache-dir" => {
                    let Some(path) = args.next() else {
                        return Err("--ibl-cache-dir requires a directory path".into());
                    };
                    let path = PathBuf::from(path);
                    if path.as_os_str().is_empty() {
                        return Err("--ibl-cache-dir requires a directory path".into());
                    }
                    ibl_cache_dir = Some(path);
                }
                "--screenshot" => {
                    let Some(path) = args.next() else {
                        return Err("--screenshot requires a file path".into());
                    };
                    let path = PathBuf::from(path);
                    if path.as_os_str().is_empty() {
                        return Err("--screenshot requires a file path".into());
                    }
                    screenshot_path = Some(path);
                }
                _ if arg.starts_with('-') => {
                    return Err(format!("unknown argument `{arg}`").into());
                }
                _ => {
                    hdri_path = PathBuf::from(arg);
                }
            }
        }

        if !help_requested {
            require_radiance_hdr_path(&hdri_path)?;
            if renderdoc_capture_once {
                require_renderdoc_capture_support(cfg!(debug_assertions))?;
            }
            if renderdoc_dll.is_some() && !renderdoc_capture_once {
                return Err("--renderdoc-dll requires --renderdoc-capture-once".into());
            }
            if renderdoc_capture_path.is_some()
                && (renderdoc_dll.is_none() || !renderdoc_capture_once)
            {
                return Err(
                    "--renderdoc-capture-path requires --renderdoc-capture-once and --renderdoc-dll"
                        .into(),
                );
            }
        }

        Ok(Self {
            hdri_path,
            face_size,
            pmrem_face_size,
            ibl_cache_dir,
            screenshot_path,
            renderdoc_capture_once,
            renderdoc_dll,
            renderdoc_capture_path,
            exit_after_capture,
            initial_yaw_degrees,
            initial_pitch_degrees,
            help_requested,
        })
    }
}

fn require_renderdoc_capture_support(debug_assertions: bool) -> Result<(), Box<dyn Error>> {
    if debug_assertions {
        return Ok(());
    }

    Err(
        "--renderdoc-capture-once requires a debug viewer build; wgpu enables RenderDoc only with debug assertions"
            .into(),
    )
}

fn require_radiance_hdr_path(path: &Path) -> Result<(), Box<dyn Error>> {
    let is_hdr = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("hdr"));
    if is_hdr {
        return Ok(());
    }
    Err(format!(
        "--hdri must reference a Radiance .hdr image for the current viewer decoder, got {}",
        path.display()
    )
    .into())
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
        "zircon_shader_pbr_viewer [--hdri <path.hdr>]\n\
         Optional: --face-size <64|128|256|512|1024>\n\
         Optional: --pmrem-face-size <64|128|256|512|1024>\n\
         Optional: --ibl-cache-dir <directory>\n\
         Optional: --screenshot <path.png> (write the first Ready frame and exit)\n\
         Optional: --renderdoc-capture-once [--renderdoc-dll <renderdoc.dll> --renderdoc-capture-path <capture-template>] [--exit-after-capture]\n\
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
    use super::{ViewerConfig, require_renderdoc_capture_support};

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

    #[test]
    fn viewer_rejects_non_hdr_input_before_background_decode() {
        let error = ViewerConfig::from_args(["--hdri".to_owned(), "studio.exr".to_owned()])
            .expect_err("the HDR-only viewer must reject an unsupported input before spawning");

        assert!(error.to_string().contains("Radiance .hdr"));
    }

    #[test]
    fn viewer_accepts_case_insensitive_hdr_extension() {
        let config = ViewerConfig::from_args(["--hdri".to_owned(), "studio.HDR".to_owned()])
            .expect("Radiance HDR input should remain supported");

        assert_eq!(config.hdri_path, std::path::PathBuf::from("studio.HDR"));
    }

    #[test]
    fn viewer_accepts_an_explicit_ibl_cache_directory_for_cold_and_warm_runs() {
        let config = ViewerConfig::from_args([
            "--ibl-cache-dir".to_owned(),
            "E:/ZirconViewerCache".to_owned(),
        ])
        .expect("an external IBL cache directory should parse");

        assert_eq!(
            config.ibl_cache_dir,
            Some(std::path::PathBuf::from("E:/ZirconViewerCache"))
        );
    }

    #[test]
    fn viewer_accepts_a_ready_frame_screenshot_destination() {
        let config = ViewerConfig::from_args([
            "--screenshot".to_owned(),
            "E:/evidence/pbr-ready.png".to_owned(),
        ])
        .expect("a screenshot destination should parse");

        assert_eq!(
            config.screenshot_path,
            Some(std::path::PathBuf::from("E:/evidence/pbr-ready.png"))
        );
    }

    #[test]
    fn viewer_rejects_a_missing_screenshot_destination() {
        let error = ViewerConfig::from_args(["--screenshot".to_owned()])
            .expect_err("a screenshot option without a path must be rejected");

        assert!(
            error
                .to_string()
                .contains("--screenshot requires a file path")
        );
    }

    #[test]
    fn renderdoc_capture_requires_debug_assertions() {
        require_renderdoc_capture_support(true)
            .expect("a debug viewer must be able to use wgpu RenderDoc integration");

        let error = require_renderdoc_capture_support(false)
            .expect_err("a release viewer must reject a capture that wgpu cannot service");

        assert!(error.to_string().contains("debug viewer build"));
    }

    #[test]
    fn viewer_accepts_an_explicit_renderdoc_dll_for_capture() {
        let config = ViewerConfig::from_args([
            "--renderdoc-capture-once".to_owned(),
            "--renderdoc-dll".to_owned(),
            "D:/Tools/renderdoc/renderdoc.dll".to_owned(),
        ])
        .expect("a debug viewer capture may preload the injected RenderDoc DLL");

        assert_eq!(
            config.renderdoc_dll,
            Some(std::path::PathBuf::from("D:/Tools/renderdoc/renderdoc.dll"))
        );
    }

    #[test]
    fn viewer_rejects_renderdoc_dll_without_capture() {
        let error = ViewerConfig::from_args([
            "--renderdoc-dll".to_owned(),
            "D:/Tools/renderdoc/renderdoc.dll".to_owned(),
        ])
        .expect_err("preloading RenderDoc must remain scoped to an explicit capture");

        assert!(
            error
                .to_string()
                .contains("requires --renderdoc-capture-once")
        );
    }

    #[test]
    fn viewer_requires_dll_preload_for_a_renderdoc_capture_path() {
        let error = ViewerConfig::from_args([
            "--renderdoc-capture-once".to_owned(),
            "--renderdoc-capture-path".to_owned(),
            "E:/evidence/pbr-frame".to_owned(),
        ])
        .expect_err("the capture file template depends on a directly loaded RenderDoc API");

        assert!(
            error
                .to_string()
                .contains("requires --renderdoc-capture-once and --renderdoc-dll")
        );
    }
}
