use std::collections::HashSet;

use serde::Deserialize;

use crate::scene::SystemStage;

use super::behavior_validation::ZIRCON_NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3;

mod system_access;

pub(super) use system_access::{
    NativePluginRegistrationThreadAffinity, NativeSystemAccessAuthority,
    NativeSystemAccessAuthorityError, NativeSystemAccessContractError,
    NativeSystemAccessDeclaration, NativeSystemAccessDomain, NativeSystemAccessMode,
    NativeSystemAccessPlan, NativeSystemAccessResolveError, NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY,
};

pub(super) type NativePluginRegistrationManifestResult<T> =
    std::result::Result<T, NativePluginRegistrationManifestError>;

#[derive(Debug)]
pub(super) enum NativePluginRegistrationManifestError {
    InvalidToml(toml::de::Error),
    UnsupportedSchema {
        actual: String,
        expected: &'static str,
    },
    UnsupportedSystemStage {
        stage: String,
    },
    MissingSystemField {
        system_id: String,
        field_name: &'static str,
    },
    InvalidSystemAccess {
        system_id: String,
        source: NativeSystemAccessContractError,
    },
    InvalidResourceField {
        resource_id: String,
        field_name: &'static str,
    },
    DuplicateResourceId {
        resource_id: String,
    },
}

impl std::fmt::Display for NativePluginRegistrationManifestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidToml(error) => {
                write!(
                    formatter,
                    "native registration manifest TOML is invalid: {error}"
                )
            }
            Self::UnsupportedSchema { actual, expected } => write!(
                formatter,
                "native registration manifest schema `{actual}` is unsupported; expected {expected}"
            ),
            Self::UnsupportedSystemStage { stage } => write!(
                formatter,
                "native registration manifest system stage `{stage}` is unsupported"
            ),
            Self::MissingSystemField {
                system_id,
                field_name,
            } => write!(
                formatter,
                "native registration manifest system `{system_id}` is missing {field_name}"
            ),
            Self::InvalidSystemAccess { system_id, source } => write!(
                formatter,
                "native registration manifest system `{system_id}` has invalid access: {source}"
            ),
            Self::InvalidResourceField {
                resource_id,
                field_name,
            } => write!(
                formatter,
                "native registration manifest resource `{resource_id}` has invalid {field_name}"
            ),
            Self::DuplicateResourceId { resource_id } => write!(
                formatter,
                "native registration manifest resource `{resource_id}` is declared more than once"
            ),
        }
    }
}

