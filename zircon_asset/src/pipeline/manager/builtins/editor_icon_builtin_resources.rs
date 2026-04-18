use crate::{AssetUri, ImportedAsset, TextureAsset};

use super::BUILTIN_EDITOR_ICON_LOCATORS;

pub(super) fn editor_icon_builtin_resources() -> Vec<(&'static str, ImportedAsset)> {
    BUILTIN_EDITOR_ICON_LOCATORS
        .iter()
        .copied()
        .map(|locator| {
            (
                locator,
                ImportedAsset::Texture(TextureAsset {
                    uri: AssetUri::parse(locator).expect("builtin editor icon uri"),
                    width: 1,
                    height: 1,
                    rgba: vec![255, 255, 255, 255],
                }),
            )
        })
        .collect()
}
