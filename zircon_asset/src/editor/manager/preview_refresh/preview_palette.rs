use super::super::super::preview::PreviewPalette;

pub(super) fn preview_palette(kind: crate::AssetKind) -> PreviewPalette {
    match kind {
        crate::AssetKind::Texture => PreviewPalette {
            primary: [74, 127, 173, 255],
            secondary: [35, 55, 82, 255],
            accent: [182, 220, 255, 255],
            banner: [212, 236, 255, 255],
        },
        crate::AssetKind::Material => PreviewPalette {
            primary: [156, 112, 66, 255],
            secondary: [74, 49, 28, 255],
            accent: [237, 204, 158, 255],
            banner: [247, 231, 199, 255],
        },
        crate::AssetKind::Scene => PreviewPalette {
            primary: [67, 118, 91, 255],
            secondary: [31, 60, 48, 255],
            accent: [180, 228, 200, 255],
            banner: [220, 245, 228, 255],
        },
        crate::AssetKind::Model => PreviewPalette {
            primary: [102, 97, 145, 255],
            secondary: [46, 43, 73, 255],
            accent: [210, 204, 250, 255],
            banner: [229, 225, 255, 255],
        },
        crate::AssetKind::Shader => PreviewPalette {
            primary: [170, 80, 97, 255],
            secondary: [78, 31, 43, 255],
            accent: [255, 208, 219, 255],
            banner: [255, 231, 236, 255],
        },
        crate::AssetKind::UiLayout => PreviewPalette {
            primary: [65, 112, 148, 255],
            secondary: [29, 54, 71, 255],
            accent: [190, 228, 250, 255],
            banner: [226, 243, 255, 255],
        },
        crate::AssetKind::UiWidget => PreviewPalette {
            primary: [116, 98, 169, 255],
            secondary: [52, 44, 81, 255],
            accent: [221, 210, 255, 255],
            banner: [238, 232, 255, 255],
        },
        crate::AssetKind::UiStyle => PreviewPalette {
            primary: [164, 113, 55, 255],
            secondary: [79, 53, 24, 255],
            accent: [246, 217, 173, 255],
            banner: [255, 239, 214, 255],
        },
    }
}
