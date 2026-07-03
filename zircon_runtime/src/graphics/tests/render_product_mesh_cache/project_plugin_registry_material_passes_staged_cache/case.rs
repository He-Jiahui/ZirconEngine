use crate::asset::AssetUri;
use crate::core::framework::render::ShaderPassType;
use crate::core::resource::ResourceId;

pub(super) fn registry_shader_cases() -> [RegistryShaderCase; 2] {
    [
        RegistryShaderCase {
            locator: "res://project/shaders/project_shader",
            material_locator: "res://materials/project_registry_material_pass.zmaterial",
            revision: 126_198_881_308_539_824,
        },
        RegistryShaderCase {
            locator: "package://native_dynamic_fixture/shaders/shader",
            material_locator: "res://materials/plugin_registry_material_pass.zmaterial",
            revision: 14_843_875_089_575_827_114,
        },
    ]
}

#[derive(Clone, Copy)]
pub(super) struct RegistryShaderCase {
    pub(super) locator: &'static str,
    material_locator: &'static str,
    pub(super) revision: u64,
}

impl RegistryShaderCase {
    pub(super) fn shader_uri(self) -> AssetUri {
        AssetUri::parse(self.locator).expect("registry shader URI")
    }

    pub(super) fn material_uri(self) -> AssetUri {
        AssetUri::parse(self.material_locator).expect("registry material URI")
    }

    pub(super) fn source_label_for_pass(self, pass_type: ShaderPassType) -> String {
        format!("{}::{}", self.locator, pass_type.token())
    }

    pub(super) fn shader_id(self) -> ResourceId {
        ResourceId::from_locator(&self.shader_uri())
    }

    pub(super) fn material_id(self) -> ResourceId {
        ResourceId::from_locator(&self.material_uri())
    }
}
