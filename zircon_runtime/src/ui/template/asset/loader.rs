use std::fs;
use std::path::Path;

use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiAssetMigrationOutcome,
};

use super::schema::UiAssetSchemaMigrator;

#[derive(Default)]
pub struct UiAssetLoader;

impl UiAssetLoader {
    pub fn load_toml_str(input: &str) -> Result<UiAssetDocument, UiAssetError> {
        Ok(Self::load_toml_str_with_migration_report(input)?.document)
    }

    pub fn load_toml_str_with_migration_report(
        input: &str,
    ) -> Result<UiAssetMigrationOutcome, UiAssetError> {
        UiAssetSchemaMigrator::migrate_toml_str(input)
    }

    pub fn load_toml_file(path: impl AsRef<Path>) -> Result<UiAssetDocument, UiAssetError> {
        let input =
            fs::read_to_string(path).map_err(|error| UiAssetError::Io(error.to_string()))?;
        Self::load_toml_str(&input)
    }

    pub fn load_toml_file_with_migration_report(
        path: impl AsRef<Path>,
    ) -> Result<UiAssetMigrationOutcome, UiAssetError> {
        let input =
            fs::read_to_string(path).map_err(|error| UiAssetError::Io(error.to_string()))?;
        Self::load_toml_str_with_migration_report(&input)
    }
}
