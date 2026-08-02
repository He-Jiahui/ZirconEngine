use std::path::PathBuf;

use crate::graphics::backend::{RenderBackend, read_texture_rgba};
use crate::graphics::scene::scene_renderer::ui::render::text_decorations::ScreenSpaceUiTextDecorations;
use crate::graphics::scene::scene_renderer::ui::render::text_effects::{
    ScreenSpaceUiTextEffects, ScreenSpaceUiTextGlow, ScreenSpaceUiTextOutline,
    ScreenSpaceUiTextShadow,
};

use super::super::material::SdfScreenPxRangeMode;
use super::*;

mod assertions;
mod transforms;

use assertions::{
    FramebufferProof, assert_framebuffer_proof_is_outside_target,
    framebuffer_proof_is_outside_target,
};
use transforms::{perspective_about_clip_center, rotation_about_clip_center};

const PROOF_FILE_NAME: &str =
    "runtime_text_sdf_effects_transformed_product_framebuffer_20260713.png";
const PROOF_WIDTH: u32 = 960;
const PROOF_HEIGHT: u32 = 560;

#[test]
fn sdf_product_proof_path_is_workspace_docs_not_target() {
    let workspace_root = workspace_root();
    let output = proof_path();

    assert_eq!(
        output,
        workspace_root
            .join("docs")
            .join("tests")
            .join("runtime")
            .join("text")
            .join(PROOF_FILE_NAME),
    );
    assert!(framebuffer_proof_is_outside_target(
        &output,
        &workspace_root.join("target"),
    ));
    assert!(!framebuffer_proof_is_outside_target(
        &output,
        &workspace_root.join("docs"),
    ));
    #[cfg(windows)]
    assert!(!framebuffer_proof_is_outside_target(
        &output,
        &workspace_root.join("DOCS"),
    ));
}

#[test]
#[should_panic(expected = "CARGO_TARGET_DIR must be an absolute coordinator path")]
fn sdf_product_proof_rejects_relative_cargo_target_directory() {
    let _ = require_absolute_cargo_target_dir(PathBuf::from("cargo-targets").join("sdf-proof"));
}

#[test]
#[ignore = "exports an explicit runtime WGPU SDF effects and transformed-text framebuffer proof"]
fn render_text_sdf_effects_transformed_product_framebuffer() {
    let backend = RenderBackend::new_offscreen().expect("headless WGPU text proof backend");
    let viewport_size = UVec2::new(PROOF_WIDTH, PROOF_HEIGHT);
    let target_format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let target = backend.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-runtime-text-sdf-effects-product-target"),
        size: wgpu::Extent3d {
            width: PROOF_WIDTH,
            height: PROOF_HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: target_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let mut texts = effect_text_batches();
    attach_transforms(&mut texts, viewport_size);
    let atlas_plan = plan_sdf_atlas(&texts);
    let mut text_state = TextRenderState::new(0);
    let asset_manager = ProjectAssetManager::default();
    let atlas_bake =
        text_state.build_sdf_atlas(atlas_plan.atlas_size, &atlas_plan.slots, &asset_manager);
    let mut sdf_cpu_runs = Vec::new();
    text_state.prepare_sdf_runs_cpu_into(&texts, &asset_manager, &mut sdf_cpu_runs);
    assert_eq!(atlas_bake.glyphs.len(), atlas_plan.slots.len());
    assert!(
        atlas_bake.report.visible_glyph_count > 0,
        "the product framebuffer must consume real generated distance-field glyphs"
    );
    assert_eq!(sdf_cpu_runs.len(), texts.len());
    let mut renderer = ScreenSpaceUiSdfRenderer::new(&backend.device, target_format);
    renderer.prepare(
        &backend.device,
        &backend.queue,
        viewport_size,
        &texts,
        &sdf_cpu_runs,
        &[],
        &[],
        &atlas_plan,
        &atlas_bake,
        SdfAtlasCacheReport::default(),
        false,
    );

    let report = renderer.prepare_report();
    assert_eq!(report.text_batch_count, texts.len());
    assert!(report.vertex_count > 180);
    assert!(report.decoration_vertex_count >= 12);
    assert!(report.outline_batch_count >= 2);
    assert!(report.shadow_batch_count >= 2);
    assert!(report.glow_batch_count >= 2);
    assert_eq!(
        renderer
            .draw_plan
            .materials
            .iter()
            .filter(|material| {
                material.projection_mode == SdfScreenPxRangeMode::FragmentDerived
            })
            .count(),
        2,
        "rotated and perspective batches must use the fragment-derived path"
    );

    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-runtime-text-sdf-effects-product-encoder"),
        });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zircon-runtime-text-sdf-effects-product-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.012,
                        g: 0.018,
                        b: 0.032,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        renderer.render(&mut pass);
    }
    backend.queue.submit([encoder.finish()]);
    let rgba = read_texture_rgba(&backend.device, &backend.queue, &target, viewport_size)
        .expect("read SDF effects product framebuffer");
    assert_product_pixels(&rgba);

    let output = proof_path();
    let workspace_root = workspace_root();
    assert_framebuffer_proof_is_outside_target(&output, &workspace_root.join("target"));
    if let Some(target_dir) = std::env::var_os("CARGO_TARGET_DIR") {
        let target_dir = require_absolute_cargo_target_dir(PathBuf::from(target_dir));
        assert_framebuffer_proof_is_outside_target(&output, &target_dir);
    }
    std::fs::create_dir_all(output.parent().expect("text proof output parent"))
        .expect("create text proof output directory");
    image::save_buffer(
        &output,
        &rgba,
        PROOF_WIDTH,
        PROOF_HEIGHT,
        image::ColorType::Rgba8,
    )
    .expect("save SDF effects transformed product framebuffer");
    assert!(output.is_file());
    eprintln!("runtime SDF effects framebuffer={}", output.display());
}

