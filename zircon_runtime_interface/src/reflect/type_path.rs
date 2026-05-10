use serde::{Deserialize, Serialize};

use super::ReflectError;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflectTypePath {
    pub type_path: String,
    pub short_type_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_id: Option<String>,
}

impl ReflectTypePath {
    pub fn new(
        type_path: impl Into<String>,
        short_type_path: impl Into<String>,
    ) -> Result<Self, ReflectError> {
        let type_path = type_path.into();
        let short_type_path = short_type_path.into();
        validate_non_empty_type_path(&type_path)?;
        validate_non_empty_short_type_path(&type_path, &short_type_path)?;

        Ok(Self {
            type_path,
            short_type_path,
            module_path: None,
            plugin_id: None,
        })
    }

    pub fn with_module_path(mut self, module_path: impl Into<String>) -> Self {
        self.module_path = Some(module_path.into());
        self
    }

    pub fn with_plugin_id(mut self, plugin_id: impl Into<String>) -> Self {
        self.plugin_id = Some(plugin_id.into());
        self
    }
}

pub(super) fn validate_non_empty_type_path(type_path: &str) -> Result<(), ReflectError> {
    if type_path.trim().is_empty() {
        return Err(ReflectError::InvalidTypePath {
            type_path: type_path.to_string(),
            reason: "type path must not be empty".to_string(),
        });
    }

    Ok(())
}

fn validate_non_empty_short_type_path(
    type_path: &str,
    short_type_path: &str,
) -> Result<(), ReflectError> {
    if short_type_path.trim().is_empty() {
        return Err(ReflectError::InvalidTypePath {
            type_path: type_path.to_string(),
            reason: "short type path must not be empty".to_string(),
        });
    }

    Ok(())
}
