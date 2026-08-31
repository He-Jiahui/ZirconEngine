use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::asset_browser_pane_nodes;
use crate::ui::retained_host::primitives::{Image, Rgba8Pixel, SharedPixelBuffer};
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
};
use crate::ui::workbench::snapshot::{AssetItemSnapshot, AssetViewMode, AssetWorkspaceSnapshot};
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::layout::UiSize;

const ASSET_THUMBNAIL_COMPONENT_SCREENSHOT: &str = "editor-components-asset-thumbnails-900x360.png";
const ASSET_THUMBNAIL_ATLAS_WIDTH: u32 = 900;
const ASSET_THUMBNAIL_ATLAS_HEIGHT: u32 = 360;
const ASSET_THUMBNAIL_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];

#[test]
fn asset_thumbnail_component_visual_paints_preview_name_area_states_and_text() {
    let nodes = asset_thumbnail_component_nodes();
    let projected_wide_name = projected_wide_name_lines();
    assert!(
        nodes.iter().any(|node| {
            node.control_id.as_str() == "MaterialSelectedCard" && node.selected && !node.focused
        }),
        "selected asset tile fixture must not impersonate keyboard focus"
    );
    assert!(
        nodes.iter().any(|node| {
            node.control_id.as_str() == "MaterialSelectedTitle"
                && node.component_role.as_str() == "asset-thumbnail-name-area-text"
                && node.selected
                && !node.focused
        }),
        "selected name-area text should carry the selected name-area text role"
    );
    assert!(
        nodes.iter().any(|node| {
            node.control_id.as_str() == "SceneHoverNameArea"
                && node.hovered
                && node.state_layer_enabled
                && !node.selected
                && !node.focused
        }),
        "hovered name area should exercise the square-top state-layer path without focus"
    );

    let bytes = asset_thumbnail_component_bytes_from_nodes(nodes);
    let panel_surface = pixel_at(&bytes, 30, 92);

    assert!(
        distinct_pixel_count(
            &bytes,
            68,
            142,
            72,
            48,
            &[ASSET_THUMBNAIL_ATLAS_BACKGROUND, panel_surface],
        ) > 0,
        "selected tile should paint real projected preview image pixels inside the retained well"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            238,
            142,
            72,
            48,
            &[ASSET_THUMBNAIL_ATLAS_BACKGROUND, panel_surface],
        ) > 0,
        "semantic placeholder tile should paint the typed main preview icon"
    );

    let selected_name_top = pixel_at(&bytes, 53, 225);
    let selected_card_interior = pixel_at(&bytes, 48, 225);
    assert_ne!(
        selected_name_top, selected_card_interior,
        "selected name-area square top should cover the rounded card corner at the seam"
    );

    let hovered_name_top = pixel_at(&bytes, 393, 225);
    let hovered_card_interior = pixel_at(&bytes, 388, 225);
    assert_ne!(
        hovered_name_top, hovered_card_interior,
        "hovered name-area state layer should keep the square top cap instead of exposing the card surface"
    );

    let selected_name_surface = pixel_at(&bytes, 86, 232);
    let idle_name_surface = pixel_at(&bytes, 566, 232);
    assert_ne!(
        selected_name_surface, idle_name_surface,
        "selected name area should remain visually distinct from an idle tile"
    );

    assert!(
        distinct_pixel_count(
            &bytes,
            60,
            234,
            88,
            24,
            &[
                ASSET_THUMBNAIL_ATLAS_BACKGROUND,
                panel_surface,
                selected_name_surface,
            ],
        ) > 0,
        "selected name-area labels should render above the selected surface"
    );
    assert_eq!(
        projected_wide_name.primary, "NavigationSettings",
        "wide logical thumbnail names should use the Asset Browser runtime-width primary line"
    );
    assert_eq!(
        projected_wide_name.continuation, "RuntimeProfile",
        "wide logical thumbnail names should use the Asset Browser runtime-width continuation line"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            742,
            252,
            94,
            24,
            &[ASSET_THUMBNAIL_ATLAS_BACKGROUND, panel_surface],
        ) > 0,
        "wide-name projection evidence should render the two-line title inside the visual artifact"
    );
}

#[test]
#[ignore = "writes local asset thumbnail component screenshot artifact for visual review"]
fn capture_asset_thumbnail_component_visual_artifact() {
    let bytes = asset_thumbnail_component_bytes();
    let output_path = visual_layout_output_path(ASSET_THUMBNAIL_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        ASSET_THUMBNAIL_ATLAS_WIDTH,
        ASSET_THUMBNAIL_ATLAS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("asset thumbnail component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn asset_thumbnail_component_bytes() -> Vec<u8> {
    asset_thumbnail_component_bytes_from_nodes(asset_thumbnail_component_nodes())
}

fn asset_thumbnail_component_bytes_from_nodes(nodes: Vec<TemplatePaneNodeData>) -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        ASSET_THUMBNAIL_ATLAS_WIDTH,
        ASSET_THUMBNAIL_ATLAS_HEIGHT,
        ASSET_THUMBNAIL_ATLAS_BACKGROUND,
        model_rc(nodes),
    )
}

