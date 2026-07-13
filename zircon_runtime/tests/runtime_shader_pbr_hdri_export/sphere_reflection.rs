use super::*;

const PBR_SINGLE_HDRI_OUTPUT_SIZE: UVec2 = UVec2::new(1280, 960);
const PBR_SINGLE_SPHERE_RINGS: usize = 96;
const PBR_SINGLE_SPHERE_SEGMENTS: usize = 192;
const PBR_SINGLE_HDRI_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_single_metal_sphere_reflection_20260706.png";
const PBR_MIRROR_ORTHOGRAPHIC_HDRI_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_mirror_sphere_orthographic_reflection_20260707.png";
const PBR_MIRROR_PERSPECTIVE_HDRI_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_mirror_sphere_perspective_reflection_20260707.png";
const PBR_MIRROR_MULTI_VIEW_HDRI_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_mirror_sphere_multi_view_reflection_20260707.png";
const PBR_MIRROR_CARDINAL_120DEG_HDRI_OUTPUT_NAME: &str =
    "runtime_shader_pbr_real_hdri_lakes_mirror_sphere_cardinal_120deg_reflection_20260708.png";
const PBR_MIRROR_MULTI_VIEW_TILE_SIZE: UVec2 = UVec2::new(800, 600);
const PBR_MIRROR_MULTI_VIEW_COLUMNS: u32 = 2;

#[derive(Clone, Copy)]
struct MirrorMultiViewCase {
    label: &'static str,
    project_name: &'static str,
    camera_view: SinglePbrSphereCameraView,
}

fn mirror_multi_view_cases() -> [MirrorMultiViewCase; 4] {
    [
        MirrorMultiViewCase {
            label: "multi-view orthographic front perfect mirror PBR sphere",
            project_name: "GraphicsPbrRealHdriMirrorMultiViewOrthoFront",
            camera_view: SinglePbrSphereCameraView::front(ProjectionMode::Orthographic),
        },
        MirrorMultiViewCase {
            label: "multi-view perspective front perfect mirror PBR sphere",
            project_name: "GraphicsPbrRealHdriMirrorMultiViewPerspectiveFront",
            camera_view: SinglePbrSphereCameraView::front(ProjectionMode::Perspective),
        },
        MirrorMultiViewCase {
            label: "multi-view perspective left-yaw perfect mirror PBR sphere",
            project_name: "GraphicsPbrRealHdriMirrorMultiViewPerspectiveLeftYaw",
            camera_view: SinglePbrSphereCameraView::perspective_eye([-2.25, 0.0, 3.65]),
        },
        MirrorMultiViewCase {
            label: "multi-view perspective right-yaw perfect mirror PBR sphere",
            project_name: "GraphicsPbrRealHdriMirrorMultiViewPerspectiveRightYaw",
            camera_view: SinglePbrSphereCameraView::perspective_eye([2.25, 0.0, 3.65]),
        },
    ]
}

fn mirror_cardinal_120deg_view_cases() -> [MirrorMultiViewCase; 4] {
    [
        MirrorMultiViewCase {
            label: "120-degree perspective up-orbit perfect mirror PBR sphere",
            project_name: "GraphicsPbrRealHdriMirrorCardinal120Up",
            camera_view: SinglePbrSphereCameraView::perspective_orbit_degrees(0.0, 120.0),
        },
        MirrorMultiViewCase {
            label: "120-degree perspective down-orbit perfect mirror PBR sphere",
            project_name: "GraphicsPbrRealHdriMirrorCardinal120Down",
            camera_view: SinglePbrSphereCameraView::perspective_orbit_degrees(0.0, -120.0),
        },
        MirrorMultiViewCase {
            label: "120-degree perspective left-yaw perfect mirror PBR sphere",
            project_name: "GraphicsPbrRealHdriMirrorCardinal120Left",
            camera_view: SinglePbrSphereCameraView::perspective_orbit_degrees(-120.0, 0.0),
        },
        MirrorMultiViewCase {
            label: "120-degree perspective right-yaw perfect mirror PBR sphere",
            project_name: "GraphicsPbrRealHdriMirrorCardinal120Right",
            camera_view: SinglePbrSphereCameraView::perspective_orbit_degrees(120.0, 0.0),
        },
    ]
}

fn mirror_multi_view_row_count() -> u32 {
    contact_sheet_row_count(mirror_multi_view_cases().len())
}

fn mirror_cardinal_120deg_row_count() -> u32 {
    contact_sheet_row_count(mirror_cardinal_120deg_view_cases().len())
}

