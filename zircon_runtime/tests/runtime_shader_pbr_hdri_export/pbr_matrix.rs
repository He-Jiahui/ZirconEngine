use super::*;
use std::io::{self, Read, Write};
use zircon_runtime::core::framework::render::IBL_BAKE_ALGORITHM_VERSION;

const PBR_MATRIX_HDRI_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_8x8_cmft_pmrem_reflection_20260710.png";
const PBR_MATRIX_HDRI_2K_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_2k_8x8_cmft_pmrem_reflection_20260710.png";
const GRAZING_SYMMETRY_MAX_MEAN_RELATIVE_DELTA: f32 = 0.05;
const GRAZING_SYMMETRY_MAX_PER_RADIUS_RELATIVE_DELTA: f32 = 0.10;
const GRAZING_SYMMETRY_RADII: [f32; 5] = [0.65, 0.75, 0.83, 0.89, 0.93];
const PBR_MATRIX_EVIDENCE_FINGERPRINT_BUFFER_BYTES: usize = 64 * 1024;

#[test]
fn runtime_shader_pbr_matrix_contract_uses_requested_eight_by_eight_grid() {
    assert_eq!(PBR_MATRIX_DIMENSION, 8);
    assert_eq!(PBR_MATRIX_DIMENSION * PBR_MATRIX_DIMENSION, 64);
    assert_eq!(pbr_matrix_axis_value(0), 0.0);
    assert_eq!(pbr_matrix_axis_value(PBR_MATRIX_DIMENSION - 1), 1.0);
}

#[test]
fn runtime_shader_pbr_quantitative_evidence_names_include_the_current_bake_version() {
    let (screenshot_name, report_name) =
        pbr_matrix_quantitative_evidence_names_for_provenance("test-executable", "test-input");
    let version = format!("v{IBL_BAKE_ALGORITHM_VERSION}");

    assert!(screenshot_name.contains(&version));
    assert!(report_name.contains(&version));
    assert!(screenshot_name.ends_with(".png"));
    assert!(report_name.ends_with(".txt"));
    assert!(!screenshot_name.contains("angular0003"));
    assert!(!report_name.contains("angular0003"));
}

#[test]
fn runtime_shader_pbr_quantitative_evidence_names_distinguish_executables_and_inputs() {
    let (first_screenshot, first_report) =
        pbr_matrix_quantitative_evidence_names_for_provenance("executable-a", "input-a");
    let (second_screenshot, second_report) =
        pbr_matrix_quantitative_evidence_names_for_provenance("executable-b", "input-b");

    assert_ne!(first_screenshot, second_screenshot);
    assert_ne!(first_report, second_report);
    assert!(first_screenshot.contains("executable-a"));
    assert!(second_report.contains("input-b"));
}

