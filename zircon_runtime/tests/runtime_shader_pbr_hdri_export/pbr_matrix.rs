use super::*;

const PBR_MATRIX_HDRI_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_8x8_cmft_pmrem_reflection_20260710.png";
const PBR_MATRIX_HDRI_2K_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_2k_8x8_cmft_pmrem_reflection_20260710.png";
const PBR_MATRIX_QUANTITATIVE_OUTPUT_NAME: &str =
    "runtime_shader_pbr_ibl_metallic_smoothness_matrix_angular0003_20260713.png";
const PBR_MATRIX_QUANTITATIVE_REPORT_NAME: &str =
    "runtime_shader_pbr_ibl_metallic_smoothness_matrix_angular0003_20260713.txt";

#[test]
fn runtime_shader_pbr_matrix_contract_uses_requested_eight_by_eight_grid() {
    assert_eq!(PBR_MATRIX_DIMENSION, 8);
    assert_eq!(PBR_MATRIX_DIMENSION * PBR_MATRIX_DIMENSION, 64);
    assert_eq!(pbr_matrix_axis_value(0), 0.0);
    assert_eq!(pbr_matrix_axis_value(PBR_MATRIX_DIMENSION - 1), 1.0);
}

#[test]
fn runtime_shader_pbr_real_hdri_2k_reflection_png_matches_plan06_metrics() {
    let output = runtime_shader_pbr_real_hdri_output_path(PBR_MATRIX_HDRI_2K_OUTPUT_NAME);

    assert_shader_test_output_path(&output);
    hdri_metrics::assert_saved_real_hdri_reflection_response(&output);
}

#[test]
#[ignore = "manual WGPU product acceptance for Shader 06 PBR matrix quantitative gates"]
fn render_product_environment_pbr_matrix_quantitative() {
    let mut source_environment =
        polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_2K_HDRI_ASSET, 11);
    source_environment.irradiance_cube = None;
    let (frame, hdr, diffuse_hdr) = render_pbr_matrix_quantitative_frames(&source_environment);
    let frequency_environment = pbr_matrix_frequency_environment();
    let (_, frequency_hdr, frequency_diffuse_hdr) =
        render_pbr_matrix_quantitative_frames(&frequency_environment);
    let report = pbr_matrix_quantitative::assert_plan06_quantitative_gates(
        &frame,
        &hdr,
        &diffuse_hdr,
        &source_environment,
        &frequency_hdr,
        &frequency_diffuse_hdr,
        &frequency_environment,
    );
    let output = runtime_shader_pbr_real_hdri_output_path(PBR_MATRIX_QUANTITATIVE_OUTPUT_NAME);
    let report_output =
        runtime_shader_pbr_real_hdri_output_path(PBR_MATRIX_QUANTITATIVE_REPORT_NAME);

    save_viewport_frame_png(
        &frame,
        &output,
        "Shader 06 quantitative PBR matrix screenshot",
    );
    fs::write(&report_output, report.to_text()).expect("write Shader 06 PBR matrix metric report");
    assert_shader_test_output_path(&output);
    assert_shader_test_output_path(&report_output);
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
        |renderer, snapshot| {
            let hdr = renderer
                .render_scene_color_hdr(snapshot.clone(), PBR_MATRIX_OUTPUT_SIZE)
                .expect("render Shader 06 linear HDR PBR matrix");
            let mut diffuse_snapshot = snapshot.clone();
            diffuse_snapshot.environment =
                EnvironmentExtract::source_cubemap(diffuse_only_environment(source_environment));
            diffuse_snapshot.preview = PreviewEnvironmentExtract::from_environment(
                &diffuse_snapshot.environment,
                true,
                Vec4::ZERO,
            );
            let diffuse_hdr = renderer
                .render_scene_color_hdr(diffuse_snapshot, PBR_MATRIX_OUTPUT_SIZE)
                .expect("render Shader 06 diffuse-only HDR PBR matrix baseline");
            let frame = renderer
                .render(snapshot, PBR_MATRIX_OUTPUT_SIZE)
                .expect("render Shader 06 display PBR matrix");
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
