use crate::core::host::asset_editor::preview::PreviewPalette;

pub(super) fn preview_palette(kind: zircon_runtime::asset::AssetKind) -> PreviewPalette {
    match kind {
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
        zircon_runtime::asset::AssetKind::PhysicsMaterial => PreviewPalette {
            primary: [114, 134, 87, 255],
            secondary: [48, 63, 34, 255],
            accent: [217, 236, 190, 255],
            banner: [235, 246, 222, 255],
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
