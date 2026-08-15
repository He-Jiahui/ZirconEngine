use std::{
    ffi::OsString,
    path::{Component, Path, PathBuf},
};

use crate::core::math::UVec2;
use crate::graphics::backend::{read_texture_rgba, RenderBackend};
use crate::text::atlas::render_plan::GlyphAtlasScreenRect;
use crate::text::atlas::{
    glyph_atlas_bitmap_render_submission_plan, GlyphAtlasBitmapSource,
    GlyphAtlasBitmapUploadSourceBytes, GlyphAtlasFormat,
};

use super::GlyphAtlasBitmapRenderer;

const PROOF_FILE_NAME: &str = "runtime_text_native_bitmap_atlas_product_framebuffer_20260731.png";
const PROOF_WIDTH: u32 = 360;
const PROOF_HEIGHT: u32 = 220;
const ATLAS_SIZE: u32 = 128;
const GLYPH_WIDTH: u32 = 24;
const GLYPH_HEIGHT: u32 = 32;

#[cfg(test)]
#[path = "product_framebuffer/cjk_layout_contract.rs"]
mod cjk_layout_contract;

#[cfg(test)]
#[path = "product_framebuffer/native_layout.rs"]
mod native_layout;

#[test]
fn native_bitmap_product_proof_path_is_workspace_docs_not_target() {
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
    assert!(!output.starts_with(workspace_root.join("target")));
    assert_eq!(
        canonicalize_or_normalize_path(&output)
            .parent()
            .expect("canonical proof path parent"),
        workspace_root
            .join("docs")
            .join("tests")
            .join("runtime")
            .join("text")
            .canonicalize()
            .expect("canonical proof directory"),
    );
}

#[test]
fn native_bitmap_product_proof_rejects_target_directories_that_overlap_docs() {
    let workspace_root = workspace_root();
    let output = proof_path();

    assert!(glyph_atlas_product_proof_is_outside_target(
        &output,
        &workspace_root.join("target"),
    ));
    assert!(!glyph_atlas_product_proof_is_outside_target(
        &output,
        &workspace_root.join("docs"),
    ));
    assert!(!glyph_atlas_product_proof_is_outside_target(
        &output,
        &workspace_root
            .join("docs")
            .join("tests")
            .join("runtime")
            .join("text")
            .join("..")
            .join("text"),
    ));
    #[cfg(windows)]
    assert!(!glyph_atlas_product_proof_is_outside_target(
        &output,
        &workspace_root.join("DOCS"),
    ));
}

#[test]
#[should_panic(expected = "CARGO_TARGET_DIR must be an absolute coordinator path")]
fn native_bitmap_product_proof_rejects_relative_cargo_target_directory() {
    let _ = require_absolute_cargo_target_dir(PathBuf::from("cargo-targets").join("native-proof"));
}