#[test]
fn runtime_shader_pbr_quantitative_evidence_rejects_a_runtime_input_change_before_commit() {
    let error = pbr_matrix_require_matching_runtime_input_fingerprint("before", "after")
        .expect_err("a changed HDR input fingerprint must reject the evidence commit");

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn runtime_shader_pbr_quantitative_evidence_fingerprint_changes_with_hdr_input_bytes() {
    let root = unique_temp_project_root("shader06_quantitative_evidence_hdr_fingerprint");
    fs::create_dir_all(&root).expect("create HDR fingerprint test directory");
    let input = root.join("environment.hdr");
    fs::write(&input, b"first HDR input").expect("write first HDR fingerprint input");
    let first = pbr_matrix_fingerprint_file(&input).expect("hash first HDR fingerprint input");

    fs::write(&input, b"second HDR input").expect("write second HDR fingerprint input");
    let second = pbr_matrix_fingerprint_file(&input).expect("hash second HDR fingerprint input");

    assert_ne!(first, second);
    fs::remove_dir_all(root).expect("remove HDR fingerprint test directory");
}

#[test]
fn runtime_shader_pbr_quantitative_evidence_identity_includes_bake_executable_and_input() {
    let identity = pbr_matrix_quantitative_evidence_identity("executable-a", "input-a");

    assert!(identity.contains(&format!("v{IBL_BAKE_ALGORITHM_VERSION}")));
    assert!(identity.ends_with("_eexecutable-a_iinput-a"));
}

#[test]
fn runtime_shader_pbr_real_hdri_2k_reflection_png_matches_plan06_metrics() {
    let output = runtime_shader_pbr_real_hdri_output_path(PBR_MATRIX_HDRI_2K_OUTPUT_NAME);

    assert_shader_test_output_path(&output);
    hdri_metrics::assert_saved_real_hdri_reflection_response(&output);
}

fn pbr_matrix_quantitative_evidence_names_for_provenance(
    executable_fingerprint: &str,
    runtime_input_fingerprint: &str,
) -> (String, String) {
    let evidence_identity = pbr_matrix_quantitative_evidence_identity(
        executable_fingerprint,
        runtime_input_fingerprint,
    );
    let stem = format!("runtime_shader_pbr_ibl_metallic_smoothness_matrix_{evidence_identity}");
    (format!("{stem}.png"), format!("{stem}.txt"))
}

fn pbr_matrix_quantitative_evidence_identity(
    executable_fingerprint: &str,
    runtime_input_fingerprint: &str,
) -> String {
    format!("v{IBL_BAKE_ALGORITHM_VERSION}_e{executable_fingerprint}_i{runtime_input_fingerprint}")
}

#[derive(Debug)]
struct PbrMatrixQuantitativeEvidenceProvenance {
    executable_fingerprint: String,
    runtime_input_fingerprint: String,
}

fn pbr_matrix_capture_quantitative_evidence_provenance(
) -> io::Result<PbrMatrixQuantitativeEvidenceProvenance> {
    Ok(PbrMatrixQuantitativeEvidenceProvenance {
        executable_fingerprint: pbr_matrix_fingerprint_file(&std::env::current_exe()?)?,
        runtime_input_fingerprint: pbr_matrix_quantitative_runtime_input_fingerprint()?,
    })
}

fn pbr_matrix_quantitative_runtime_input_fingerprint() -> io::Result<String> {
    pbr_matrix_fingerprint_file(&shader_test_asset_dir().join(POLYHAVEN_LAKES_2K_HDRI_ASSET))
}

fn pbr_matrix_fingerprint_file(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; PBR_MATRIX_EVIDENCE_FINGERPRINT_BUFFER_BYTES];
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn pbr_matrix_require_matching_runtime_input_fingerprint(
    expected: &str,
    actual: &str,
) -> io::Result<()> {
    if expected == actual {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "quantitative PBR HDR input changed while rendering evidence",
        ))
    }
}

fn pbr_matrix_verify_quantitative_evidence_runtime_input(expected: &str) -> io::Result<()> {
    let actual = pbr_matrix_quantitative_runtime_input_fingerprint()?;
    pbr_matrix_require_matching_runtime_input_fingerprint(expected, &actual)
}

#[test]
fn render_product_environment_pbr_matrix_quantitative() {
    run_render_product_environment_pbr_matrix_quantitative(false);
}

#[test]
#[ignore = "manual evidence export after the non-ignored Shader06 WGPU gates pass"]
fn export_product_environment_pbr_matrix_quantitative_evidence() {
    run_render_product_environment_pbr_matrix_quantitative(true);
}

fn run_render_product_environment_pbr_matrix_quantitative(write_evidence: bool) {
    std::thread::Builder::new()
        .name("runtime_shader_pbr_matrix_quantitative".to_string())
        .stack_size(128 * 1024 * 1024)
        .spawn(move || render_product_environment_pbr_matrix_quantitative_inner(write_evidence))
        .expect("spawn large-stack Shader 06 PBR matrix test")
        .join()
        .expect("Shader 06 PBR matrix test thread should not panic");
}

