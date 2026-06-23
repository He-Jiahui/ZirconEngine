use serde::Deserialize;

use crate::scene::SystemStage;

use super::behavior_validation::ZIRCON_NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3;

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
    pub(super) fn from_toml(text: &str) -> Result<Self, String> {
        let manifest: Self = toml::from_str(text)
            .map_err(|error| format!("native registration manifest TOML is invalid: {error}"))?;
        manifest.validate()?;
        Ok(manifest)
    }

    fn validate(&self) -> Result<(), String> {
        if self.schema.trim() != ZIRCON_NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3 {
            return Err(format!(
                "native registration manifest schema `{}` is unsupported; expected {}",
                self.schema, ZIRCON_NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3
            ));
        }
        for system in &self.systems {
            system.stage()?;
            system.bridge_interface()?;
            system.bridge_method()?;
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
    pub bridge_interface: Option<String>,
    pub bridge_method: Option<String>,
}

impl NativePluginRegistrationSystem {
    pub(super) fn stage(&self) -> Result<SystemStage, String> {
        system_stage_from_manifest(&self.stage)
    }

    pub(super) fn bridge_interface(&self) -> Result<&str, String> {
        required_non_empty(
            self.bridge_interface.as_deref(),
            &self.id,
            "bridge_interface",
        )
    }

    pub(super) fn bridge_method(&self) -> Result<&str, String> {
        required_non_empty(self.bridge_method.as_deref(), &self.id, "bridge_method")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NativePluginRegistrationResource {
    pub id: String,
    pub module: String,
    pub schema: String,
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

fn system_stage_from_manifest(stage: &str) -> Result<SystemStage, String> {
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
        _ => Err(format!(
            "native registration manifest system stage `{stage}` is unsupported"
        )),
    }
}

fn required_non_empty<'a>(
    value: Option<&'a str>,
    system_id: &str,
    field_name: &str,
) -> Result<&'a str, String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            format!("native registration manifest system `{system_id}` is missing {field_name}")
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_registration_manifest_parses_bridge_systems() {
        let manifest = NativePluginRegistrationManifest::from_toml(
            r#"
schema = "zircon.native.registration-manifest/3"
capabilities = ["runtime.plugin.physics"]

[[modules]]
name = "runtime"
kind = "runtime"

[[systems]]
id = "physics.runtime_tick"
module = "runtime"
stage = "Update"
order = 2
sets = ["physics.tick"]
before = ["physics.render"]
after = ["physics.bootstrap"]
access = ["write:world"]
bridge_interface = "native.live_host.bridge.v1"
bridge_method = "sample_count"
"#,
        )
        .expect("native registration manifest should parse");

        assert_eq!(
            manifest.schema,
            ZIRCON_NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3
        );
        assert_eq!(manifest.modules.len(), 1);
        assert_eq!(manifest.systems.len(), 1);
        assert_eq!(manifest.systems[0].stage().unwrap(), SystemStage::Update);
        assert_eq!(
            manifest.systems[0].bridge_interface().unwrap(),
            "native.live_host.bridge.v1"
        );
    }
}