fn contact_sheet_row_count(case_count: usize) -> u32 {
    (case_count as u32 + PBR_MIRROR_MULTI_VIEW_COLUMNS - 1) / PBR_MIRROR_MULTI_VIEW_COLUMNS
}

#[test]
fn runtime_shader_pbr_real_hdri_mirror_reflection_png_matches_orientation_and_grazing_metrics() {
    for (output_name, label) in mirror_output_cases() {
        let output = runtime_shader_pbr_real_hdri_output_path(output_name);
        assert_shader_test_output_path(&output);
        let frame = load_saved_viewport_frame(&output);
        assert_eq!(
            (frame.width, frame.height),
            (PBR_SINGLE_HDRI_OUTPUT_SIZE.x, PBR_SINGLE_HDRI_OUTPUT_SIZE.y),
            "{label} screenshot should keep the accepted mirror validation dimensions"
        );
        assert_single_sphere_reflects_environment(&frame, label);
        assert_mirror_sphere_reflection_orientation(&frame);
    }
}

#[test]
fn runtime_shader_pbr_real_hdri_mirror_reflection_png_matches_source_reference_metrics() {
    let mut source_environment =
        polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_2K_HDRI_ASSET, 4);
    source_environment.intensity = 0.65;

    for (output_name, label) in mirror_output_cases() {
        let projection_mode = if output_name == PBR_MIRROR_ORTHOGRAPHIC_HDRI_OUTPUT_NAME {
            ProjectionMode::Orthographic
        } else {
            ProjectionMode::Perspective
        };
        let output = runtime_shader_pbr_real_hdri_output_path(output_name);
        assert_shader_test_output_path(&output);
        let frame = load_saved_viewport_frame(&output);
        assert_mirror_sphere_matches_source_reference(
            &frame,
            projection_mode,
            &source_environment,
            label,
        );
    }
}

fn mirror_output_cases() -> [(&'static str, &'static str); 2] {
    [
        (
            PBR_MIRROR_ORTHOGRAPHIC_HDRI_OUTPUT_NAME,
            "saved orthographic perfect mirror PBR sphere",
        ),
        (
            PBR_MIRROR_PERSPECTIVE_HDRI_OUTPUT_NAME,
            "saved perspective perfect mirror PBR sphere",
        ),
    ]
}

#[test]
fn runtime_shader_pbr_real_hdri_mirror_multi_view_png_matches_orientation_metrics() {
    assert_saved_contact_sheet(
        PBR_MIRROR_MULTI_VIEW_HDRI_OUTPUT_NAME,
        mirror_multi_view_row_count(),
        mirror_multi_view_cases(),
        |tile, view_case, _| {
            assert_single_sphere_reflects_environment(tile, view_case.label);
            assert_mirror_sphere_reflection_orientation(tile);
        },
    );
}

#[test]
fn runtime_shader_pbr_real_hdri_mirror_multi_view_png_matches_source_reference_metrics() {
    let mut source_environment =
        polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_2K_HDRI_ASSET, 11);
    source_environment.intensity = 0.65;
    assert_saved_contact_sheet(
        PBR_MIRROR_MULTI_VIEW_HDRI_OUTPUT_NAME,
        mirror_multi_view_row_count(),
        mirror_multi_view_cases(),
        |tile, view_case, _| {
            assert_mirror_sphere_matches_source_reference_with_camera_view(
                tile,
                view_case.camera_view,
                &source_environment,
                view_case.label,
            );
        },
    );
}

#[test]
fn runtime_shader_pbr_real_hdri_mirror_cardinal_120deg_png_matches_environment_metrics() {
    assert_saved_contact_sheet(
        PBR_MIRROR_CARDINAL_120DEG_HDRI_OUTPUT_NAME,
        mirror_cardinal_120deg_row_count(),
        mirror_cardinal_120deg_view_cases(),
        |tile, view_case, _| assert_single_sphere_reflects_environment(tile, view_case.label),
    );
}

#[test]
fn runtime_shader_pbr_real_hdri_mirror_cardinal_120deg_png_matches_source_reference_metrics() {
    let mut source_environment =
        polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_2K_HDRI_ASSET, 12);
    source_environment.intensity = 0.65;
    assert_saved_contact_sheet(
        PBR_MIRROR_CARDINAL_120DEG_HDRI_OUTPUT_NAME,
        mirror_cardinal_120deg_row_count(),
        mirror_cardinal_120deg_view_cases(),
        |tile, view_case, _| {
            assert_mirror_sphere_matches_source_reference_with_camera_view(
                tile,
                view_case.camera_view,
                &source_environment,
                view_case.label,
            );
        },
    );
}

