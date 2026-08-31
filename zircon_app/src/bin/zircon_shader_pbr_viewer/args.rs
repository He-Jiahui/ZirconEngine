use std::error::Error;
use std::path::{Path, PathBuf};

use crate::gpu_timing_evidence::validate_gpu_timing_report_output;
use crate::material_fixture::ViewerMaterialFixture;

pub(crate) const MIN_FACE_SIZE: u32 = 64;
pub(crate) const MAX_FACE_SIZE: u32 = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ViewerHostMode {
    OffscreenDiagnostic,
    NativePresent,
}

impl ViewerHostMode {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::OffscreenDiagnostic => "offscreen-diagnostic",
            Self::NativePresent => "native-present",
        }
    }

    pub(crate) const fn capture_target(self) -> &'static str {
        match self {
            Self::OffscreenDiagnostic => "offscreen-scene-renderer-cpu-readback",
            Self::NativePresent => "native-viewport-surface",
        }
    }

    pub(crate) const fn gpu_scene_surface_present_count(self) -> u32 {
        match self {
            Self::OffscreenDiagnostic => 0,
            Self::NativePresent => 1,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ViewerConfig {
    pub(crate) hdri_path: PathBuf,
    // None keeps import sizing tied to the decoded HDRI instead of a viewer-only default.
    pub(crate) face_size: Option<u32>,
    // None gives PMREM the resolved source face size while retaining an independent override.
    pub(crate) pmrem_face_size: Option<u32>,
    pub(crate) work_dir: PathBuf,
    // None uses the stable cache below work_dir shared by independent viewer launches.
    pub(crate) ibl_cache_dir: Option<PathBuf>,
    pub(crate) screenshot_path: Option<PathBuf>,
    pub(crate) evidence_identity_path: Option<PathBuf>,
    pub(crate) gpu_timing_report_path: Option<PathBuf>,
    pub(crate) material_fixture: ViewerMaterialFixture,
    pub(crate) host_mode: ViewerHostMode,
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
        let mut work_dir = default_work_dir();
        let mut ibl_cache_dir = None;
        let mut screenshot_path = None;
        let mut evidence_identity_path = None;
        let mut gpu_timing_report_path = None;
        let mut material_fixture = ViewerMaterialFixture::default();
        let mut requested_host_mode = None;
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
                "--work-dir" => {
                    let Some(path) = args.next() else {
                        return Err("--work-dir requires a directory path".into());
                    };
                    let path = PathBuf::from(path);
                    if path.as_os_str().is_empty() {
                        return Err("--work-dir requires a directory path".into());
                    }
                    work_dir = path;
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
                "--evidence-identity" => {
                    let Some(path) = args.next() else {
                        return Err("--evidence-identity requires a JSON file path".into());
                    };
                    let path = PathBuf::from(path);
                    if path.as_os_str().is_empty() {
                        return Err("--evidence-identity requires a JSON file path".into());
                    }
                    evidence_identity_path = Some(path);
                }
                "--gpu-timing-report" => {
                    let Some(path) = args.next() else {
                        return Err("--gpu-timing-report requires a file path".into());
                    };
                    let path = PathBuf::from(path);
                    if path.as_os_str().is_empty() {
                        return Err("--gpu-timing-report requires a file path".into());
                    }
                    gpu_timing_report_path = Some(path);
                }
                "--material-fixture" => {
                    let Some(value) = args.next() else {
                        return Err(
                            "--material-fixture requires metal-mirror or dielectric-ior".into()
                        );
                    };
                    material_fixture = ViewerMaterialFixture::from_cli_value(&value)?;
                }
                "--host-mode" => {
                    let Some(value) = args.next() else {
                        return Err(
                            "--host-mode requires offscreen-diagnostic or native-present".into(),
                        );
                    };
                    requested_host_mode = Some(parse_host_mode(&value)?);
                }
                _ if arg.starts_with('-') => {
                    return Err(format!("unknown argument `{arg}`").into());
                }
                _ => {
                    hdri_path = PathBuf::from(arg);
                }
            }
        }

        let host_mode = requested_host_mode.unwrap_or_else(|| {
            if screenshot_path.is_some() {
                ViewerHostMode::OffscreenDiagnostic
            } else {
                ViewerHostMode::NativePresent
            }
        });

        if !help_requested {
            require_radiance_hdr_path(&hdri_path)?;
            work_dir = resolve_non_c_artifact_path("--work-dir", &work_dir)?;
            ibl_cache_dir = ibl_cache_dir
                .as_deref()
                .map(|path| resolve_non_c_artifact_path("--ibl-cache-dir", path))
                .transpose()?;
            screenshot_path = screenshot_path
                .as_deref()
                .map(|path| resolve_non_c_artifact_path("--screenshot", path))
                .transpose()?;
            evidence_identity_path = evidence_identity_path
                .as_deref()
                .map(|path| resolve_non_c_artifact_path("--evidence-identity", path))
                .transpose()?;
            gpu_timing_report_path = gpu_timing_report_path
                .as_deref()
                .map(|path| resolve_non_c_artifact_path("--gpu-timing-report", path))
                .transpose()?;
            renderdoc_capture_path = renderdoc_capture_path
                .as_deref()
                .map(|path| resolve_non_c_artifact_path("--renderdoc-capture-path", path))
                .transpose()?;
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
            if gpu_timing_report_path.is_some() && screenshot_path.is_none() {
                return Err("--gpu-timing-report requires --screenshot".into());
            }
            match host_mode {
                ViewerHostMode::OffscreenDiagnostic if screenshot_path.is_none() => {
                    return Err("--host-mode offscreen-diagnostic requires --screenshot".into());
                }
                ViewerHostMode::NativePresent if screenshot_path.is_some() => {
                    return Err(
                        "--host-mode native-present forbids --screenshot and CPU readback evidence"
                            .into(),
                    );
                }
                _ => {}
            }
            if screenshot_path.is_some() && evidence_identity_path.is_none() {
                return Err("--screenshot requires --evidence-identity <path.json>".into());
            }
            if evidence_identity_path.is_some() && screenshot_path.is_none() {
                return Err("--evidence-identity requires --screenshot".into());
            }
            if let (Some(screenshot_path), Some(gpu_timing_report_path)) = (
                screenshot_path.as_deref(),
                gpu_timing_report_path.as_deref(),
            ) {
                validate_gpu_timing_report_output(screenshot_path, gpu_timing_report_path)?;
            }
        }

        Ok(Self {
            hdri_path,
            face_size,
            pmrem_face_size,
            work_dir,
            ibl_cache_dir,
            screenshot_path,
            evidence_identity_path,
            gpu_timing_report_path,
            material_fixture,
            host_mode,
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

fn parse_host_mode(value: &str) -> Result<ViewerHostMode, Box<dyn Error>> {
    match value {
        "offscreen-diagnostic" => Ok(ViewerHostMode::OffscreenDiagnostic),
        "native-present" => Ok(ViewerHostMode::NativePresent),
        "packaged-product" => Err(
            "--host-mode packaged-product is not implemented by zircon_shader_pbr_viewer; use the product entrypoint"
                .into(),
        ),
        _ => Err(format!(
            "--host-mode must be offscreen-diagnostic or native-present, got {value}"
        )
        .into()),
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

fn resolve_non_c_artifact_path(option: &str, path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| format!("failed to resolve {option}: {error}"))?
            .join(path)
    };
    if is_c_drive_path(path) || is_c_drive_path(&absolute_path) {
        return Err(format!(
            "{option} must write artifacts outside C:, got {}",
            absolute_path.display()
        )
        .into());
    }

    let mut existing_ancestor = None;
    for ancestor in absolute_path.ancestors() {
        match std::fs::symlink_metadata(ancestor) {
            Ok(metadata) => {
                if is_reparse_point(&metadata) {
                    return Err(format!(
                        "{option} must not traverse a reparse point: {}",
                        ancestor.display()
                    )
                    .into());
                }
                if existing_ancestor.is_none() {
                    existing_ancestor = Some(ancestor);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "failed to inspect {option} ancestor {}: {error}",
                    ancestor.display()
                )
                .into());
            }
        }
    }
    let existing_ancestor = existing_ancestor.ok_or_else(|| {
        format!(
            "failed to find an existing ancestor while resolving {option}: {}",
            absolute_path.display()
        )
    })?;
    let canonical_ancestor = std::fs::canonicalize(existing_ancestor).map_err(|error| {
        format!(
            "failed to canonicalize {option} ancestor {}: {error}",
            existing_ancestor.display()
        )
    })?;
    if is_c_drive_path(&canonical_ancestor) {
        return Err(format!(
            "{option} must write artifacts outside C:, got {}",
            canonical_ancestor.display()
        )
        .into());
    }
    let unresolved_suffix = absolute_path
        .strip_prefix(existing_ancestor)
        .map_err(|error| {
            format!(
                "failed to preserve {option} path below {}: {error}",
                existing_ancestor.display()
            )
        })?;
    let resolved_path = normalize_artifact_path(canonical_ancestor.join(unresolved_suffix));
    if is_c_drive_path(&resolved_path) {
        return Err(format!(
            "{option} must write artifacts outside C:, got {}",
            resolved_path.display()
        )
        .into());
    }
    Ok(resolved_path)
}

fn normalize_artifact_path(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        let value = path.as_os_str().to_string_lossy();
        if let Some(unc_path) = value.strip_prefix("\\\\?\\UNC\\") {
            return PathBuf::from(format!("\\\\{unc_path}"));
        }
        if let Some(normal_path) = value.strip_prefix("\\\\?\\") {
            return PathBuf::from(normal_path);
        }
    }
    path
}

fn is_reparse_point(metadata: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;

        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
        return metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0;
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

fn is_c_drive_path(path: &Path) -> bool {
    let path = path.to_string_lossy();
    let path = path.strip_prefix(r"\\?\").unwrap_or(&path);
    let bytes = path.as_bytes();
    bytes.len() >= 2 && matches!(bytes[0], b'C' | b'c') && bytes[1] == b':'
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
    workspace_root()
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("shader")
        .join("assets")
        .join("polyhaven_lakes_2k.hdr")
}

pub(crate) fn default_work_dir() -> PathBuf {
    default_work_dir_for_workspace(&workspace_root())
}

fn default_work_dir_for_workspace(workspace_root: &Path) -> PathBuf {
    let evidence_root = workspace_root
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("shader")
        .join("zircon_shader_pbr_viewer_work");
    if is_c_drive_path(&evidence_root) {
        PathBuf::from("D:/ZirconEngineArtifacts/zircon_shader_pbr_viewer_work")
    } else {
        evidence_root
    }
}

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or(manifest_dir)
}

pub(crate) fn print_help() {
    println!(
        "zircon_shader_pbr_viewer [--hdri <path.hdr>]\n\
         Optional: --face-size <64|128|256|512|1024>\n\
         Optional: --pmrem-face-size <64|128|256|512|1024>\n\
         Optional: --work-dir <directory>\n\
         Optional: --ibl-cache-dir <directory>\n\
         Optional: --screenshot <path.png> --evidence-identity <path.json> (write the first Ready frame and its bound provenance, then exit)\n\
         Optional: --gpu-timing-report <path.txt> (requires --screenshot; write GPU timing after nonblocking readback)\n\
         Optional: --material-fixture <metal-mirror|dielectric-ior> (default: metal-mirror)\n\
         Optional: --host-mode <offscreen-diagnostic|native-present> (screenshot selects offscreen-diagnostic when omitted)\n\
         Optional: --renderdoc-capture-once [--renderdoc-dll <renderdoc.dll> --renderdoc-capture-path <capture-template>] [--exit-after-capture]\n\
         Optional: --yaw <degrees> --pitch <degrees>\n\
         Left mouse drag: orbit camera\n\
         Mouse wheel: zoom\n\
         Default HDRI: {}\n\
         Default work directory: {}\n\
         Default source face size: automatic from HDRI height (64..1024)\n\
         Default PMREM face size: resolved source face size",
        default_hdri_path().display(),
        default_work_dir().display()
    );
}

#[cfg(test)]
mod tests {
    use super::{
        default_work_dir, default_work_dir_for_workspace, require_renderdoc_capture_support,
        resolve_non_c_artifact_path, ViewerConfig, ViewerHostMode,
    };

    #[test]
    fn default_face_size_uses_hdri_native_angular_resolution() {
        let config = ViewerConfig::from_args([]).expect("default viewer arguments should parse");

        assert_eq!(config.face_size, None);
    }

    #[test]
    fn material_fixture_defaults_to_the_existing_metal_mirror_baseline() {
        let config = ViewerConfig::from_args([]).expect("default viewer arguments should parse");

        assert_eq!(
            config.material_fixture,
            crate::material_fixture::ViewerMaterialFixture::MetalMirror
        );
    }

    #[test]
    fn dielectric_ior_fixture_is_an_explicit_closed_cli_choice() {
        let config =
            ViewerConfig::from_args(["--material-fixture".to_owned(), "dielectric-ior".to_owned()])
                .expect("the IOR fixture should parse");
        assert_eq!(
            config.material_fixture,
            crate::material_fixture::ViewerMaterialFixture::DielectricIor
        );

        let error =
            ViewerConfig::from_args(["--material-fixture".to_owned(), "unsupported".to_owned()])
                .expect_err("the fixture mode must stay closed");
        assert!(error.to_string().contains("metal-mirror or dielectric-ior"));
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
    fn viewer_accepts_an_explicit_work_directory() {
        let config = ViewerConfig::from_args([
            "--work-dir".to_owned(),
            "E:/shader-pbr-viewer-work".to_owned(),
        ])
        .expect("an external viewer work directory should parse");

        assert_eq!(
            config.work_dir,
            std::path::PathBuf::from("E:/shader-pbr-viewer-work")
        );
    }

    #[test]
    fn default_work_directory_keeps_generated_viewer_artifacts_in_the_repository_evidence_root() {
        let work_dir = default_work_dir();
        let normalized = work_dir.to_string_lossy().replace('\\', "/");

        assert!(
            !normalized.starts_with("C:/"),
            "the default viewer work directory must not create artifacts on C:, got {normalized}"
        );
        assert!(
            normalized.ends_with("docs/tests/runtime/shader/zircon_shader_pbr_viewer_work")
                || normalized == "D:/ZirconEngineArtifacts/zircon_shader_pbr_viewer_work",
            "default work directory must use the evidence root or the non-C fallback, got {normalized}"
        );
    }

    #[test]
    fn c_drive_workspace_uses_the_d_drive_artifact_fallback() {
        assert_eq!(
            default_work_dir_for_workspace(std::path::Path::new("C:/ZirconEngine")),
            std::path::PathBuf::from("D:/ZirconEngineArtifacts/zircon_shader_pbr_viewer_work")
        );
    }

    #[test]
    fn viewer_rejects_c_drive_artifact_destinations() {
        for (option, path) in [
            ("--work-dir", "C:/viewer-work"),
            ("--ibl-cache-dir", "C:/ibl-cache"),
            ("--screenshot", "C:/evidence/ready.png"),
            ("--evidence-identity", "C:/evidence/identity.json"),
            ("--gpu-timing-report", "C:/evidence/timing.txt"),
            ("--renderdoc-capture-path", "C:/evidence/frame"),
        ] {
            let mut args = vec![
                "--screenshot".to_owned(),
                "E:/evidence/ready.png".to_owned(),
            ];
            args.push(option.to_owned());
            args.push(path.to_owned());

            let error =
                ViewerConfig::from_args(args).expect_err("viewer artifacts must stay outside C:");
            assert!(
                error.to_string().contains("outside C:"),
                "{option} should identify the forbidden artifact drive: {error}"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn viewer_rejects_artifact_paths_below_a_c_drive_reparse_point() {
        let root = crate::work_paths::viewer_test_artifact_root("c-drive-reparse-point");
        let reparse_path = root.join("redirected-output");
        match std::os::windows::fs::symlink_dir(r"C:\Windows", &reparse_path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                std::fs::remove_dir_all(root).expect("test artifact root should be removed");
                return;
            }
            Err(error) => panic!("create C-drive test reparse point: {error}"),
        }

        let error = resolve_non_c_artifact_path("--screenshot", &reparse_path.join("ready.png"))
            .expect_err("an artifact path below a reparse point must be rejected");
        assert!(error.to_string().contains("reparse point"));
        std::fs::remove_dir_all(root).expect("test artifact root should be removed");
    }

    #[test]
    fn viewer_rejects_a_missing_work_directory() {
        let error = ViewerConfig::from_args(["--work-dir".to_owned()])
            .expect_err("a work directory option without a path must be rejected");

        assert!(error
            .to_string()
            .contains("--work-dir requires a directory path"));
    }

    #[test]
    fn viewer_accepts_a_ready_frame_screenshot_destination() {
        let config = ViewerConfig::from_args([
            "--screenshot".to_owned(),
            "E:/evidence/pbr-ready.png".to_owned(),
            "--evidence-identity".to_owned(),
            "E:/evidence/identity.json".to_owned(),
        ])
        .expect("a screenshot destination should parse");

        assert_eq!(
            config.screenshot_path,
            Some(std::path::PathBuf::from("E:/evidence/pbr-ready.png"))
        );
        assert_eq!(config.host_mode, ViewerHostMode::OffscreenDiagnostic);
        assert_eq!(
            config.evidence_identity_path,
            Some(std::path::PathBuf::from("E:/evidence/identity.json"))
        );
    }

    #[test]
    fn host_mode_makes_cpu_readback_and_native_presentation_mutually_exclusive() {
        let offscreen = ViewerConfig::from_args([
            "--host-mode".to_owned(),
            "offscreen-diagnostic".to_owned(),
            "--screenshot".to_owned(),
            "E:/evidence/pbr-ready.png".to_owned(),
            "--evidence-identity".to_owned(),
            "E:/evidence/identity.json".to_owned(),
        ])
        .expect("offscreen diagnostic evidence must be explicit and self-consistent");
        assert_eq!(offscreen.host_mode, ViewerHostMode::OffscreenDiagnostic);

        let native =
            ViewerConfig::from_args(["--host-mode".to_owned(), "native-present".to_owned()])
                .expect("native presentation must remain available without an offscreen artifact");
        assert_eq!(native.host_mode, ViewerHostMode::NativePresent);

        let native_with_screenshot = ViewerConfig::from_args([
            "--host-mode".to_owned(),
            "native-present".to_owned(),
            "--screenshot".to_owned(),
            "E:/evidence/pbr-ready.png".to_owned(),
        ])
        .expect_err("a native claim cannot use the CPU readback capture path");
        assert!(native_with_screenshot
            .to_string()
            .contains("native-present forbids --screenshot"));

        let offscreen_without_screenshot =
            ViewerConfig::from_args(["--host-mode".to_owned(), "offscreen-diagnostic".to_owned()])
                .expect_err("offscreen diagnostic mode needs a committed screenshot artifact");
        assert!(offscreen_without_screenshot
            .to_string()
            .contains("offscreen-diagnostic requires --screenshot"));

        let packaged_product =
            ViewerConfig::from_args(["--host-mode".to_owned(), "packaged-product".to_owned()])
                .expect_err("the standalone diagnostic viewer must not claim product composition");
        assert!(packaged_product
            .to_string()
            .contains("not implemented by zircon_shader_pbr_viewer"));
    }

    #[test]
    fn viewer_rejects_a_missing_screenshot_destination() {
        let error = ViewerConfig::from_args(["--screenshot".to_owned()])
            .expect_err("a screenshot option without a path must be rejected");

        assert!(error
            .to_string()
            .contains("--screenshot requires a file path"));
    }

    #[test]
    fn gpu_timing_report_requires_a_screenshot_and_retains_its_output_path() {
        let error = ViewerConfig::from_args([
            "--gpu-timing-report".to_owned(),
            "E:/evidence/pbr-gpu-timing.txt".to_owned(),
        ])
        .expect_err("GPU timing needs a concrete screenshot frame to identify");
        assert!(error.to_string().contains("requires --screenshot"));

        let config = ViewerConfig::from_args([
            "--screenshot".to_owned(),
            "E:/evidence/pbr-ready.png".to_owned(),
            "--evidence-identity".to_owned(),
            "E:/evidence/identity.json".to_owned(),
            "--gpu-timing-report".to_owned(),
            "E:/evidence/pbr-gpu-timing.txt".to_owned(),
        ])
        .expect("a screenshot-scoped GPU timing report should parse");
        assert_eq!(
            config.gpu_timing_report_path,
            Some(std::path::PathBuf::from("E:/evidence/pbr-gpu-timing.txt"))
        );
    }

    #[test]
    fn viewer_requires_a_screenshot_and_identity_manifest_pair() {
        let screenshot_without_identity = ViewerConfig::from_args([
            "--screenshot".to_owned(),
            "E:/evidence/pbr-ready.png".to_owned(),
        ])
        .expect_err("a screenshot without an identity manifest must be rejected");
        assert!(screenshot_without_identity
            .to_string()
            .contains("--screenshot requires --evidence-identity"));

        let identity_without_screenshot = ViewerConfig::from_args([
            "--evidence-identity".to_owned(),
            "E:/evidence/identity.json".to_owned(),
        ])
        .expect_err("an identity manifest without a screenshot must be rejected");
        assert!(identity_without_screenshot
            .to_string()
            .contains("--evidence-identity requires --screenshot"));
    }

    #[test]
    fn gpu_timing_report_rejects_the_ready_png_or_its_sidecar_as_an_output_target() {
        for (screenshot_path, report_path) in [
            (
                "E:/evidence/pbr-ready.png",
                "E:/evidence/frames/../pbr-ready.png",
            ),
            ("E:/evidence/pbr-ready.png", "E:/evidence/pbr-ready.png.txt"),
            ("E:/Evidence/PBR-READY.PNG", "e:/evidence/pbr-ready.png"),
        ] {
            let error = ViewerConfig::from_args([
                "--screenshot".to_owned(),
                screenshot_path.to_owned(),
                "--evidence-identity".to_owned(),
                "E:/evidence/identity.json".to_owned(),
                "--gpu-timing-report".to_owned(),
                report_path.to_owned(),
            ])
            .expect_err("timing evidence must not overwrite the Ready PNG or sidecar");
            assert!(
                error.to_string().contains("must not overwrite"),
                "{report_path} should be rejected as a Ready evidence collision: {error}"
            );
        }
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

        assert!(error
            .to_string()
            .contains("requires --renderdoc-capture-once"));
    }

    #[test]
    fn viewer_requires_dll_preload_for_a_renderdoc_capture_path() {
        let error = ViewerConfig::from_args([
            "--renderdoc-capture-once".to_owned(),
            "--renderdoc-capture-path".to_owned(),
            "E:/evidence/pbr-frame".to_owned(),
        ])
        .expect_err("the capture file template depends on a directly loaded RenderDoc API");

        assert!(error
            .to_string()
            .contains("requires --renderdoc-capture-once and --renderdoc-dll"));
    }
}