fn render_product_environment_pbr_matrix_quantitative_inner(write_evidence: bool) {
    let evidence_provenance = if write_evidence {
        Some(
            pbr_matrix_capture_quantitative_evidence_provenance()
                .expect("freeze quantitative PBR executable and HDR input before rendering"),
        )
    } else {
        None
    };
    let mut source_environment =
        polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_2K_HDRI_ASSET, 11);
    source_environment.irradiance_cube = None;
    let (frame, hdr, diffuse_hdr) = render_pbr_matrix_quantitative_frames(&source_environment);
    let frequency_environment = pbr_matrix_frequency_environment();
    let (_, frequency_hdr, frequency_diffuse_hdr) =
        render_pbr_matrix_quantitative_frames(&frequency_environment);
    let symmetric_environment = pbr_matrix_symmetric_environment();
    let (_, symmetric_hdr, symmetric_diffuse_hdr) =
        render_pbr_matrix_quantitative_frames(&symmetric_environment);
    let (grazing_symmetry_delta, grazing_max_per_radius_delta) =
        assert_dielectric_grazing_is_left_right_symmetric(&symmetric_hdr, &symmetric_diffuse_hdr);
    eprintln!(
        "plan06 grazing_left_right_mean_relative_delta={grazing_symmetry_delta:.6} max_per_radius_relative_delta={grazing_max_per_radius_delta:.6}"
    );
    let report = pbr_matrix_quantitative::assert_plan06_quantitative_gates(
        &frame,
        &hdr,
        &diffuse_hdr,
        &source_environment,
        &frequency_hdr,
        &frequency_diffuse_hdr,
        &frequency_environment,
        grazing_symmetry_delta,
        grazing_max_per_radius_delta,
    );
    if let Some(evidence_provenance) = evidence_provenance {
        let evidence_identity = pbr_matrix_quantitative_evidence_identity(
            &evidence_provenance.executable_fingerprint,
            &evidence_provenance.runtime_input_fingerprint,
        );
        let (output_name, report_name) = pbr_matrix_quantitative_evidence_names_for_provenance(
            &evidence_provenance.executable_fingerprint,
            &evidence_provenance.runtime_input_fingerprint,
        );
        let output = runtime_shader_pbr_real_hdri_output_path(&output_name);
        let report_output = runtime_shader_pbr_real_hdri_output_path(&report_name);
        let mut evidence = DatedEvidenceFiles::create(&output, &report_output)
            .unwrap_or_else(|error| panic!("claim immutable Shader 06 evidence paths: {error}"));
        write_viewport_frame_png_to_file(
            &frame,
            &mut evidence.screenshot,
            "Shader 06 quantitative PBR matrix screenshot",
        );
        writeln!(evidence.report, "evidence_identity={evidence_identity}")
            .expect("write Shader 06 quantitative evidence identity");
        writeln!(
            evidence.report,
            "executable_fingerprint={}",
            evidence_provenance.executable_fingerprint
        )
        .expect("write Shader 06 quantitative executable fingerprint");
        writeln!(
            evidence.report,
            "runtime_input_fingerprint={}",
            evidence_provenance.runtime_input_fingerprint
        )
        .expect("write Shader 06 quantitative HDR input fingerprint");
        evidence
            .report
            .write_all(report.to_text().as_bytes())
            .expect("write Shader 06 PBR matrix metric report");
        pbr_matrix_verify_quantitative_evidence_runtime_input(
            &evidence_provenance.runtime_input_fingerprint,
        )
        .expect("quantitative PBR HDR input must remain unchanged before evidence commit");
        evidence
            .commit()
            .expect("commit immutable Shader 06 evidence pair");
        assert_shader_test_output_path(&output);
        assert_shader_test_output_path(&report_output);
    }
}

#[derive(Debug)]
struct DatedEvidenceFiles {
    screenshot: fs::File,
    report: fs::File,
    screenshot_path: PathBuf,
    report_path: PathBuf,
    committed: bool,
}

impl DatedEvidenceFiles {
    fn create(screenshot_path: &Path, report_path: &Path) -> io::Result<Self> {
        let screenshot = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(screenshot_path)?;
        let report = match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(report_path)
        {
            Ok(report) => report,
            Err(error) => {
                drop(screenshot);
                fs::remove_file(screenshot_path)?;
                return Err(error);
            }
        };
        Ok(Self {
            screenshot,
            report,
            screenshot_path: screenshot_path.to_path_buf(),
            report_path: report_path.to_path_buf(),
            committed: false,
        })
    }

