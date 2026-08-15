use crate::asset::MeshSdfCookSettings;

const MESH_SDF_IMPORT_SETTING: &str = "mesh_sdf";
const ENABLED_IMPORT_SETTING: &str = "enabled";
const MAX_DIMENSION_IMPORT_SETTING: &str = "max_dimension";
const MAX_VOXEL_COUNT_IMPORT_SETTING: &str = "max_voxel_count";
const MAX_PAYLOAD_BYTES_IMPORT_SETTING: &str = "max_payload_bytes";
const SURFACE_BAND_VOXELS_IMPORT_SETTING: &str = "surface_band_voxels";
const TWO_SIDED_IMPORT_SETTING: &str = "two_sided";
const MIN_MESH_SDF_DIMENSION: u64 = 4;
const MAX_MESH_SDF_DIMENSION: u64 = 256;

/// Typed opt-in policy for import-time Mesh SDF generation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum MeshSdfCookRequest {
    #[default]
    Disabled,
    Enabled(MeshSdfCookSettings),
}

impl MeshSdfCookRequest {
    pub fn from_import_settings(import_settings: &toml::Table) -> Result<Self, String> {
        let Some(value) = import_settings.get(MESH_SDF_IMPORT_SETTING) else {
            return Ok(Self::Disabled);
        };
        let table = value
            .as_table()
            .ok_or_else(|| format!("import setting `{MESH_SDF_IMPORT_SETTING}` must be a table"))?;
        let enabled = required_bool(table, ENABLED_IMPORT_SETTING)?;
        if !enabled {
            return Ok(Self::Disabled);
        }

        let defaults = MeshSdfCookSettings::default();
        let settings = MeshSdfCookSettings {
            max_dimension: optional_u64(
                table,
                MAX_DIMENSION_IMPORT_SETTING,
                u64::from(defaults.max_dimension),
            )?
            .try_into()
            .map_err(|_| invalid_integer(MAX_DIMENSION_IMPORT_SETTING))?,
            max_voxel_count: optional_u64(
                table,
                MAX_VOXEL_COUNT_IMPORT_SETTING,
                defaults.max_voxel_count,
            )?,
            max_payload_bytes: optional_u64(
                table,
                MAX_PAYLOAD_BYTES_IMPORT_SETTING,
                defaults.max_payload_bytes,
            )?,
            surface_band_voxels: optional_u64(
                table,
                SURFACE_BAND_VOXELS_IMPORT_SETTING,
                u64::from(defaults.surface_band_voxels),
            )?
            .try_into()
            .map_err(|_| invalid_integer(SURFACE_BAND_VOXELS_IMPORT_SETTING))?,
            two_sided: optional_bool(table, TWO_SIDED_IMPORT_SETTING, defaults.two_sided)?,
        };
        validate_settings(settings)?;
        Ok(Self::Enabled(settings))
    }

    pub fn settings(&self) -> Option<MeshSdfCookSettings> {
        match self {
            Self::Disabled => None,
            Self::Enabled(settings) => Some(*settings),
        }
    }
}

fn validate_settings(settings: MeshSdfCookSettings) -> Result<(), String> {
    if !(MIN_MESH_SDF_DIMENSION..=MAX_MESH_SDF_DIMENSION)
        .contains(&u64::from(settings.max_dimension))
    {
        return Err(format!(
            "import setting `{MESH_SDF_IMPORT_SETTING}.{MAX_DIMENSION_IMPORT_SETTING}` must be between {MIN_MESH_SDF_DIMENSION} and {MAX_MESH_SDF_DIMENSION}"
        ));
    }
    if settings.max_voxel_count < MIN_MESH_SDF_DIMENSION.pow(3)
        || settings.max_payload_bytes == 0
        || settings.surface_band_voxels == 0
    {
        return Err(format!(
            "import settings `{MESH_SDF_IMPORT_SETTING}` must provide positive budgets and surface band"
        ));
    }
    Ok(())
}

fn required_bool(table: &toml::Table, key: &str) -> Result<bool, String> {
    table
        .get(key)
        .and_then(toml::Value::as_bool)
        .ok_or_else(|| {
            format!("import setting `{MESH_SDF_IMPORT_SETTING}.{key}` must be a boolean")
        })
}

fn optional_bool(table: &toml::Table, key: &str, default: bool) -> Result<bool, String> {
    table.get(key).map_or(Ok(default), |value| {
        value.as_bool().ok_or_else(|| {
            format!("import setting `{MESH_SDF_IMPORT_SETTING}.{key}` must be a boolean")
        })
    })
}

fn optional_u64(table: &toml::Table, key: &str, default: u64) -> Result<u64, String> {
    let Some(value) = table.get(key) else {
        return Ok(default);
    };
    value
        .as_integer()
        .and_then(|value| u64::try_from(value).ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| invalid_integer(key))
}

fn invalid_integer(key: &str) -> String {
    format!("import setting `{MESH_SDF_IMPORT_SETTING}.{key}` must be a positive integer")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh_sdf_cook_is_disabled_without_an_explicit_request() {
        assert_eq!(
            MeshSdfCookRequest::from_import_settings(&toml::Table::new()).unwrap(),
            MeshSdfCookRequest::Disabled
        );
    }

    #[test]
    fn enabled_request_preserves_bounded_settings() {
        let settings = toml::from_str(
            r#"
                [mesh_sdf]
                enabled = true
                max_dimension = 24
                max_voxel_count = 8192
                max_payload_bytes = 32768
                surface_band_voxels = 3
                two_sided = true
            "#,
        )
        .unwrap();

        assert_eq!(
            MeshSdfCookRequest::from_import_settings(&settings)
                .unwrap()
                .settings(),
            Some(MeshSdfCookSettings {
                max_dimension: 24,
                max_voxel_count: 8192,
                max_payload_bytes: 32768,
                surface_band_voxels: 3,
                two_sided: true,
            })
        );
    }

    #[test]
    fn malformed_or_unbounded_settings_are_rejected() {
        let settings = toml::from_str(
            r#"
                [mesh_sdf]
                enabled = true
                max_dimension = 1024
            "#,
        )
        .unwrap();

        assert!(MeshSdfCookRequest::from_import_settings(&settings)
            .unwrap_err()
            .contains("max_dimension"));
    }
}