fn effect_text_batches() -> Vec<ScreenSpaceUiTextBatch> {
    let mut fill = styled_text(
        "SDF FILL",
        UiFrame::new(42.0, 34.0, 390.0, 70.0),
        SdfMode::Sdf,
    );
    fill.color = [0.94, 0.97, 1.0, 1.0];

    let mut outline = styled_text(
        "MSDF OUTLINE",
        UiFrame::new(42.0, 116.0, 390.0, 76.0),
        SdfMode::Msdf,
    );
    outline.text_effects.outline = Some(ScreenSpaceUiTextOutline {
        width_px: 3.0,
        color: [0.05, 0.88, 1.0, 1.0],
    });

    let mut shadow = styled_text(
        "MSDF SHADOW",
        UiFrame::new(42.0, 204.0, 390.0, 78.0),
        SdfMode::Msdf,
    );
    shadow.text_effects.shadow = Some(ScreenSpaceUiTextShadow {
        offset_px: [7.0, 6.0],
        color: [1.0, 0.05, 0.72, 0.88],
    });

    let mut glow = styled_text(
        "MTSDF GLOW",
        UiFrame::new(42.0, 296.0, 390.0, 82.0),
        SdfMode::Mtsdf,
    );
    glow.text_effects.glow = Some(ScreenSpaceUiTextGlow {
        radius_px: 6.0,
        color: [1.0, 0.58, 0.04, 0.9],
    });

    let mut decorated = styled_text(
        "UNDERLINE  STRIKE",
        UiFrame::new(42.0, 404.0, 400.0, 84.0),
        SdfMode::Sdf,
    );
    decorated.text_decorations = ScreenSpaceUiTextDecorations {
        underline: true,
        strikethrough: true,
        underline_color: [0.18, 1.0, 0.42, 1.0],
        strikethrough_color: [1.0, 0.18, 0.16, 1.0],
    };

    let mut rotated = styled_text(
        "ROTATED 45",
        UiFrame::new(522.0, 76.0, 330.0, 82.0),
        SdfMode::Msdf,
    );
    rotated.text_effects = ScreenSpaceUiTextEffects {
        outline: Some(ScreenSpaceUiTextOutline {
            width_px: 2.5,
            color: [0.1, 0.82, 1.0, 1.0],
        }),
        shadow: Some(ScreenSpaceUiTextShadow {
            offset_px: [5.0, 5.0],
            color: [0.7, 0.08, 1.0, 0.8],
        }),
        glow: None,
    };

    let mut perspective = styled_text(
        "PERSPECTIVE MTSDF",
        UiFrame::new(500.0, 348.0, 410.0, 86.0),
        SdfMode::Mtsdf,
    );
    perspective.text_effects = ScreenSpaceUiTextEffects {
        outline: None,
        shadow: None,
        glow: Some(ScreenSpaceUiTextGlow {
            radius_px: 5.0,
            color: [0.98, 0.34, 0.08, 0.9],
        }),
    };
    perspective.text_decorations.underline = true;
    perspective.text_decorations.underline_color = [0.96, 0.86, 0.12, 1.0];

    vec![fill, outline, shadow, glow, decorated, rotated, perspective]
}