    fn commit(mut self) -> io::Result<()> {
        self.screenshot.flush()?;
        self.report.flush()?;
        self.screenshot.sync_all()?;
        self.report.sync_all()?;
        self.committed = true;
        Ok(())
    }
}

impl Drop for DatedEvidenceFiles {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        let _ = fs::remove_file(&self.screenshot_path);
        let _ = fs::remove_file(&self.report_path);
    }
}

fn write_viewport_frame_png_to_file(
    frame: &zircon_runtime::graphics::ViewportFrame,
    output: &mut fs::File,
    context: &str,
) {
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(frame.width, frame.height, frame.rgba.clone())
        .expect("rendered real HDRI PBR frame should match output image dimensions");
    image::DynamicImage::ImageRgba8(image)
        .write_to(output, ImageFormat::Png)
        .unwrap_or_else(|error| panic!("write {context}: {error}"));
}

#[test]
fn dated_quantitative_evidence_claim_is_exclusive() {
    let root = unique_temp_project_root("shader06_immutable_evidence");
    fs::create_dir_all(&root).expect("create immutable-evidence test directory");
    let output = root.join("evidence.png");
    let report = root.join("evidence.txt");

    let claim = DatedEvidenceFiles::create(&output, &report)
        .expect("first dated-evidence claim should be exclusive");
    let competing_error = DatedEvidenceFiles::create(&output, &report)
        .expect_err("a competing dated-evidence claim must fail");
    assert_eq!(competing_error.kind(), std::io::ErrorKind::AlreadyExists);
    drop(claim);
    assert!(
        !output.exists(),
        "abandoned screenshot claim must be removed"
    );
    assert!(!report.exists(), "abandoned report claim must be removed");

    fs::remove_dir_all(root).expect("remove immutable-evidence test directory");
}

#[test]
fn dated_quantitative_evidence_claim_rolls_back_partial_pair() {
    let root = unique_temp_project_root("shader06_immutable_evidence_rollback");
    fs::create_dir_all(&root).expect("create immutable-evidence rollback test directory");
    let output = root.join("evidence.png");
    let report = root.join("evidence.txt");
    fs::write(&report, b"existing").expect("write existing report fixture");

    let error = DatedEvidenceFiles::create(&output, &report)
        .expect_err("an existing report must reject the paired evidence claim");
    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert!(
        !output.exists(),
        "failed report claim must roll back the screenshot reservation"
    );

    fs::remove_dir_all(root).expect("remove immutable-evidence rollback test directory");
}

#[test]
#[ignore = "manual screenshot export for runtime PBR real HDRI reflection validation"]
fn export_runtime_shader_pbr_real_hdri_reflection_png() {
    std::thread::Builder::new()
        .name("runtime_shader_pbr_hdri_export".to_string())
        .stack_size(128 * 1024 * 1024)
        .spawn(export_runtime_shader_pbr_real_hdri_reflection_png_inner)
        .expect("spawn large-stack HDRI export test")
        .join()
        .expect("HDRI export test thread should not panic");
}

#[test]
#[ignore = "manual 2K screenshot export for runtime PBR real HDRI reflection validation"]
fn export_runtime_shader_pbr_real_hdri_2k_reflection_png() {
    std::thread::Builder::new()
        .name("runtime_shader_pbr_hdri_2k_export".to_string())
        .stack_size(128 * 1024 * 1024)
        .spawn(export_runtime_shader_pbr_real_hdri_2k_reflection_png_inner)
        .expect("spawn large-stack 2K HDRI export test")
        .join()
        .expect("2K HDRI export test thread should not panic");
}