impl std::error::Error for NativePluginRegistrationManifestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidToml(error) => Some(error),
            Self::InvalidSystemAccess { source, .. } => Some(source),
            Self::UnsupportedSchema { .. }
            | Self::UnsupportedSystemStage { .. }
            | Self::MissingSystemField { .. }
            | Self::InvalidResourceField { .. }
            | Self::DuplicateResourceId { .. } => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NativePluginRegistrationManifest {
    pub schema: String,
    #[serde(default)]
    pub modules: Vec<NativePluginRegistrationModule>,
    #[serde(default)]
    pub systems: Vec<NativePluginRegistrationSystem>,
    #[serde(default)]
    pub resources: Vec<NativePluginRegistrationResource>,
    #[serde(default)]
    pub events: Vec<NativePluginRegistrationEvent>,
    #[serde(default)]
    pub extensions: Vec<NativePluginRegistrationExtension>,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

impl NativePluginRegistrationManifest {
    pub(super) fn from_toml(text: &str) -> NativePluginRegistrationManifestResult<Self> {
        let manifest: Self =
            toml::from_str(text).map_err(NativePluginRegistrationManifestError::InvalidToml)?;
        manifest.validate()?;
        Ok(manifest)
    }

    fn validate(&self) -> NativePluginRegistrationManifestResult<()> {
        if self.schema.trim() != ZIRCON_NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3 {
            return Err(NativePluginRegistrationManifestError::UnsupportedSchema {
                actual: self.schema.clone(),
                expected: ZIRCON_NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3,
            });
        }
        for system in &self.systems {
            system.stage()?;
            system.bridge_interface()?;
            system.bridge_method()?;
            system.access_plan(&self.capabilities)?;
        }
        let mut resource_ids = HashSet::with_capacity(self.resources.len());
        for resource in &self.resources {
            resource.validate()?;
            if !resource_ids.insert(resource.id.as_str()) {
                return Err(NativePluginRegistrationManifestError::DuplicateResourceId {
                    resource_id: resource.id.clone(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NativePluginRegistrationModule {
    pub name: String,
    pub kind: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NativePluginRegistrationSystem {
    pub id: String,
    pub module: String,
    pub stage: String,
    #[serde(default)]
    pub order: i32,
    #[serde(default)]
    pub sets: Vec<String>,
    #[serde(default)]
    pub before: Vec<String>,
    #[serde(default)]
    pub after: Vec<String>,
    #[serde(default)]
    pub access: Vec<String>,
    #[serde(default)]
    pub thread_affinity: NativePluginRegistrationThreadAffinity,
    pub bridge_interface: Option<String>,
    pub bridge_method: Option<String>,
}

impl NativePluginRegistrationSystem {
    pub(super) fn stage(&self) -> NativePluginRegistrationManifestResult<SystemStage> {
        system_stage_from_manifest(&self.stage)
    }

    pub(super) fn bridge_interface(&self) -> NativePluginRegistrationManifestResult<&str> {
        required_non_empty(
            self.bridge_interface.as_deref(),
            &self.id,
            "bridge_interface",
        )
    }

    pub(super) fn bridge_method(&self) -> NativePluginRegistrationManifestResult<&str> {
        required_non_empty(self.bridge_method.as_deref(), &self.id, "bridge_method")
    }

    pub(super) fn access_plan(
        &self,
        capabilities: &[String],
    ) -> NativePluginRegistrationManifestResult<NativeSystemAccessPlan> {
        NativeSystemAccessPlan::from_manifest(self.thread_affinity, &self.access, capabilities)
            .map_err(
                |source| NativePluginRegistrationManifestError::InvalidSystemAccess {
                    system_id: self.id.clone(),
                    source,
                },
            )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NativePluginRegistrationResource {
    pub id: String,
    pub module: String,
    pub schema: String,
}

impl NativePluginRegistrationResource {
    fn validate(&self) -> NativePluginRegistrationManifestResult<()> {
        for (field_name, value) in [
            ("id", self.id.as_str()),
            ("module", self.module.as_str()),
            ("schema", self.schema.as_str()),
        ] {
            if value.trim().is_empty() || value.trim() != value {
                return Err(
                    NativePluginRegistrationManifestError::InvalidResourceField {
                        resource_id: self.id.clone(),
                        field_name,
                    },
                );
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NativePluginRegistrationEvent {
    pub namespace: String,
    pub name: String,
    #[serde(default)]
    pub stable_hash: u64,
    pub schema: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NativePluginRegistrationExtension {
    pub point: String,
    pub contribution: String,
    pub schema: String,
}

fn system_stage_from_manifest(stage: &str) -> NativePluginRegistrationManifestResult<SystemStage> {
    match stage {
        "First" => Ok(SystemStage::First),
        "PreUpdate" => Ok(SystemStage::PreUpdate),
        "FixedFirst" => Ok(SystemStage::FixedFirst),
        "FixedUpdate" => Ok(SystemStage::FixedUpdate),
        "FixedPostUpdate" => Ok(SystemStage::FixedPostUpdate),
        "Update" => Ok(SystemStage::Update),
        "PostUpdate" => Ok(SystemStage::PostUpdate),
        "Last" => Ok(SystemStage::Last),
        "RenderExtract" => Ok(SystemStage::RenderExtract),
        _ => Err(
            NativePluginRegistrationManifestError::UnsupportedSystemStage {
                stage: stage.to_string(),
            },
        ),
    }
}

fn required_non_empty<'a>(
    value: Option<&'a str>,
    system_id: &str,
    field_name: &'static str,
) -> NativePluginRegistrationManifestResult<&'a str> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(
            || NativePluginRegistrationManifestError::MissingSystemField {
                system_id: system_id.to_string(),
                field_name,
            },
        )
}

#[cfg(test)]
#[path = "tests/registration_manifest.rs"]
mod tests;
