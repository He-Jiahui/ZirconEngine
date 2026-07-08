use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::{Image, Rgba8Pixel, SharedPixelBuffer};
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
};

const SURFACE_MEDIA_COMPONENT_SCREENSHOT: &str =
    "editor-components-surfaces-media-spacing-900x360.png";
const SURFACE_MEDIA_ATLAS_WIDTH: u32 = 900;
const SURFACE_MEDIA_ATLAS_HEIGHT: u32 = 360;
const SURFACE_MEDIA_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];

#[test]
fn surface_media_spacing_component_visual_paints_layered_surfaces_images_and_gutters() {
    let bytes = surface_media_component_bytes();

    let shell_surface = pixel_at(&bytes, 8, 8);
    let left_panel_surface = pixel_at(&bytes, 28, 92);
    let left_inset_surface = pixel_at(&bytes, 50, 140);
    assert_ne!(
        left_panel_surface, shell_surface,
        "panel containers should paint a distinct Slate-style layer above the shell"
    );
    assert_ne!(
        left_inset_surface, left_panel_surface,
        "inset/content wells should be visibly recessed from their parent panel"
    );

    assert_eq!(
        pixel_at(&bytes, 294, 140),
        shell_surface,
        "top-level gutter between component groups should remain shell background"
    );
    assert_eq!(
        pixel_at(&bytes, 594, 140),
        shell_surface,
        "top-level gutter should not be consumed by adjacent panels"
    );

    let asset_panel_surface = pixel_at(&bytes, 320, 92);
    assert!(
        distinct_pixel_count(
            &bytes,
            354,
            146,
            56,
            56,
            &[
                SURFACE_MEDIA_ATLAS_BACKGROUND,
                shell_surface,
                asset_panel_surface
            ],
        ) > 0,
        "asset preview visual should paint projected image pixels inside its retained well"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            482,
            152,
            48,
            48,
            &[
                SURFACE_MEDIA_ATLAS_BACKGROUND,
                shell_surface,
                asset_panel_surface
            ],
        ) > 0,
        "asset placeholder visual should paint semantic media/icon pixels inside its retained well"
    );

    let image_panel_surface = pixel_at(&bytes, 622, 92);
    assert!(
        distinct_pixel_count(
            &bytes,
            676,
            142,
            92,
            54,
            &[
                SURFACE_MEDIA_ATLAS_BACKGROUND,
                shell_surface,
                image_panel_surface
            ],
        ) > 0,
        "generic image node should paint a fitted bitmap within the image well"
    );
    let spacing_left_surface = pixel_at(&bytes, 644, 266);
    let spacing_gutter_surface = pixel_at(&bytes, 733, 266);
    let spacing_right_surface = pixel_at(&bytes, 754, 266);
    assert_ne!(
        spacing_left_surface, spacing_gutter_surface,
        "inner gutter should preserve parent panel paint between compact child surfaces"
    );
    assert_ne!(
        spacing_right_surface, spacing_gutter_surface,
        "inner gutter should remain visibly separate from the following child surface"
    );
}

