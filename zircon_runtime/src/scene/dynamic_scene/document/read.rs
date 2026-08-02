use crate::scene::dynamic_scene::{DynamicScene, DynamicSceneError};
use serde_json::value::RawValue;
use zircon_runtime_interface::serialization::{
    Format, LoadError, PayloadHeader, VersionedSchema, load_versioned,
};

impl DynamicScene {
    pub fn from_versioned_json(json: &str) -> Result<Self, DynamicSceneError> {
        let loaded = load_versioned::<Self>(json.as_bytes(), Format::Text)?;
        let scene = loaded.value;
        scene.ensure_supported()?;
        Ok(scene)
    }

    pub(crate) fn from_versioned_json_payload(
        header: PayloadHeader,
        payload: &RawValue,
    ) -> Result<Self, DynamicSceneError> {
        if header.schema_id != Self::SCHEMA {
            return Err(DynamicSceneError::UnsupportedSchema {
                expected: Self::SCHEMA.as_str().to_string(),
                actual: header.schema_id.as_str().to_string(),
            });
        }
        if header.schema_version > Self::VERSION {
            return Err(DynamicSceneError::UnsupportedFormatVersion {
                expected: Self::VERSION,
                actual: header.schema_version,
            });
        }

        let mut scene = if header.schema_version == Self::VERSION {
            serde_json::from_str::<Self>(payload.get()).map_err(|error| {
                DynamicSceneError::Parse {
                    reason: error.to_string(),
                }
            })?
        } else {
            let value =
                serde_json::from_str(payload.get()).map_err(|error| DynamicSceneError::Parse {
                    reason: error.to_string(),
                })?;
            let migrated = Self::migrations()
                .migrate_value(&Self::SCHEMA, value, header.schema_version, Self::VERSION)
                .map_err(|error| DynamicSceneError::from(LoadError::Migration(error)))?;
            serde_json::from_value(migrated).map_err(|error| DynamicSceneError::Parse {
                reason: error.to_string(),
            })?
        };
        scene.payload_header = super::current_dynamic_scene_header();
        scene.ensure_supported()?;
        Ok(scene)
    }
}
