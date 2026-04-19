use std::fs;
use std::path::Path;

use super::{UiTemplateDocument, UiTemplateError};

#[derive(Default)]
pub struct UiTemplateLoader;

impl UiTemplateLoader {
    pub fn load_toml_str(input: &str) -> Result<UiTemplateDocument, UiTemplateError> {
        toml::from_str(input).map_err(|error| UiTemplateError::ParseToml(error.to_string()))
    }

    pub fn load_toml_file(path: impl AsRef<Path>) -> Result<UiTemplateDocument, UiTemplateError> {
        let input =
            fs::read_to_string(path).map_err(|error| UiTemplateError::Io(error.to_string()))?;
        Self::load_toml_str(&input)
    }
}
