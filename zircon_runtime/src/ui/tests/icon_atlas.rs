use crate::asset::{UiIconAsset, UiIconSource, UiIconSourceKind};
use crate::ui::icon_atlas::{
    parse_ui_svg_icon, UiIconAtlasBuilder, UiIconRasterRequest, UiSvgIconElement,
};

const RUN_ICON_SVG: &str = r##"
<svg width="24" height="24" viewBox="0 0 24 24">
  <path d="M8 5v14l11-7z" fill="#ffffff" />
</svg>
"##;

#[test]
fn icon_atlas_parses_supported_svg_subset() {
    let svg = parse_ui_svg_icon(RUN_ICON_SVG).unwrap();

    assert_eq!(svg.viewport.width, 24.0);
    assert_eq!(svg.viewport.height, 24.0);
    assert_eq!(svg.viewport.view_width, 24.0);
    assert_eq!(
        svg.elements,
        vec![UiSvgIconElement::Path {
            data: "M8 5v14l11-7z".to_string(),
            fill: Some("#ffffff".to_string()),
            stroke: None,
        }]
    );
}

#[test]
fn icon_atlas_assigns_stable_slots_and_uvs() {
    let plan = UiIconAtlasBuilder::new()
        .with_min_side_px(32)
        .with_padding_px(1)
        .build_plan([
            request("icons/save", 16.0, 2.0, RUN_ICON_SVG),
            request("icons/open", 16.0, 1.0, RUN_ICON_SVG),
        ])
        .unwrap();

    assert_eq!(plan.atlas_width, 68);
    assert_eq!(plan.atlas_height, 34);
    assert_eq!(plan.slots.len(), 2);
    assert_eq!(plan.slots[0].icon_id, "icons/open");
    assert_eq!(plan.slots[0].rect.x, 1);
    assert_eq!(plan.slots[0].rect.y, 1);
    assert_eq!(plan.slots[0].pixel_size, 16);
    assert_eq!(plan.slots[1].icon_id, "icons/save");
    assert_eq!(plan.slots[1].rect.x, 35);
    assert_eq!(plan.slots[1].pixel_size, 32);
    assert!(plan.slots[0].uv.min_u > 0.0);
    assert!(plan.slots[1].uv.max_u <= 1.0);
}

#[test]
fn icon_atlas_deduplicates_icon_ids() {
    let plan = UiIconAtlasBuilder::new()
        .build_plan([
            request("icons/save", 16.0, 1.0, RUN_ICON_SVG),
            request("icons/save", 32.0, 1.0, RUN_ICON_SVG),
        ])
        .unwrap();

    assert_eq!(plan.slots.len(), 1);
    assert_eq!(plan.slots[0].icon_id, "icons/save");
    assert_eq!(plan.slots[0].pixel_size, 16);
}

#[test]
fn icon_atlas_accepts_external_icon_assets_without_svg_parse() {
    let icon = UiIconAsset {
        semantic_id: "icons/external".to_string(),
        default_size: 18.0,
        source: UiIconSource {
            kind: UiIconSourceKind::SvgAsset,
            text: None,
            uri: Some("res://ui/icons/external.svg".to_string()),
        },
    };
    let plan = UiIconAtlasBuilder::new()
        .build_plan([UiIconRasterRequest {
            icon_id: "icons/external".to_string(),
            asset: icon,
            dpi_scale: 1.0,
        }])
        .unwrap();

    assert_eq!(plan.slots.len(), 1);
    assert_eq!(plan.slots[0].pixel_size, 18);
    assert!(plan.slots[0].svg.is_none());
}

#[test]
fn icon_atlas_never_rasterizes_below_native_dpi_but_preserves_supersampling() {
    let asset = UiIconAsset {
        semantic_id: "icons/scale-floor".to_string(),
        default_size: 16.0,
        source: UiIconSource {
            kind: UiIconSourceKind::Svg,
            text: Some(RUN_ICON_SVG.to_string()),
            uri: None,
        },
    };

    let plan = UiIconAtlasBuilder::new()
        .build_plan([
            UiIconRasterRequest {
                icon_id: "icons/undersampled".to_string(),
                asset: asset.clone(),
                dpi_scale: 0.75,
            },
            UiIconRasterRequest {
                icon_id: "icons/supersampled".to_string(),
                asset,
                dpi_scale: 1.5,
            },
        ])
        .unwrap();

    assert_eq!(
        plan.slots
            .iter()
            .find(|slot| slot.icon_id == "icons/undersampled")
            .map(|slot| slot.pixel_size),
        Some(16)
    );
    assert_eq!(
        plan.slots
            .iter()
            .find(|slot| slot.icon_id == "icons/supersampled")
            .map(|slot| slot.pixel_size),
        Some(24)
    );
}

#[test]
fn editor_default_icon_pack_parses_and_enters_atlas_plan() {
    let icons = [
        (
            "editor.icons.run",
            include_str!("../../../../zircon_editor/assets/ui/editor/icons/run.icon.toml"),
        ),
        (
            "editor.icons.save",
            include_str!("../../../../zircon_editor/assets/ui/editor/icons/save.icon.toml"),
        ),
        (
            "editor.icons.search",
            include_str!("../../../../zircon_editor/assets/ui/editor/icons/search.icon.toml"),
        ),
    ]
    .into_iter()
    .map(|(icon_id, source)| {
        let asset = UiIconAsset::from_toml_str(source).unwrap();
        UiIconRasterRequest {
            icon_id: icon_id.to_string(),
            asset,
            dpi_scale: 1.0,
        }
    })
    .collect::<Vec<_>>();

    let plan = UiIconAtlasBuilder::new()
        .with_min_side_px(64)
        .build_plan(icons)
        .unwrap();

    assert_eq!(plan.slots.len(), 3);
    assert_eq!(plan.slots[0].icon_id, "editor.icons.run");
    assert_eq!(plan.slots[1].icon_id, "editor.icons.save");
    assert_eq!(plan.slots[2].icon_id, "editor.icons.search");
    assert!(plan.slots.iter().all(|slot| slot.svg.is_some()));
}

fn request(icon_id: &str, size: f32, dpi_scale: f32, svg: &str) -> UiIconRasterRequest {
    UiIconRasterRequest {
        icon_id: icon_id.to_string(),
        asset: UiIconAsset {
            semantic_id: icon_id.to_string(),
            default_size: size,
            source: UiIconSource {
                kind: UiIconSourceKind::Svg,
                text: Some(svg.to_string()),
                uri: None,
            },
        },
        dpi_scale,
    }
}