fn pbr_matrix_frequency_environment() -> SourceCubemapEnvironment {
    let mip_chain = build_source_cubemap_from_equirect(64, |u, v| {
        let mut signal = 3.5;
        for (frequency, amplitude, phase) in [
            (1.0, 1.20, 0.00),
            (2.0, 0.95, 0.37),
            (4.0, 0.72, 0.91),
            (8.0, 0.52, 1.43),
            (16.0, 0.36, 2.07),
            (32.0, 0.24, 2.71),
        ] {
            let longitude = std::f32::consts::TAU * frequency * u + phase;
            let latitude = std::f32::consts::PI * frequency * (v - 0.5) - phase * 0.5;
            signal += amplitude * longitude.sin() * latitude.cos();
        }
        // Keep the terminal PDF-LOD step measurable without isolated impulses.
        let radiance = (signal * 0.65).exp();
        [radiance, radiance * 0.78, radiance * 0.52, 1.0]
    });
    SourceCubemapEnvironment::new(
        mip_chain,
        0x7062_7266_6672_6571,
        [0x7062_722d, 0x6672_6571, 0x7565_6e63, 0x7900_0001],
    )
}

fn pbr_matrix_symmetric_environment() -> SourceCubemapEnvironment {
    let mip_chain = build_source_cubemap_from_equirect(64, |_u, _v| [1.0, 1.0, 1.0, 1.0]);
    SourceCubemapEnvironment::new(
        mip_chain,
        0x7062_7273_796d_6d65,
        [0x7062_722d, 0x7379_6d6d, 0x6574_7279, 0x0000_0001],
    )
}

fn assert_dielectric_grazing_is_left_right_symmetric(
    hdr: &[[f32; 4]],
    diffuse_hdr: &[[f32; 4]],
) -> (f32, f32) {
    let width = PBR_MATRIX_OUTPUT_SIZE.x;
    let height = PBR_MATRIX_OUTPUT_SIZE.y;
    assert_eq!(hdr.len(), width as usize * height as usize);
    assert_eq!(diffuse_hdr.len(), hdr.len());

    let center_x = pbr_matrix_world_x(0);
    let center_y = pbr_matrix_world_y(PBR_MATRIX_DIMENSION - 1);
    let center_y_pixel = pbr_matrix_pixel_y(center_y);
    let mut max_relative_delta = 0.0_f32;
    let mut left_sum = 0.0_f32;
    let mut right_sum = 0.0_f32;

    for radius in GRAZING_SYMMETRY_RADII {
        let world_offset = PBR_MATRIX_SPHERE_SCALE * radius;
        let left_x = pbr_matrix_pixel_x(center_x - world_offset);
        let right_x = pbr_matrix_pixel_x(center_x + world_offset);
        let left = pbr_specular_patch_luma(hdr, diffuse_hdr, width, height, left_x, center_y_pixel);
        let right =
            pbr_specular_patch_luma(hdr, diffuse_hdr, width, height, right_x, center_y_pixel);
        let pair_mean = ((left + right) * 0.5).max(1.0e-4);
        let relative_delta = (left - right).abs() / pair_mean;
        max_relative_delta = max_relative_delta.max(relative_delta);
        left_sum += left;
        right_sum += right;
    }

    let left_mean = left_sum / GRAZING_SYMMETRY_RADII.len() as f32;
    let right_mean = right_sum / GRAZING_SYMMETRY_RADII.len() as f32;
    let mean_relative_delta =
        (left_mean - right_mean).abs() / ((left_mean + right_mean) * 0.5).max(1.0e-4);
    assert!(
        left_mean.min(right_mean) > 0.02,
        "symmetric-environment grazing samples must contain a visible specular response, left={left_mean}, right={right_mean}"
    );
    assert!(
        mean_relative_delta <= GRAZING_SYMMETRY_MAX_MEAN_RELATIVE_DELTA,
        "smooth dielectric grazing response must have symmetric left/right aggregate energy in a constant environment, mean_relative_delta={mean_relative_delta}, threshold={GRAZING_SYMMETRY_MAX_MEAN_RELATIVE_DELTA}, max_per_radius_delta={max_relative_delta}, left_mean={left_mean}, right_mean={right_mean}"
    );
    assert!(
        max_relative_delta <= GRAZING_SYMMETRY_MAX_PER_RADIUS_RELATIVE_DELTA,
        "smooth dielectric grazing response must remain locally symmetric at every sampled radius in a constant environment, max_per_radius_delta={max_relative_delta}, threshold={GRAZING_SYMMETRY_MAX_PER_RADIUS_RELATIVE_DELTA}, mean_relative_delta={mean_relative_delta}, left_mean={left_mean}, right_mean={right_mean}"
    );
    (mean_relative_delta, max_relative_delta)
}

