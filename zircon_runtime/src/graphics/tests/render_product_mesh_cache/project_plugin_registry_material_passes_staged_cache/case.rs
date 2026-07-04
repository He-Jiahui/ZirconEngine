use crate::asset::AssetUri;
use crate::core::framework::render::ShaderPassType;
use crate::core::resource::{ResourceId, ResourceKind, ResourceRecord, ResourceState};

const PROJECT_SHADER_LOCATOR: &str = "res://project/shaders/project_shader";
const PROJECT_MATERIAL_LOCATOR: &str = "res://materials/project_registry_material_pass.zmaterial";
const PLUGIN_SHADER_LOCATOR: &str = "package://native_dynamic_fixture/shaders/shader";
const PLUGIN_MATERIAL_LOCATOR: &str = "res://materials/plugin_registry_material_pass.zmaterial";

pub(super) fn registry_shader_cases() -> [RegistryShaderCase; 2] {
    [
        RegistryShaderCase {
            locator: PROJECT_SHADER_LOCATOR,
            material_locator: PROJECT_MATERIAL_LOCATOR,
            revision: 126_198_881_308_539_824,
        },
        RegistryShaderCase {
            locator: PLUGIN_SHADER_LOCATOR,
            material_locator: PLUGIN_MATERIAL_LOCATOR,
            revision: 14_843_875_089_575_827_114,
        },
    ]
}

pub(super) fn registry_shader_cases_from_live_records(
    records: &[ResourceRecord],
) -> Vec<RegistryShaderCase> {
    let mut cases = records
        .iter()
        .filter_map(registry_shader_case_from_live_record)
        .collect::<Vec<_>>();
    cases.sort_by(|left, right| left.locator.cmp(right.locator));
    cases
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

fn registry_shader_case_from_live_record(record: &ResourceRecord) -> Option<RegistryShaderCase> {
    if record.kind != ResourceKind::Shader
        || record.state != ResourceState::Ready
        || record.revision == 0
    {
        return None;
    }

    match record.primary_locator.to_string().as_str() {
        PROJECT_SHADER_LOCATOR => Some(RegistryShaderCase {
            locator: PROJECT_SHADER_LOCATOR,
            material_locator: PROJECT_MATERIAL_LOCATOR,
            revision: record.revision,
        }),
        PLUGIN_SHADER_LOCATOR => Some(RegistryShaderCase {
            locator: PLUGIN_SHADER_LOCATOR,
            material_locator: PLUGIN_MATERIAL_LOCATOR,
            revision: record.revision,
        }),
        _ => None,
    }
}
