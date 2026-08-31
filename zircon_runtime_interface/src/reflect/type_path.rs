use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use super::ReflectError;
use validation::{validate_module_path, validate_plugin_id, validate_short_type_path};

mod validation;

pub(super) use validation::validate_type_path;
pub use validation::{
    MAX_REFLECT_MODULE_PATH_BYTES, MAX_REFLECT_PLUGIN_ID_BYTES, MAX_REFLECT_SHORT_TYPE_PATH_BYTES,
    MAX_REFLECT_TYPE_PATH_BYTES,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReflectTypePath {
    type_path: String,
    short_type_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    module_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    plugin_id: Option<String>,
}

impl ReflectTypePath {
    pub fn new(
        type_path: impl Into<String>,
        short_type_path: impl Into<String>,
    ) -> Result<Self, ReflectError> {
        let type_path = type_path.into();
        let short_type_path = short_type_path.into();
        validate_type_path(&type_path)?;
        validate_short_type_path(&type_path, &short_type_path)?;

        Ok(Self {
            type_path,
            short_type_path,
            module_path: None,
            plugin_id: None,
        })
    }

    pub fn type_path(&self) -> &str {
        self.type_path.as_str()
    }

    pub fn short_type_path(&self) -> &str {
        self.short_type_path.as_str()
    }

    pub fn module_path(&self) -> Option<&str> {
        self.module_path.as_deref()
    }

    pub fn plugin_id(&self) -> Option<&str> {
        self.plugin_id.as_deref()
    }

    pub fn with_module_path(
        mut self,
        module_path: impl Into<String>,
    ) -> Result<Self, ReflectError> {
        let module_path = module_path.into();
        validate_module_path(&self.type_path, &module_path)?;
        self.module_path = Some(module_path);
        Ok(self)
    }

    pub fn with_plugin_id(mut self, plugin_id: impl Into<String>) -> Result<Self, ReflectError> {
        let plugin_id = plugin_id.into();
        validate_plugin_id(&self.type_path, &plugin_id)?;
        self.plugin_id = Some(plugin_id);
        Ok(self)
    }
}

impl<'de> Deserialize<'de> for ReflectTypePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct ReflectTypePathWire {
            type_path: String,
            short_type_path: String,
            #[serde(default)]
            module_path: Option<String>,
            #[serde(default)]
            plugin_id: Option<String>,
        }

        let wire = ReflectTypePathWire::deserialize(deserializer)?;
        let mut type_path =
            Self::new(wire.type_path, wire.short_type_path).map_err(D::Error::custom)?;
        if let Some(module_path) = wire.module_path {
            type_path = type_path
                .with_module_path(module_path)
                .map_err(D::Error::custom)?;
        }
        if let Some(plugin_id) = wire.plugin_id {
            type_path = type_path
                .with_plugin_id(plugin_id)
                .map_err(D::Error::custom)?;
        }
        Ok(type_path)
    }
}