fn pbr_specular_patch_luma(
    hdr: &[[f32; 4]],
    diffuse_hdr: &[[f32; 4]],
    width: u32,
    height: u32,
    center_x: u32,
    center_y: u32,
) -> f32 {
    let mut sum = 0.0;
    let mut sample_count = 0_u32;
    for y_offset in -1_i32..=1 {
        let y = center_y.saturating_add_signed(y_offset).min(height - 1);
        for x_offset in -1_i32..=1 {
            let x = center_x.saturating_add_signed(x_offset).min(width - 1);
            sum += pbr_specular_luma(hdr, diffuse_hdr, width, x, y);
            sample_count += 1;
        }
    }
    sum / sample_count as f32
}

fn pbr_matrix_pixel_x(world_x: f32) -> u32 {
    let width = PBR_MATRIX_OUTPUT_SIZE.x as f32;
    let half_width = PBR_MATRIX_ORTHO_SIZE * width / PBR_MATRIX_OUTPUT_SIZE.y as f32;
    ((((world_x + half_width) / (2.0 * half_width)) * width) - 0.5)
        .round()
        .clamp(0.0, width - 1.0) as u32
}

fn pbr_matrix_pixel_y(world_y: f32) -> u32 {
    let height = PBR_MATRIX_OUTPUT_SIZE.y as f32;
    ((((PBR_MATRIX_ORTHO_SIZE - world_y) / (2.0 * PBR_MATRIX_ORTHO_SIZE)) * height) - 0.5)
        .round()
        .clamp(0.0, height - 1.0) as u32
}

fn pbr_specular_luma(
    hdr: &[[f32; 4]],
    diffuse_hdr: &[[f32; 4]],
    width: u32,
    x: u32,
    y: u32,
) -> f32 {
    let index = y as usize * width as usize + x as usize;
    luma([
        (hdr[index][0] - diffuse_hdr[index][0]).max(0.0),
        (hdr[index][1] - diffuse_hdr[index][1]).max(0.0),
        (hdr[index][2] - diffuse_hdr[index][2]).max(0.0),
    ])
}

fn export_runtime_shader_pbr_real_hdri_reflection_png_inner() {
    export_runtime_shader_pbr_real_hdri_reflection_png_with_asset(
        POLYHAVEN_LAKES_1K_HDRI_ASSET,
        PBR_MATRIX_HDRI_OUTPUT_NAME,
        1,
    );
}

fn export_runtime_shader_pbr_real_hdri_2k_reflection_png_inner() {
    export_runtime_shader_pbr_real_hdri_reflection_png_with_asset(
        POLYHAVEN_LAKES_2K_HDRI_ASSET,
        PBR_MATRIX_HDRI_2K_OUTPUT_NAME,
        2,
    );
}

fn export_runtime_shader_pbr_real_hdri_reflection_png_with_asset(
    asset_name: &str,
    output_name: &str,
    source_revision: u64,
) {
    let frame = render_pbr_matrix_frame_with_environment(EnvironmentExtract::source_cubemap(
        polyhaven_lakes_source_cubemap_environment(asset_name, source_revision),
    ));
    let output = runtime_shader_pbr_real_hdri_output_path(output_name);

    save_viewport_frame_png(&frame, &output, "real HDRI PBR reflection screenshot");
    assert_shader_test_output_path(&output);
    hdri_metrics::assert_real_hdri_reflection_response(&frame);
}

fn render_pbr_matrix_frame_with_environment(environment: EnvironmentExtract) -> ViewportFrame {
    render_project_frame_with_environment(
        "graphics_pbr_real_hdri_integration",
        "GraphicsPbrRealHdriIntegration",
        "res://scenes/pbr_matrix.scene.toml",
        PBR_MATRIX_OUTPUT_SIZE,
        environment,
        |paths| write_pbr_matrix_assets(paths, 24, 48),
    )
}