#[test]
#[ignore = "exports an explicit runtime WGPU native bitmap atlas framebuffer proof"]
fn render_text_native_bitmap_atlas_product_framebuffer() {
    let backend = RenderBackend::new_offscreen().expect("headless WGPU native bitmap backend");
    let viewport_size = UVec2::new(PROOF_WIDTH, PROOF_HEIGHT);
    let target_format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let target = backend.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-runtime-native-bitmap-atlas-product-target"),
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
    let glyphs = native_bitmap_glyphs();
    let submission = glyph_atlas_bitmap_render_submission_plan(
        glyphs.iter().map(|glyph| glyph.source),
        UVec2::new(ATLAS_SIZE, ATLAS_SIZE),
        1,
        1,
        viewport_size,
        GlyphAtlasScreenRect::new(0.0, 0.0, PROOF_WIDTH as f32, PROOF_HEIGHT as f32),
    );
    assert_eq!(submission.run.glyphs.len(), glyphs.len());
    assert_eq!(submission.run.upload_copies.len(), glyphs.len());

    let mut renderer = GlyphAtlasBitmapRenderer::new(&backend.device, target_format);
    let _shadow_commit = renderer.prepare_submission(
        &backend.device,
        &backend.queue,
        &submission,
        glyphs.iter().enumerate().map(|(source_index, glyph)| {
            GlyphAtlasBitmapUploadSourceBytes::new(source_index, glyph.mask.as_slice())
        }),
        UVec2::new(ATLAS_SIZE, ATLAS_SIZE),
        1,
        GlyphAtlasFormat::AlphaMask,
    );
    let report = renderer.prepare_report();
    assert_eq!(report.storage_pass_visible_glyph_count, glyphs.len());
    assert_eq!(report.upload_request_count, glyphs.len());
    assert!(report.upload_ready_to_write_texture);
    assert_eq!(report.upload_failure_count, 0);
    assert_eq!(report.instance_buffer_reallocation_count, 1);
    assert!(
        report.instance_buffer_capacity_byte_len >= report.vertex_buffer_byte_len,
        "the first native atlas draw must allocate enough persistent instance capacity"
    );
    let first_instance_buffer_capacity = report.instance_buffer_capacity_byte_len;

    let _steady_shadow_commit = renderer.prepare_submission(
        &backend.device,
        &backend.queue,
        &submission,
        glyphs.iter().enumerate().map(|(source_index, glyph)| {
            GlyphAtlasBitmapUploadSourceBytes::new(source_index, glyph.mask.as_slice())
        }),
        UVec2::new(ATLAS_SIZE, ATLAS_SIZE),
        1,
        GlyphAtlasFormat::AlphaMask,
    );
    let steady_report = renderer.prepare_report();
    assert_eq!(steady_report.instance_buffer_reallocation_count, 0);
    assert_eq!(
        steady_report.instance_buffer_capacity_byte_len,
        first_instance_buffer_capacity
    );

    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-runtime-native-bitmap-atlas-product-encoder"),
        });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zircon-runtime-native-bitmap-atlas-product-pass"),
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
        .expect("read native bitmap atlas product framebuffer");
    assert_native_bitmap_layout_pixels(&rgba, &glyphs);

    let output = proof_path();
    assert!(output.components().all(|part| part.as_os_str() != "target"));
    let workspace_target = workspace_root().join("target");
    assert_product_proof_is_outside_target(&output, &workspace_target);
    if let Some(target_dir) = std::env::var_os("CARGO_TARGET_DIR") {
        let target_dir = require_absolute_cargo_target_dir(PathBuf::from(target_dir));
        assert_product_proof_is_outside_target(&output, &target_dir);
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
    .expect("save native bitmap atlas product framebuffer");
    assert!(output.is_file());
    eprintln!(
        "runtime native bitmap atlas framebuffer={}",
        output.display()
    );
}

#[derive(Clone)]
struct NativeBitmapGlyph {
    source: GlyphAtlasBitmapSource,
    mask: Vec<u8>,
}

fn native_bitmap_glyphs() -> [NativeBitmapGlyph; 4] {
    [
        native_bitmap_glyph('Z', 38.0, 48.0, [0.95, 0.98, 1.0, 1.0]),
        native_bitmap_glyph('I', 78.0, 48.0, [0.18, 0.9, 1.0, 1.0]),
        native_bitmap_glyph('R', 118.0, 48.0, [1.0, 0.6, 0.12, 1.0]),
        native_bitmap_glyph('Z', 38.0, 112.0, [0.82, 0.3, 1.0, 1.0]),
    ]
}

fn native_bitmap_glyph(
    letter: char,
    x: f32,
    y: f32,
    foreground_color: [f32; 4],
) -> NativeBitmapGlyph {
    let content_size = UVec2::new(GLYPH_WIDTH, GLYPH_HEIGHT);
    NativeBitmapGlyph {
        source: GlyphAtlasBitmapSource {
            raster_key: None,
            format: GlyphAtlasFormat::AlphaMask,
            content_size,
            screen_rect: GlyphAtlasScreenRect::new(x, y, GLYPH_WIDTH as f32, GLYPH_HEIGHT as f32),
            foreground_color,
            background_color: [0.0, 0.0, 0.0, 0.0],
            source_byte_len: (GLYPH_WIDTH * GLYPH_HEIGHT) as usize,
        },
        mask: glyph_mask(letter),
    }
}

fn glyph_mask(letter: char) -> Vec<u8> {
    let mut mask = vec![0; (GLYPH_WIDTH * GLYPH_HEIGHT) as usize];
    for y in 3..GLYPH_HEIGHT - 3 {
        for x in 3..GLYPH_WIDTH - 3 {
            let stroke = match letter {
                'Z' => y < 7 || y >= GLYPH_HEIGHT - 7 || x.abs_diff(GLYPH_WIDTH - 1 - y) < 3,
                'I' => y < 7 || y >= GLYPH_HEIGHT - 7 || x.abs_diff(GLYPH_WIDTH / 2) < 3,
                'R' => {
                    x < 7
                        || (y < 7 && x < GLYPH_WIDTH - 6)
                        || (y > 12 && y < 19 && x < GLYPH_WIDTH - 6)
                        || (x >= GLYPH_WIDTH - 7 && y >= 7 && y < 17)
                        || (y >= 17 && x.abs_diff(y - 2) < 3)
                }
                _ => false,
            };
            if stroke {
                let index = (y * GLYPH_WIDTH + x) as usize;
                mask[index] = 255;
            }
        }
    }
    mask
}

