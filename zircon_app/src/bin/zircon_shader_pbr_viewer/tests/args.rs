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

    let native = ViewerConfig::from_args(["--host-mode".to_owned(), "native-present".to_owned()])
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