fn render_pbr_matrix_quantitative_frames(
    source_environment: &SourceCubemapEnvironment,
) -> (ViewportFrame, Vec<[f32; 4]>, Vec<[f32; 4]>) {
    render_project_with_environment(
        "graphics_pbr_matrix_quantitative",
        "GraphicsPbrMatrixQuantitative",
        "res://scenes/pbr_matrix.scene.toml",
        PBR_MATRIX_OUTPUT_SIZE,
        EnvironmentExtract::source_cubemap(source_environment.clone()),
        |paths| write_pbr_matrix_assets(paths, 48, 96),
        |framework, viewport, snapshot| {
            framework
                .submit_frame_extract(
                    viewport,
                    RenderFrameExtract::from_snapshot(
                        RenderWorldSnapshotHandle::new(0),
                        snapshot.clone(),
                    ),
                )
                .expect("submit Shader 06 linear HDR PBR matrix");
            let hdr = framework
                .capture_scene_color_hdr(viewport)
                .expect("capture Shader 06 linear HDR PBR matrix")
                .expect("compiled Shader 06 HDR PBR matrix should be available")
                .rgba16f;
            let mut diffuse_snapshot = snapshot.clone();
            diffuse_snapshot.environment =
                EnvironmentExtract::source_cubemap(diffuse_only_environment(source_environment));
            diffuse_snapshot.preview = PreviewEnvironmentExtract::from_environment(
                &diffuse_snapshot.environment,
                true,
                Vec4::ZERO,
            );
            framework
                .submit_frame_extract(
                    viewport,
                    RenderFrameExtract::from_snapshot(
                        RenderWorldSnapshotHandle::new(0),
                        diffuse_snapshot,
                    ),
                )
                .expect("submit Shader 06 diffuse-only HDR PBR matrix baseline");
            let diffuse_hdr = framework
                .capture_scene_color_hdr(viewport)
                .expect("capture Shader 06 diffuse-only HDR PBR matrix baseline")
                .expect("compiled diffuse-only HDR PBR matrix should be available")
                .rgba16f;
            let frame = submit_frame_extract_and_capture(framework, viewport, snapshot);
            (frame, hdr, diffuse_hdr)
        },
    )
}

fn write_pbr_matrix_assets(paths: &ProjectPaths, rings: usize, segments: usize) {
    let asset_root =
        paths.asset_root(&zircon_runtime_interface::project::RelPath::project_assets());
    write_uv_sphere_model(
        asset_root
            .join("models")
            .join("pbr_matrix_sphere.model.toml"),
        "res://models/pbr_matrix_sphere.model.toml",
        rings,
        segments,
    );
    for row in 0..PBR_MATRIX_DIMENSION {
        for column in 0..PBR_MATRIX_DIMENSION {
            write_pbr_matrix_material(
                asset_root
                    .join("materials")
                    .join(format!("pbr_matrix_r{row}_c{column}.zmaterial")),
                pbr_matrix_axis_value(column),
                pbr_matrix_axis_value(row),
            );
        }
    }
    write_pbr_matrix_scene(asset_root.join("scenes").join("pbr_matrix.scene.toml"));
}

fn diffuse_only_environment(
    source_environment: &SourceCubemapEnvironment,
) -> SourceCubemapEnvironment {
    let mip_chain = SourceCubemapMipChain::new(
        source_environment.mip_chain.source_face_size(),
        source_environment.mip_chain.source_mip_count(),
        vec![[0.0; 4]; source_environment.mip_chain.source_texels().len()],
        source_environment.mip_chain.pmrem_face_size(),
        source_environment.mip_chain.pmrem_mip_count(),
        vec![[0.0; 4]; source_environment.mip_chain.pmrem_texels().len()],
    );
    let mut environment = SourceCubemapEnvironment::new(
        mip_chain,
        source_environment.source_revision.saturating_add(1),
        [0x6469_6666, 0x7573_652d, 0x6f6e_6c79, 0x0000_0001],
    );
    environment.irradiance_sh9 = source_environment.irradiance_sh9;
    environment.intensity = source_environment.intensity;
    environment.rotation_radians = source_environment.rotation_radians;
    environment
}