fn assert_saved_contact_sheet(
    output_name: &str,
    rows: u32,
    cases: [MirrorMultiViewCase; 4],
    mut assert_tile: impl FnMut(&ViewportFrame, MirrorMultiViewCase, usize),
) {
    let output = runtime_shader_pbr_real_hdri_output_path(output_name);
    assert_shader_test_output_path(&output);
    let sheet = load_saved_viewport_frame(&output);
    assert_eq!(
        (sheet.width, sheet.height),
        (
            PBR_MIRROR_MULTI_VIEW_TILE_SIZE.x * PBR_MIRROR_MULTI_VIEW_COLUMNS,
            PBR_MIRROR_MULTI_VIEW_TILE_SIZE.y * rows,
        ),
        "mirror contact sheet should keep the accepted dimensions"
    );
    for (index, view_case) in cases.into_iter().enumerate() {
        let tile = viewport_frame_tile(
            &sheet,
            index as u32 % PBR_MIRROR_MULTI_VIEW_COLUMNS,
            index as u32 / PBR_MIRROR_MULTI_VIEW_COLUMNS,
            PBR_MIRROR_MULTI_VIEW_TILE_SIZE,
        );
        assert_tile(&tile, view_case, index);
    }
}

#[test]
#[ignore = "manual single material sphere export for runtime PBR real HDRI reflection validation"]
fn export_runtime_shader_pbr_real_hdri_single_reflection_png() {
    run_large_stack_export(
        "runtime_shader_pbr_hdri_single_export",
        export_runtime_shader_pbr_real_hdri_single_reflection_png_inner,
    );
}

#[test]
#[ignore = "manual mirror material sphere export for runtime PBR real HDRI reflection orientation validation"]
fn export_runtime_shader_pbr_real_hdri_mirror_reflection_png() {
    run_large_stack_export(
        "runtime_shader_pbr_hdri_mirror_export",
        export_runtime_shader_pbr_real_hdri_mirror_reflection_png_inner,
    );
}

#[test]
#[ignore = "manual multi-view mirror material sphere export for runtime PBR real HDRI reflection validation"]
fn export_runtime_shader_pbr_real_hdri_mirror_multi_view_png() {
    run_large_stack_export(
        "runtime_shader_pbr_hdri_mirror_multi_view_export",
        export_runtime_shader_pbr_real_hdri_mirror_multi_view_png_inner,
    );
}

#[test]
#[ignore = "manual 120-degree cardinal mirror material sphere export for runtime PBR real HDRI reflection validation"]
fn export_runtime_shader_pbr_real_hdri_mirror_cardinal_120deg_png() {
    run_large_stack_export(
        "runtime_shader_pbr_hdri_mirror_120deg_export",
        export_runtime_shader_pbr_real_hdri_mirror_cardinal_120deg_png_inner,
    );
}

fn run_large_stack_export(name: &str, export: fn()) {
    std::thread::Builder::new()
        .name(name.to_string())
        .stack_size(128 * 1024 * 1024)
        .spawn(export)
        .expect("spawn large-stack sphere HDRI export test")
        .join()
        .expect("sphere HDRI export test thread should not panic");
}

fn export_runtime_shader_pbr_real_hdri_single_reflection_png_inner() {
    let environment = EnvironmentExtract::source_cubemap(
        polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_2K_HDRI_ASSET, 2),
    );
    let frame = render_single_pbr_sphere_frame_with_environment(
        environment,
        "GraphicsPbrRealHdriSingleReflection",
        |paths| {
            write_single_pbr_material(
                single_sphere_material_path(paths),
                "Single Metal Sphere",
                [0.86, 0.88, 0.9, 1.0],
                1.0,
                0.04,
                None,
                None,
                None,
            );
        },
    );
    let output = runtime_shader_pbr_real_hdri_output_path(PBR_SINGLE_HDRI_OUTPUT_NAME);
    save_viewport_frame_png(
        &frame,
        &output,
        "single real HDRI PBR reflection screenshot",
    );
    assert_shader_test_output_path(&output);
    assert_single_sphere_reflects_environment(&frame, "single metal PBR sphere");
}

