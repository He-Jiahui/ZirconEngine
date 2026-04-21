use std::fs;
use std::path::Path;

use crate::ui::template::{UiAssetDocument, UiAssetError};

#[derive(Default)]
pub struct UiAssetLoader;

impl UiAssetLoader {
    pub fn load_toml_str(input: &str) -> Result<UiAssetDocument, UiAssetError> {
        parse_tree_toml(input)
    }

    pub fn load_toml_file(path: impl AsRef<Path>) -> Result<UiAssetDocument, UiAssetError> {
        let input =
            fs::read_to_string(path).map_err(|error| UiAssetError::Io(error.to_string()))?;
        Self::load_toml_str(&input)
    }
}

fn parse_tree_toml(input: &str) -> Result<UiAssetDocument, UiAssetError> {
    let document: UiAssetDocument =
        toml::from_str(input).map_err(|error| UiAssetError::ParseToml(error.to_string()))?;
    document.validate_tree_authority()?;
    Ok(document)
}