fn asset_thumbnail_component_nodes() -> Vec<TemplatePaneNodeData> {
    let projected_wide_name = projected_wide_name_lines();
    let mut nodes = vec![
        surface("AssetThumbnailRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label(
            "AssetThumbnailTitle",
            "Asset Thumbnails",
            22.0,
            20.0,
            260.0,
            22.0,
            13.0,
            "",
        ),
        label(
            "AssetThumbnailSubtitle",
            "Content Browser tile states, preview wells, selected name areas and placeholder icons",
            22.0,
            42.0,
            700.0,
            18.0,
            10.0,
            "muted",
        ),
        surface("AssetTilePanel", "panel", 18.0, 72.0, 864.0, 232.0),
    ];

    push_asset_tile(
        &mut nodes,
        AssetTileSpec {
            prefix: "MaterialSelected",
            title: "M_Wall_Surface",
            kind: "MAT",
            variant: "asset-material",
            x: 42.0,
            y: 116.0,
            selected: true,
            focused: false,
            hovered: false,
            has_preview: true,
            tone: "",
        },
    );
    push_asset_tile(
        &mut nodes,
        AssetTileSpec {
            prefix: "SceneFocused",
            title: "Level_01",
            kind: "SCN",
            variant: "asset-scene",
            x: 212.0,
            y: 116.0,
            selected: false,
            focused: true,
            hovered: false,
            has_preview: false,
            tone: "muted",
        },
    );
    push_asset_tile(
        &mut nodes,
        AssetTileSpec {
            prefix: "SceneHover",
            title: "BP_DoorController",
            kind: "PFB",
            variant: "asset-scene",
            x: 382.0,
            y: 116.0,
            selected: false,
            focused: false,
            hovered: true,
            has_preview: false,
            tone: "muted",
        },
    );
    push_asset_tile(
        &mut nodes,
        AssetTileSpec {
            prefix: "TextureIdle",
            title: "T_Trim_Rough",
            kind: "TEX",
            variant: "asset-texture",
            x: 552.0,
            y: 116.0,
            selected: false,
            focused: false,
            hovered: false,
            has_preview: true,
            tone: "muted",
        },
    );

    nodes.extend([
        surface(
            "TileLegendPanel",
            "content-panel",
            728.0,
            116.0,
            126.0,
            166.0,
        ),
        label(
            "TileLegendTitle",
            "State Split",
            742.0,
            132.0,
            92.0,
            16.0,
            10.0,
            "",
        ),
        label(
            "TileLegendSelected",
            "selected != focus",
            742.0,
            160.0,
            98.0,
            16.0,
            9.0,
            "muted",
        ),
        label(
            "TileLegendPreview",
            "image before icon",
            742.0,
            184.0,
            98.0,
            16.0,
            9.0,
            "muted",
        ),
        label(
            "TileLegendName",
            "square name seam",
            742.0,
            208.0,
            104.0,
            16.0,
            9.0,
            "muted",
        ),
        label(
            "TileLegendWideName",
            "runtime width split",
            742.0,
            236.0,
            104.0,
            16.0,
            9.0,
            "muted",
        ),
        label(
            "TileLegendWideNamePrimary",
            projected_wide_name.primary.as_str(),
            742.0,
            252.0,
            98.0,
            14.0,
            9.0,
            "",
        ),
        label(
            "TileLegendWideNameContinuation",
            projected_wide_name.continuation.as_str(),
            742.0,
            266.0,
            98.0,
            12.0,
            8.0,
            "muted",
        ),
    ]);

    nodes
}

struct ProjectedWideNameLines {
    primary: String,
    continuation: String,
}

fn projected_wide_name_lines() -> ProjectedWideNameLines {
    let nodes = asset_browser_pane_nodes(&wide_name_asset_snapshot(), UiSize::new(900.0, 360.0));
    ProjectedWideNameLines {
        primary: find_view_template_text(&nodes, "AssetBrowserThumbName01"),
        continuation: find_view_template_text(&nodes, "AssetBrowserThumbNameContinuation01"),
    }
}

