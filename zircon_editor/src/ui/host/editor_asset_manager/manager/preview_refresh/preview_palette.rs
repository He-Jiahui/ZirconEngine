use crate::ui::host::editor_asset_manager::preview::PreviewPalette;

pub(super) fn preview_palette(kind: zircon_runtime::asset::AssetKind) -> PreviewPalette {
    match kind {
        zircon_runtime::asset::AssetKind::Data => PreviewPalette {
            primary: [92, 112, 124, 255],
            secondary: [38, 50, 58, 255],
            accent: [204, 222, 232, 255],
            banner: [232, 241, 246, 255],
        },
        zircon_runtime::asset::AssetKind::Texture => PreviewPalette {
            primary: [74, 127, 173, 255],
            secondary: [35, 55, 82, 255],
            accent: [182, 220, 255, 255],
            banner: [212, 236, 255, 255],
        },
        zircon_runtime::asset::AssetKind::Material => PreviewPalette {
            primary: [156, 112, 66, 255],
            secondary: [74, 49, 28, 255],
            accent: [237, 204, 158, 255],
            banner: [247, 231, 199, 255],
        },
        zircon_runtime::asset::AssetKind::MaterialGraph => PreviewPalette {
            primary: [150, 104, 118, 255],
            secondary: [72, 45, 55, 255],
            accent: [237, 201, 211, 255],
            banner: [249, 231, 236, 255],
        },
        zircon_runtime::asset::AssetKind::PhysicsMaterial => PreviewPalette {
            primary: [114, 134, 87, 255],
            secondary: [48, 63, 34, 255],
            accent: [217, 236, 190, 255],
            banner: [235, 246, 222, 255],
        },
        zircon_runtime::asset::AssetKind::NavMesh => PreviewPalette {
            primary: [72, 142, 116, 255],
            secondary: [32, 70, 57, 255],
            accent: [187, 235, 218, 255],
            banner: [224, 247, 239, 255],
        },
        zircon_runtime::asset::AssetKind::NavigationSettings => PreviewPalette {
            primary: [91, 132, 105, 255],
            secondary: [39, 62, 48, 255],
            accent: [199, 230, 207, 255],
            banner: [230, 245, 234, 255],
        },
        zircon_runtime::asset::AssetKind::Terrain => PreviewPalette {
            primary: [120, 128, 73, 255],
            secondary: [56, 60, 33, 255],
            accent: [226, 232, 180, 255],
            banner: [243, 246, 218, 255],
        },
        zircon_runtime::asset::AssetKind::TerrainLayerStack => PreviewPalette {
            primary: [136, 121, 76, 255],
            secondary: [65, 56, 34, 255],
            accent: [235, 222, 181, 255],
            banner: [249, 241, 218, 255],
        },
        zircon_runtime::asset::AssetKind::TileSet => PreviewPalette {
            primary: [73, 130, 145, 255],
            secondary: [32, 61, 70, 255],
            accent: [190, 229, 239, 255],
            banner: [226, 245, 250, 255],
        },
        zircon_runtime::asset::AssetKind::TileMap => PreviewPalette {
            primary: [82, 118, 158, 255],
            secondary: [35, 55, 78, 255],
            accent: [199, 223, 250, 255],
            banner: [229, 241, 255, 255],
        },
        zircon_runtime::asset::AssetKind::Prefab => PreviewPalette {
            primary: [130, 102, 151, 255],
            secondary: [58, 44, 72, 255],
            accent: [226, 210, 242, 255],
            banner: [242, 234, 250, 255],
        },
        zircon_runtime::asset::AssetKind::Scene => PreviewPalette {
            primary: [67, 118, 91, 255],
            secondary: [31, 60, 48, 255],
            accent: [180, 228, 200, 255],
            banner: [220, 245, 228, 255],
        },
        zircon_runtime::asset::AssetKind::Model => PreviewPalette {
            primary: [102, 97, 145, 255],
            secondary: [46, 43, 73, 255],
            accent: [210, 204, 250, 255],
            banner: [229, 225, 255, 255],
        },
        zircon_runtime::asset::AssetKind::Sound => PreviewPalette {
            primary: [181, 92, 57, 255],
            secondary: [89, 41, 24, 255],
            accent: [255, 213, 186, 255],
            banner: [255, 237, 223, 255],
        },
        zircon_runtime::asset::AssetKind::Font => PreviewPalette {
            primary: [96, 116, 145, 255],
            secondary: [43, 52, 67, 255],
            accent: [214, 225, 244, 255],
            banner: [236, 241, 252, 255],
        },
        zircon_runtime::asset::AssetKind::AnimationSkeleton => PreviewPalette {
            primary: [141, 117, 69, 255],
            secondary: [70, 53, 26, 255],
            accent: [240, 223, 179, 255],
            banner: [250, 240, 214, 255],
        },
        zircon_runtime::asset::AssetKind::AnimationClip => PreviewPalette {
            primary: [88, 131, 166, 255],
            secondary: [39, 62, 81, 255],
            accent: [197, 229, 250, 255],
            banner: [226, 243, 255, 255],
        },
        zircon_runtime::asset::AssetKind::AnimationSequence => PreviewPalette {
            primary: [158, 104, 76, 255],
            secondary: [80, 49, 35, 255],
            accent: [246, 212, 190, 255],
            banner: [255, 236, 224, 255],
        },
        zircon_runtime::asset::AssetKind::AnimationGraph => PreviewPalette {
            primary: [94, 117, 164, 255],
            secondary: [40, 53, 86, 255],
            accent: [206, 220, 255, 255],
            banner: [230, 237, 255, 255],
        },
        zircon_runtime::asset::AssetKind::AnimationStateMachine => PreviewPalette {
            primary: [125, 92, 153, 255],
            secondary: [58, 39, 75, 255],
            accent: [224, 210, 245, 255],
            banner: [241, 234, 255, 255],
        },
        zircon_runtime::asset::AssetKind::Shader => PreviewPalette {
            primary: [170, 80, 97, 255],
            secondary: [78, 31, 43, 255],
            accent: [255, 208, 219, 255],
            banner: [255, 231, 236, 255],
        },
        zircon_runtime::asset::AssetKind::UiLayout => PreviewPalette {
            primary: [65, 112, 148, 255],
            secondary: [29, 54, 71, 255],
            accent: [190, 228, 250, 255],
            banner: [226, 243, 255, 255],
        },
        zircon_runtime::asset::AssetKind::UiWidget => PreviewPalette {
            primary: [116, 98, 169, 255],
            secondary: [52, 44, 81, 255],
            accent: [221, 210, 255, 255],
            banner: [238, 232, 255, 255],
        },
        zircon_runtime::asset::AssetKind::UiStyle => PreviewPalette {
            primary: [164, 113, 55, 255],
            secondary: [79, 53, 24, 255],
            accent: [246, 217, 173, 255],
            banner: [255, 239, 214, 255],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::preview_palette;
    use crate::ui::host::editor_asset_manager::preview::PreviewPalette;

    #[test]
    fn font_assets_have_a_stable_preview_palette() {
        assert_eq!(
            preview_palette(zircon_runtime::asset::AssetKind::Font),
            PreviewPalette {
                primary: [96, 116, 145, 255],
                secondary: [43, 52, 67, 255],
                accent: [214, 225, 244, 255],
                banner: [236, 241, 252, 255],
            }
        );
    }
}