fn styled_text(text: &str, frame: UiFrame, mode: SdfMode) -> ScreenSpaceUiTextBatch {
    let mut batch = text_batch(text, frame);
    batch.font_size = 44.0;
    batch.line_height = 58.0;
    batch.color = [0.9, 0.94, 1.0, 1.0];
    batch.distance_field_mode = mode;
    batch
}

fn attach_transforms(texts: &mut [ScreenSpaceUiTextBatch], viewport_size: UVec2) {
    let clip_center = |frame: UiFrame| {
        [
            pixel_to_ndc_x(frame.x + frame.width * 0.5, viewport_size.x as f32),
            pixel_to_ndc_y(frame.y + frame.height * 0.5, viewport_size.y as f32),
        ]
    };
    texts[5].clip_transform = Some(rotation_about_clip_center(
        clip_center(texts[5].frame),
        std::f32::consts::FRAC_PI_4,
        [viewport_size.x as f32, viewport_size.y as f32],
    ));
    texts[6].clip_transform = Some(perspective_about_clip_center(
        clip_center(texts[6].frame),
        0.7,
        [1.0, 0.82],
    ));
}

fn assert_product_pixels(rgba: &[u8]) {
    assert_eq!(rgba.len(), (PROOF_WIDTH * PROOF_HEIGHT * 4) as usize);
    let background = [rgba[0], rgba[1], rgba[2], rgba[3]];
    let proof = FramebufferProof {
        rgba,
        width: PROOF_WIDTH,
        height: PROOF_HEIGHT,
        background,
    };
    assert!(proof.changed_pixels(UiFrame::new(20.0, 20.0, 920.0, 520.0), 10) > 12_000);
    assert!(
        proof.soft_edge_pixels() > 1_000,
        "SDF AA edges must remain visible"
    );
    assert!(
        proof.dominant_color_pixels(UiFrame::new(20.0, 95.0, 430.0, 115.0), |p| {
            p[1] > p[0].saturating_add(35) && p[2] > p[0].saturating_add(35)
        }) > 120,
        "cyan outline must occupy real framebuffer pixels"
    );
    assert!(
        proof.dominant_color_pixels(UiFrame::new(20.0, 190.0, 440.0, 110.0), |p| {
            p[0] > p[1].saturating_add(40) && p[2] > p[1].saturating_add(25)
        }) > 100,
        "magenta drop shadow must occupy real framebuffer pixels"
    );
    assert!(
        proof.dominant_color_pixels(UiFrame::new(20.0, 285.0, 440.0, 110.0), |p| {
            p[0] > 150 && p[1] > p[2].saturating_add(30)
        }) > 100,
        "MTSDF glow must occupy true-distance framebuffer pixels"
    );
    assert!(
        proof.changed_pixels(UiFrame::new(430.0, 20.0, 500.0, 280.0), 10) > 2_000,
        "45-degree transformed MSDF text must render"
    );
    let rotated_frame = UiFrame::new(430.0, 20.0, 500.0, 280.0);
    let rotated_axis = proof.changed_pixel_principal_axis_degrees(rotated_frame, 18);
    assert!(
        (38.0..=52.0).contains(&rotated_axis),
        "rotated MSDF screen-space principal axis must remain 45 degrees; actual={rotated_axis}"
    );
    assert!(proof.soft_edge_pixels_in(rotated_frame) > 180);
    assert!(
        proof.changed_pixels(UiFrame::new(450.0, 300.0, 490.0, 200.0), 10) > 2_000,
        "perspective-scaled MTSDF text must render"
    );
    assert!(
        proof.soft_edge_pixels_in(UiFrame::new(450.0, 300.0, 490.0, 200.0)) > 180,
        "perspective fragment-derived MTSDF must retain antialiased edge pixels"
    );
}

fn proof_path() -> PathBuf {
    workspace_root()
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("text")
        .join(PROOF_FILE_NAME)
}

fn workspace_root() -> PathBuf {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_root
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or(crate_root)
}

fn require_absolute_cargo_target_dir(target_dir: PathBuf) -> PathBuf {
    assert!(
        target_dir.is_absolute(),
        "CARGO_TARGET_DIR must be an absolute coordinator path before exporting a framebuffer proof: {}",
        target_dir.display(),
    );
    target_dir
}