fn wide_name_asset_snapshot() -> AssetWorkspaceSnapshot {
    AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        visible_assets: vec![AssetItemSnapshot {
            uuid: "asset-navigation-profile".to_string(),
            locator: "res://data/NavigationSettingsRuntimeProfile".to_string(),
            display_name: "NavigationSettingsRuntimeProfile".to_string(),
            file_name: "NavigationSettingsRuntimeProfile".to_string(),
            extension: String::new(),
            kind: ResourceKind::Data,
            asset_type:
                crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
                    ResourceKind::Data,
                ),
            preview_artifact_path: String::new(),
            dirty: false,
            diagnostics: Vec::new(),
            selected: true,
            resource_state: None,
            resource_revision: Some(42),
        }]
        .into(),
        ..AssetWorkspaceSnapshot::default()
    }
}

fn find_view_template_text(
    nodes: &crate::ui::retained_host::primitives::ModelRc<
        crate::ui::layouts::views::ViewTemplateNodeData,
    >,
    control_id: &str,
) -> String {
    for index in 0..nodes.row_count() {
        let Some(node) = nodes.row_data(index) else {
            continue;
        };
        if node.control_id.as_str() == control_id {
            return node.text.to_string();
        }
    }
    panic!("missing projected Asset Browser node `{control_id}`");
}

struct AssetTileSpec {
    prefix: &'static str,
    title: &'static str,
    kind: &'static str,
    variant: &'static str,
    x: f32,
    y: f32,
    selected: bool,
    focused: bool,
    hovered: bool,
    has_preview: bool,
    tone: &'static str,
}

fn push_asset_tile(nodes: &mut Vec<TemplatePaneNodeData>, spec: AssetTileSpec) {
    let card_width = 128.0;
    let card_height = 166.0;
    let content_x = spec.x + 10.0;
    let visual_y = spec.y + 10.0;
    let visual_width = card_width - 20.0;
    let visual_height = 88.0;
    let name_y = spec.y + 108.0;
    let name_height = 44.0;

    nodes.push(asset_thumbnail_card(
        &format!("{}Card", spec.prefix),
        spec.x,
        spec.y,
        card_width,
        card_height,
        spec.selected,
        spec.focused,
    ));
    nodes.push(asset_thumbnail_visual(
        &format!("{}Visual", spec.prefix),
        content_x,
        visual_y,
        visual_width,
        visual_height,
        spec.variant,
        spec.focused,
        spec.has_preview,
    ));
    nodes.push(asset_name_area(
        &format!("{}NameArea", spec.prefix),
        content_x,
        name_y,
        visual_width,
        name_height,
        spec.selected,
        spec.hovered,
    ));
    nodes.push(asset_label(
        &format!("{}Title", spec.prefix),
        spec.title,
        content_x + 8.0,
        name_y + 8.0,
        visual_width - 16.0,
        14.0,
        spec.selected,
        spec.tone,
    ));
    nodes.push(asset_label(
        &format!("{}Kind", spec.prefix),
        spec.kind,
        content_x + 8.0,
        name_y + 25.0,
        36.0,
        12.0,
        spec.selected,
        "muted",
    ));
}

fn asset_thumbnail_card(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    selected: bool,
    focused: bool,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Panel".into(),
        surface_variant: "asset-thumbnail-card".into(),
        selected,
        focused,
        border_width: 1.0,
        corner_radius: 4.0,
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn asset_thumbnail_visual(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    variant: &str,
    focused: bool,
    has_preview: bool,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Panel".into(),
        component_role: "asset-thumbnail-visual".into(),
        component_variant: variant.into(),
        surface_variant: if has_preview {
            "asset-preview-visual".into()
        } else {
            "asset-placeholder-visual".into()
        },
        focused,
        has_preview_image: has_preview,
        preview_image: if has_preview {
            checker_preview_image()
        } else {
            Image::default()
        },
        border_width: 1.0,
        corner_radius: 5.0,
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn asset_name_area(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    selected: bool,
    hovered: bool,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Panel".into(),
        surface_variant: "asset-thumbnail-name-area".into(),
        selected,
        hovered,
        state_layer_enabled: hovered,
        border_width: 0.0,
        corner_radius: 4.0,
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn asset_label(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    selected: bool,
    tone: &str,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Label".into(),
        component_role: "asset-thumbnail-name-area-text".into(),
        selected,
        text: text.into(),
        font_size: 10.0,
        text_tone: tone.into(),
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
        0 => [202, 84, 58, 255],
        1 => [224, 148, 64, 255],
        2 => [62, 148, 180, 255],
        _ => [27, 35, 42, 255],
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
    let index = ((y as usize * ASSET_THUMBNAIL_ATLAS_WIDTH as usize) + x as usize) * 4;
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
            let index = ((py as usize * ASSET_THUMBNAIL_ATLAS_WIDTH as usize) + px as usize) * 4;
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