#[test]
#[ignore = "writes local surface/media/spacing component screenshot artifact for visual review"]
fn capture_surface_media_spacing_component_visual_artifact() {
    let bytes = surface_media_component_bytes();
    let output_path = visual_layout_output_path(SURFACE_MEDIA_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        SURFACE_MEDIA_ATLAS_WIDTH,
        SURFACE_MEDIA_ATLAS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("surface/media/spacing component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn surface_media_component_bytes() -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        SURFACE_MEDIA_ATLAS_WIDTH,
        SURFACE_MEDIA_ATLAS_HEIGHT,
        SURFACE_MEDIA_ATLAS_BACKGROUND,
        model_rc(surface_media_component_nodes()),
    )
}

fn surface_media_component_nodes() -> Vec<TemplatePaneNodeData> {
    vec![
        surface("SurfaceMediaRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label(
            "SurfaceMediaTitle",
            "Surface, Media and Spacing",
            22.0,
            20.0,
            320.0,
            22.0,
            13.0,
            "",
        ),
        label(
            "SurfaceMediaSubtitle",
            "Layered containers, recessed wells, preview images and preserved gutters",
            22.0,
            42.0,
            620.0,
            18.0,
            10.0,
            "muted",
        ),
        surface("SurfaceStackPanel", "panel", 18.0, 72.0, 260.0, 232.0),
        label(
            "SurfaceStackTitle",
            "Container Stack",
            34.0,
            90.0,
            142.0,
            18.0,
            11.0,
            "",
        ),
        surface(
            "SurfaceStackRaised",
            "component-panel",
            34.0,
            116.0,
            228.0,
            48.0,
        ),
        surface("SurfaceStackInset", "inset", 46.0, 130.0, 204.0, 24.0),
        surface(
            "SurfaceContentPanel",
            "content-panel",
            34.0,
            184.0,
            228.0,
            88.0,
        ),
        label(
            "SurfaceContentLabel",
            "Content well",
            48.0,
            199.0,
            130.0,
            16.0,
            10.0,
            "muted",
        ),
        surface(
            "SurfaceAssetContentBand",
            "asset-content",
            48.0,
            224.0,
            200.0,
            30.0,
        ),
        surface("AssetGridPanel", "panel", 310.0, 72.0, 270.0, 232.0),
        label(
            "AssetGridTitle",
            "Media Tiles",
            326.0,
            90.0,
            110.0,
            18.0,
            11.0,
            "",
        ),
        asset_thumbnail_card("MaterialAssetCard", 328.0, 116.0, 108.0, 164.0, true),
        asset_preview_visual("MaterialPreviewVisual", 338.0, 128.0, 88.0, 86.0),
        surface(
            "MaterialNameArea",
            "asset-thumbnail-name-area",
            338.0,
            226.0,
            88.0,
            34.0,
        ),
        label(
            "MaterialNameText",
            "M_Wall",
            346.0,
            236.0,
            72.0,
            14.0,
            10.0,
            "",
        ),
        asset_thumbnail_card("SceneAssetCard", 454.0, 116.0, 108.0, 164.0, false),
        asset_placeholder_visual("ScenePlaceholderVisual", 464.0, 128.0, 88.0, 86.0),
        surface(
            "SceneNameArea",
            "asset-thumbnail-name-area",
            464.0,
            226.0,
            88.0,
            34.0,
        ),
        label(
            "SceneNameText",
            "Level_01",
            472.0,
            236.0,
            72.0,
            14.0,
            10.0,
            "muted",
        ),
        surface("ImagePreviewPanel", "panel", 612.0, 72.0, 270.0, 232.0),
        label(
            "ImagePreviewTitle",
            "Image Fit + Gaps",
            628.0,
            90.0,
            150.0,
            18.0,
            11.0,
            "",
        ),
        surface(
            "ImagePreviewWell",
            "content-panel",
            632.0,
            118.0,
            222.0,
            112.0,
        ),
        image_node("GenericPreviewImage", 664.0, 134.0, 158.0, 78.0),
        surface("SpacingLeftBlock", "inset", 632.0, 252.0, 92.0, 34.0),
        surface(
            "SpacingRightBlock",
            "asset-type-badge",
            742.0,
            252.0,
            92.0,
            34.0,
        ),
        label(
            "SpacingLeftText",
            "8 px",
            648.0,
            262.0,
            56.0,
            12.0,
            10.0,
            "muted",
        ),
        label(
            "SpacingRightText",
            "12 px",
            760.0,
            262.0,
            56.0,
            12.0,
            10.0,
            "",
        ),
    ]
}

fn asset_thumbnail_card(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    selected: bool,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Panel".into(),
        surface_variant: "asset-thumbnail-card".into(),
        selected,
        focused: selected,
        border_width: 1.0,
        corner_radius: 4.0,
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn asset_preview_visual(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Panel".into(),
        component_role: "asset-thumbnail-visual".into(),
        component_variant: "asset-texture".into(),
        surface_variant: "asset-preview-visual".into(),
        has_preview_image: true,
        preview_image: checker_preview_image(),
        border_width: 1.0,
        corner_radius: 6.0,
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn asset_placeholder_visual(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Panel".into(),
        component_role: "asset-thumbnail-visual".into(),
        component_variant: "asset-scene".into(),
        surface_variant: "asset-placeholder-visual".into(),
        border_width: 1.0,
        corner_radius: 6.0,
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn image_node(control_id: &str, x: f32, y: f32, width: f32, height: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Image".into(),
        has_preview_image: true,
        preview_image: wide_preview_image(),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn surface(
    control_id: &str,
    variant: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Panel".into(),
        surface_variant: variant.into(),
        border_width: 1.0,
        corner_radius: 6.0,
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn label(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    font_size: f32,
    tone: &str,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        font_size,
        text_tone: tone.into(),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn checker_preview_image() -> Image {
    image_from_fn(4, 4, |x, y| match (x + y) % 4 {
        0 => [205, 78, 56, 255],
        1 => [235, 160, 76, 255],
        2 => [56, 150, 178, 255],
        _ => [24, 32, 38, 255],
    })
}

fn wide_preview_image() -> Image {
    image_from_fn(8, 4, |x, y| {
        if y == 0 || y == 3 {
            [30, 37, 43, 255]
        } else if x < 3 {
            [54, 137, 166, 255]
        } else if x < 6 {
            [91, 181, 190, 255]
        } else {
            [218, 126, 58, 255]
        }
    })
}

fn image_from_fn<F>(width: u32, height: u32, mut color_at: F) -> Image
where
    F: FnMut(u32, u32) -> [u8; 4],
{
    let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
    for y in 0..height {
        for x in 0..width {
            pixels.extend_from_slice(&color_at(x, y));
        }
    }
    Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        &pixels, width, height,
    ))
}

fn node_id(control_id: &str) -> String {
    format!("{control_id}.node")
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> TemplateNodeFrameData {
    TemplateNodeFrameData {
        x,
        y,
        width,
        height,
    }
}

fn pixel_at(bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * SURFACE_MEDIA_ATLAS_WIDTH as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}

fn distinct_pixel_count(
    bytes: &[u8],
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    excluded_colors: &[[u8; 4]],
) -> usize {
    let mut changed = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            let index = ((py as usize * SURFACE_MEDIA_ATLAS_WIDTH as usize) + px as usize) * 4;
            let color = [
                bytes[index],
                bytes[index + 1],
                bytes[index + 2],
                bytes[index + 3],
            ];
            if !excluded_colors.contains(&color) {
                changed += 1;
            }
        }
    }
    changed
}

fn visual_layout_output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should live under the repository root")
        .join("docs")
        .join("tests")
        .join("editor")
}

fn visual_layout_output_path(filename: &str) -> PathBuf {
    let output_dir = visual_layout_output_dir();
    std::fs::create_dir_all(&output_dir).expect("visual-layout output directory should exist");
    output_dir.join(filename)
}