fn assert_native_bitmap_layout_pixels(rgba: &[u8], glyphs: &[NativeBitmapGlyph]) {
    assert_eq!(rgba.len(), (PROOF_WIDTH * PROOF_HEIGHT * 4) as usize);
    let background = [rgba[0], rgba[1], rgba[2], rgba[3]];
    let all_changed = changed_pixels(
        rgba,
        GlyphAtlasScreenRect::new(20.0, 30.0, 160.0, 140.0),
        background,
    );
    assert!(
        all_changed > 900,
        "native bitmap glyph pixels must reach the WGPU target"
    );
    for glyph in glyphs {
        let changed = changed_pixels(rgba, glyph.source.screen_rect, background);
        assert!(
            changed > 120,
            "glyph at ({}, {}) must remain at its planned screen layout position; changed={changed}",
            glyph.source.screen_rect.x,
            glyph.source.screen_rect.y,
        );
    }
}

fn changed_pixels(rgba: &[u8], rect: GlyphAtlasScreenRect, background: [u8; 4]) -> usize {
    let x0 = rect.x.max(0.0).floor() as u32;
    let y0 = rect.y.max(0.0).floor() as u32;
    let x1 = (rect.x + rect.width).min(PROOF_WIDTH as f32).ceil() as u32;
    let y1 = (rect.y + rect.height).min(PROOF_HEIGHT as f32).ceil() as u32;
    (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| ((y * PROOF_WIDTH + x) * 4) as usize))
        .filter(|&offset| rgba[offset..offset + 4] != background)
        .count()
}

fn proof_path() -> PathBuf {
    proof_path_for(PROOF_FILE_NAME)
}

fn proof_path_for(file_name: &str) -> PathBuf {
    workspace_root()
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("text")
        .join(file_name)
}

fn workspace_root() -> PathBuf {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_root
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or(crate_root)
}

fn assert_product_proof_is_outside_target(output: &Path, target_dir: &Path) {
    assert!(
        glyph_atlas_product_proof_is_outside_target(output, target_dir),
        "product framebuffer proof must not be written under cargo target: output={}, target={}",
        output.display(),
        target_dir.display(),
    );
}

fn require_absolute_cargo_target_dir(target_dir: PathBuf) -> PathBuf {
    assert!(
        target_dir.is_absolute(),
        "CARGO_TARGET_DIR must be an absolute coordinator path before exporting a framebuffer proof: {}",
        target_dir.display(),
    );
    target_dir
}

fn glyph_atlas_product_proof_is_outside_target(output: &Path, target_dir: &Path) -> bool {
    let output = canonicalize_or_normalize_path(output);
    let target_dir = canonicalize_or_normalize_path(target_dir);
    !path_starts_with(&output, &target_dir)
}

fn canonicalize_or_normalize_path(path: &Path) -> PathBuf {
    let normalized = normalize_path_lexically(path);
    let mut existing_ancestor = normalized.as_path();
    let mut missing_components = Vec::<OsString>::new();
    while !existing_ancestor.exists() {
        let Some(component) = existing_ancestor.file_name() else {
            return normalized;
        };
        missing_components.push(component.to_os_string());
        let Some(parent) = existing_ancestor.parent() else {
            return normalized;
        };
        existing_ancestor = parent;
    }

    let Ok(mut canonical) = existing_ancestor.canonicalize() else {
        return normalized;
    };
    for component in missing_components.iter().rev() {
        canonical.push(component);
    }
    canonical
}

fn path_starts_with(path: &Path, prefix: &Path) -> bool {
    #[cfg(windows)]
    {
        let mut path_components = path.components();
        return prefix.components().all(|prefix_component| {
            path_components.next().is_some_and(|path_component| {
                path_component
                    .as_os_str()
                    .to_string_lossy()
                    .eq_ignore_ascii_case(&prefix_component.as_os_str().to_string_lossy())
            })
        });
    }

    #[cfg(not(windows))]
    path.starts_with(prefix)
}

fn normalize_path_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}