fn export_runtime_shader_pbr_real_hdri_mirror_reflection_png_inner() {
    for (projection_mode, project_name, output_name, label) in [
        (
            ProjectionMode::Orthographic,
            "GraphicsPbrRealHdriMirrorOrthographicReflection",
            PBR_MIRROR_ORTHOGRAPHIC_HDRI_OUTPUT_NAME,
            "orthographic perfect mirror PBR sphere",
        ),
        (
            ProjectionMode::Perspective,
            "GraphicsPbrRealHdriMirrorPerspectiveReflection",
            PBR_MIRROR_PERSPECTIVE_HDRI_OUTPUT_NAME,
            "perspective perfect mirror PBR sphere",
        ),
    ] {
        export_mirror_projection(projection_mode, project_name, output_name, label);
    }
}

fn export_mirror_projection(
    projection_mode: ProjectionMode,
    project_name: &str,
    output_name: &str,
    assertion_label: &str,
) {
    let mut source_environment =
        polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_2K_HDRI_ASSET, 4);
    source_environment.intensity = 0.65;
    let frame = render_single_pbr_sphere_frame_with_environment_and_projection(
        EnvironmentExtract::source_cubemap(source_environment),
        project_name,
        projection_mode,
        write_perfect_mirror_material,
    );
    let output = runtime_shader_pbr_real_hdri_output_path(output_name);
    save_viewport_frame_png(
        &frame,
        &output,
        "mirror real HDRI PBR reflection screenshot",
    );
    assert_shader_test_output_path(&output);
    assert_single_sphere_reflects_environment(&frame, assertion_label);
    assert_mirror_sphere_reflection_orientation(&frame);
}

fn export_runtime_shader_pbr_real_hdri_mirror_multi_view_png_inner() {
    export_mirror_contact_sheet(
        mirror_multi_view_cases(),
        11,
        PBR_MIRROR_MULTI_VIEW_HDRI_OUTPUT_NAME,
        "multi-view mirror real HDRI PBR reflection screenshot",
        false,
    );
}

fn export_runtime_shader_pbr_real_hdri_mirror_cardinal_120deg_png_inner() {
    export_mirror_contact_sheet(
        mirror_cardinal_120deg_view_cases(),
        12,
        PBR_MIRROR_CARDINAL_120DEG_HDRI_OUTPUT_NAME,
        "120-degree mirror real HDRI PBR reflection screenshot",
        true,
    );
}

fn export_mirror_contact_sheet(
    cases: [MirrorMultiViewCase; 4],
    source_revision: u64,
    output_name: &str,
    context: &str,
    compare_source: bool,
) {
    let mut source_environment =
        polyhaven_lakes_source_cubemap_environment(POLYHAVEN_LAKES_2K_HDRI_ASSET, source_revision);
    source_environment.intensity = 0.65;
    let mut frames = Vec::new();
    for view_case in cases {
        let frame = render_single_pbr_sphere_frame_with_environment_and_camera_view(
            EnvironmentExtract::source_cubemap(source_environment.clone()),
            view_case.project_name,
            view_case.camera_view,
            PBR_MIRROR_MULTI_VIEW_TILE_SIZE,
            write_perfect_mirror_material,
        );
        assert_single_sphere_reflects_environment(&frame, view_case.label);
        if compare_source {
            assert_mirror_sphere_matches_source_reference_with_camera_view(
                &frame,
                view_case.camera_view,
                &source_environment,
                view_case.label,
            );
        } else {
            assert_mirror_sphere_reflection_orientation(&frame);
        }
        frames.push(frame);
    }
    let output = runtime_shader_pbr_real_hdri_output_path(output_name);
    save_viewport_frame_contact_sheet_png(&frames, PBR_MIRROR_MULTI_VIEW_COLUMNS, &output, context);
    assert_shader_test_output_path(&output);
}

fn write_perfect_mirror_material(paths: &ProjectPaths) {
    write_single_pbr_material(
        single_sphere_material_path(paths),
        "Perfect Mirror Sphere",
        [1.0, 1.0, 1.0, 1.0],
        1.0,
        0.0,
        None,
        None,
        None,
    );
}

fn single_sphere_material_path(paths: &ProjectPaths) -> PathBuf {
    paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("materials")
        .join("single_metal_sphere.zmaterial")
}

pub(super) fn render_single_pbr_sphere_frame_with_environment(
    environment: EnvironmentExtract,
    project_name: &str,
    write_material_assets: impl FnOnce(&ProjectPaths),
) -> ViewportFrame {
    render_single_pbr_sphere_frame_with_environment_and_projection(
        environment,
        project_name,
        ProjectionMode::Orthographic,
        write_material_assets,
    )
}

