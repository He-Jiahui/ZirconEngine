use std::path::PathBuf;

use super::{SettingsAuthority, SettingsLoad, SettingsRegistry, SettingsScope, SettingsStore};

/// Durable provenance for the User settings layer consumed during editor startup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingsUserLayerLoad {
    Loaded {
        path: PathBuf,
        schema_version: u32,
    },
    Missing {
        path: PathBuf,
    },
    Invalid {
        path: Option<PathBuf>,
        message: String,
    },
}

/// Owns the one mutable registry while startup definitions and the User layer are resolved.
pub(crate) struct SettingsStartup {
    registry: SettingsRegistry,
    user_layer_load: SettingsUserLayerLoad,
}

impl SettingsStartup {
    pub(crate) fn load_from_environment(registry: SettingsRegistry) -> Self {
        match SettingsStore::from_user_environment() {
            Ok(store) => Self::load_from_store(registry, &store),
            Err(error) => Self {
                registry,
                user_layer_load: SettingsUserLayerLoad::Invalid {
                    path: None,
                    message: error.to_string(),
                },
            },
        }
    }

    pub(crate) fn load_from_store(mut registry: SettingsRegistry, store: &SettingsStore) -> Self {
        let user_layer_load = match store.load_into(SettingsScope::User, &mut registry) {
            Ok(SettingsLoad::Loaded {
                path,
                schema_version,
                ..
            }) => SettingsUserLayerLoad::Loaded {
                path,
                schema_version,
            },
            Ok(SettingsLoad::Missing { path }) => SettingsUserLayerLoad::Missing { path },
            Err(error) => SettingsUserLayerLoad::Invalid {
                path: Some(store.paths().user().to_path_buf()),
                message: error.to_string(),
            },
        };
        Self {
            registry,
            user_layer_load,
        }
    }

    pub(crate) fn registry(&self) -> &SettingsRegistry {
        &self.registry
    }

    pub(crate) fn user_layer_load(&self) -> &SettingsUserLayerLoad {
        &self.user_layer_load
    }

    pub(crate) fn into_authority(self) -> SettingsAuthority {
        SettingsAuthority::from_startup(self.registry, self.user_layer_load)
    }
}