fn render_single_pbr_sphere_frame_with_environment_and_projection(
    environment: EnvironmentExtract,
    project_name: &str,
    projection_mode: ProjectionMode,
    write_material_assets: impl FnOnce(&ProjectPaths),
) -> ViewportFrame {
    render_single_pbr_sphere_frame_with_environment_and_camera_view(
        environment,
        project_name,
        SinglePbrSphereCameraView::front(projection_mode),
        PBR_SINGLE_HDRI_OUTPUT_SIZE,
        write_material_assets,
    )
}

fn render_single_pbr_sphere_frame_with_environment_and_camera_view(
    environment: EnvironmentExtract,
    project_name: &str,
    camera_view: SinglePbrSphereCameraView,
    output_size: UVec2,
    write_material_assets: impl FnOnce(&ProjectPaths),
) -> ViewportFrame {
    render_project_frame_with_environment(
        "graphics_pbr_single_real_hdri_integration",
        project_name,
        "res://scenes/single_pbr_sphere.scene.toml",
        output_size,
        environment,
        |paths| {
            let asset_root =
                paths.asset_root(&zircon_runtime_interface::project::RelPath::project_assets());
            write_uv_sphere_model(
                asset_root
                    .join("models")
                    .join("single_pbr_sphere.model.toml"),
                "res://models/single_pbr_sphere.model.toml",
                PBR_SINGLE_SPHERE_RINGS,
                PBR_SINGLE_SPHERE_SEGMENTS,
            );
            write_material_assets(paths);
            write_single_pbr_sphere_scene_with_camera_view(
                asset_root
                    .join("scenes")
                    .join("single_pbr_sphere.scene.toml"),
                camera_view,
            );
        },
    )
}

fn save_viewport_frame_contact_sheet_png(
    frames: &[ViewportFrame],
    columns: u32,
    output: &Path,
    context: &str,
) {
    assert!(
        !frames.is_empty(),
        "{context} should contain at least one frame"
    );
    assert!(columns > 0, "{context} should use at least one column");
    let tile_width = frames[0].width;
    let tile_height = frames[0].height;
    let rows = (frames.len() as u32 + columns - 1) / columns;
    let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(tile_width * columns, tile_height * rows);
    for (index, frame) in frames.iter().enumerate() {
        assert_eq!((frame.width, frame.height), (tile_width, tile_height));
        let tile_column = index as u32 % columns;
        let tile_row = index as u32 / columns;
        for y in 0..tile_height {
            for x in 0..tile_width {
                let source_index = ((y * tile_width + x) * 4) as usize;
                image.put_pixel(
                    tile_column * tile_width + x,
                    tile_row * tile_height + y,
                    Rgba(
                        frame.rgba[source_index..source_index + 4]
                            .try_into()
                            .unwrap(),
                    ),
                );
            }
        }
    }
    image
        .save_with_format(output, ImageFormat::Png)
        .unwrap_or_else(|error| panic!("write {context}: {error}"));
}

fn load_saved_viewport_frame(path: &Path) -> ViewportFrame {
    let image = image::open(path)
        .unwrap_or_else(|error| panic!("read saved runtime shader screenshot {path:?}: {error}"))
        .to_rgba8();
    ViewportFrame {
        width: image.width(),
        height: image.height(),
        rgba: image.into_raw(),
        generation: 0,
        capture_report: Default::default(),
    }
}

fn viewport_frame_tile(
    sheet: &ViewportFrame,
    column: u32,
    row: u32,
    tile_size: UVec2,
) -> ViewportFrame {
    let x0 = column * tile_size.x;
    let y0 = row * tile_size.y;
    assert!(
        x0 + tile_size.x <= sheet.width && y0 + tile_size.y <= sheet.height,
        "requested multi-view tile should fit inside saved contact sheet"
    );
    let mut rgba = vec![0_u8; (tile_size.x * tile_size.y * 4) as usize];
    for y in 0..tile_size.y {
        let source_start = (((y0 + y) * sheet.width + x0) * 4) as usize;
        let source_end = source_start + (tile_size.x * 4) as usize;
        let target_start = (y * tile_size.x * 4) as usize;
        rgba[target_start..target_start + (tile_size.x * 4) as usize]
            .copy_from_slice(&sheet.rgba[source_start..source_end]);
    }
    ViewportFrame {
        width: tile_size.x,
        height: tile_size.y,
        rgba,
        generation: 0,
        capture_report: Default::default(),
    }
}
