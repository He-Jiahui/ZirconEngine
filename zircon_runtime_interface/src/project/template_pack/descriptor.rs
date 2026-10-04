use std::fmt::Write;
use std::sync::OnceLock;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};

use crate::project::{
    assess_project_engine_compatibility, ProjectEngineCompatibility,
    ProjectEngineCompatibilityError, ProjectEngineVersion,
};
use crate::runtime_build_set::{ZrRuntimeModuleCompositionTargetV1, ZrRuntimeModuleProfileV1};

use super::content_digest::ProjectTemplateContentDigest;
use super::embedded::RENDERABLE_EMPTY_ENTRIES;
use super::ProjectTemplateId;

const RENDERABLE_EMPTY_ENGINE_VERSION_REQ: &str = ">=0.1.0, <0.2.0";
const RENDERABLE_EMPTY_CAPABILITIES: &[ProjectTemplateCapability] = &[
    ProjectTemplateCapability::Scene3d,
    ProjectTemplateCapability::ObjModelImport,
    ProjectTemplateCapability::PbrMaterialImport,
    ProjectTemplateCapability::WgslShaderImport,
    ProjectTemplateCapability::NativeWindow,
];
const RENDERABLE_EMPTY_RUNTIME_PROVIDERS: &[&str] =
    &["rendering", "obj_importer", "shader_wgsl_importer"];

static RENDERABLE_EMPTY_TARGET_REQUIREMENTS: OnceLock<Box<[ProjectTemplateTargetRequirement]>> =
    OnceLock::new();
static RENDERABLE_EMPTY_ENTRY_DESCRIPTORS: OnceLock<Box<[ProjectTemplateEntryDescriptor]>> =
    OnceLock::new();

/// Immutable composition contract for one target produced by a template.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProjectTemplateTargetRequirement {
    target: ZrRuntimeModuleCompositionTargetV1,
    module_profile: ZrRuntimeModuleProfileV1,
    required_capabilities: Box<[ProjectTemplateCapability]>,
    required_runtime_providers: Box<[String]>,
}

impl ProjectTemplateTargetRequirement {
    fn new(
        target: ZrRuntimeModuleCompositionTargetV1,
        module_profile: ZrRuntimeModuleProfileV1,
        required_capabilities: impl IntoIterator<Item = ProjectTemplateCapability>,
        required_runtime_providers: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        Self {
            target,
            module_profile,
            required_capabilities: required_capabilities.into_iter().collect(),
            required_runtime_providers: required_runtime_providers
                .into_iter()
                .map(str::to_string)
                .collect(),
        }
    }

    pub const fn target(&self) -> ZrRuntimeModuleCompositionTargetV1 {
        self.target
    }

    pub const fn module_profile(&self) -> ZrRuntimeModuleProfileV1 {
        self.module_profile
    }

    pub fn required_capabilities(&self) -> &[ProjectTemplateCapability] {
        &self.required_capabilities
    }

    pub fn required_runtime_providers(&self) -> impl Iterator<Item = &str> {
        self.required_runtime_providers.iter().map(String::as_str)
    }
}

/// Exact source-file evidence for one embedded template entry.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectTemplateEntryDescriptor {
    path: String,
    byte_len: u64,
    sha256: String,
}

impl ProjectTemplateEntryDescriptor {
    pub fn path(&self) -> &str {
        &self.path
    }

    pub const fn byte_len(&self) -> u64 {
        self.byte_len
    }

    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

/// Stable product capability required by a template before its project can be marked ready.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectTemplateCapability {
    Scene3d,
    ObjModelImport,
    PbrMaterialImport,
    WgslShaderImport,
    NativeWindow,
}

/// Immutable identity and admission metadata for one packaged project template.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProjectTemplateDescriptor {
    id: ProjectTemplateId,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProjectTemplateDescriptorWire {
    id: ProjectTemplateId,
    version: u32,
    content_digest: ProjectTemplateContentDigest,
    #[serde(default)]
    engine_version_req: Option<String>,
    target_requirements: Vec<ProjectTemplateTargetRequirementWire>,
    entries: Vec<ProjectTemplateEntryDescriptor>,
}

#[derive(Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ProjectTemplateTargetRequirementWire {
    target: ZrRuntimeModuleCompositionTargetV1,
    module_profile: ZrRuntimeModuleProfileV1,
    required_capabilities: Vec<ProjectTemplateCapability>,
    required_runtime_providers: Vec<String>,
}

impl<'de> Deserialize<'de> for ProjectTemplateDescriptor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ProjectTemplateDescriptorWire::deserialize(deserializer)?;
        // 只接受内置注册表给出的完整元数据，不能让持久化副本改写模板能力或内容身份。
        let expected = project_template_descriptor(wire.id);
        let expected_target_requirements = expected
            .target_requirements()
            .iter()
            .map(|requirement| ProjectTemplateTargetRequirementWire {
                target: requirement.target(),
                module_profile: requirement.module_profile(),
                required_capabilities: requirement.required_capabilities().to_vec(),
                required_runtime_providers: requirement
                    .required_runtime_providers()
                    .map(str::to_string)
                    .collect(),
            })
            .collect::<Vec<_>>();
        if wire.version != expected.version()
            || wire.content_digest != expected.content_digest()
            || wire.engine_version_req.as_deref() != expected.engine_version_req()
            || wire.target_requirements != expected_target_requirements
            || wire.entries.as_slice() != expected.entries()
        {
            return Err(serde::de::Error::custom(format_args!(
                "project template descriptor zircon/{}@{} does not match canonical {}",
                wire.id.as_str(),
                wire.version,
                expected.qualified_id()
            )));
        }
        Ok(expected)
    }
}

impl Serialize for ProjectTemplateDescriptor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ProjectTemplateDescriptorWire {
            id: self.id,
            version: self.version(),
            content_digest: self.content_digest(),
            engine_version_req: self.engine_version_req().map(str::to_string),
            target_requirements: self
                .target_requirements()
                .iter()
                .map(|requirement| ProjectTemplateTargetRequirementWire {
                    target: requirement.target(),
                    module_profile: requirement.module_profile(),
                    required_capabilities: requirement.required_capabilities().to_vec(),
                    required_runtime_providers: requirement
                        .required_runtime_providers()
                        .map(str::to_string)
                        .collect(),
                })
                .collect(),
            entries: self.entries().to_vec(),
        }
        .serialize(serializer)
    }
}

impl ProjectTemplateDescriptor {
    pub fn qualified_id(&self) -> String {
        format!("zircon/{}@{}", self.id.as_str(), self.version())
    }

    pub const fn id(&self) -> ProjectTemplateId {
        self.id
    }

    pub const fn version(&self) -> u32 {
        match self.id {
            ProjectTemplateId::RenderableEmpty => 1,
        }
    }

    pub fn content_digest(&self) -> ProjectTemplateContentDigest {
        match self.id {
            ProjectTemplateId::RenderableEmpty => {
                *RENDERABLE_EMPTY_CONTENT_DIGEST.get_or_init(|| {
                    ProjectTemplateContentDigest::from_entries(
                        RENDERABLE_EMPTY_ENTRIES
                            .iter()
                            .map(|entry| (entry.path, entry.bytes)),
                    )
                })
            }
        }
    }

    pub const fn engine_version_req(&self) -> Option<&'static str> {
        match self.id {
            ProjectTemplateId::RenderableEmpty => Some(RENDERABLE_EMPTY_ENGINE_VERSION_REQ),
        }
    }

    pub fn target_requirements(&self) -> &'static [ProjectTemplateTargetRequirement] {
        match self.id {
            ProjectTemplateId::RenderableEmpty => {
                RENDERABLE_EMPTY_TARGET_REQUIREMENTS.get_or_init(|| {
                    [
                        ProjectTemplateTargetRequirement::new(
                            ZrRuntimeModuleCompositionTargetV1::EditorHost,
                            ZrRuntimeModuleProfileV1::Editor,
                            RENDERABLE_EMPTY_CAPABILITIES.iter().copied(),
                            RENDERABLE_EMPTY_RUNTIME_PROVIDERS.iter().copied(),
                        ),
                        ProjectTemplateTargetRequirement::new(
                            ZrRuntimeModuleCompositionTargetV1::ClientRuntime,
                            ZrRuntimeModuleProfileV1::Client3d,
                            RENDERABLE_EMPTY_CAPABILITIES.iter().copied(),
                            RENDERABLE_EMPTY_RUNTIME_PROVIDERS.iter().copied(),
                        ),
                    ]
                    .into()
                })
            }
        }
    }

    pub fn entries(&self) -> &'static [ProjectTemplateEntryDescriptor] {
        match self.id {
            ProjectTemplateId::RenderableEmpty => {
                RENDERABLE_EMPTY_ENTRY_DESCRIPTORS.get_or_init(|| {
                    let mut entries = RENDERABLE_EMPTY_ENTRIES
                        .iter()
                        .map(|entry| ProjectTemplateEntryDescriptor {
                            path: entry.path.to_string(),
                            byte_len: entry.bytes.len() as u64,
                            sha256: sha256_hex(entry.bytes),
                        })
                        .collect::<Vec<_>>();
                    entries.sort_unstable_by(|left, right| left.path.cmp(&right.path));
                    entries.into_boxed_slice()
                })
            }
        }
    }

    /// Evaluates the descriptor's admission range without loading project code or assets.
    pub fn assess_engine_compatibility(
        &self,
        running_engine: &ProjectEngineVersion,
    ) -> Result<ProjectEngineCompatibility, ProjectEngineCompatibilityError> {
        assess_project_engine_compatibility(self.engine_version_req(), running_engine)
    }
}

/// 按稳定 ID 取得内置模板的规范描述；版本、摘要和准入要求均由本 crate 计算。
pub fn project_template_descriptor(id: ProjectTemplateId) -> ProjectTemplateDescriptor {
    ProjectTemplateDescriptor { id }
}

static RENDERABLE_EMPTY_CONTENT_DIGEST: OnceLock<ProjectTemplateContentDigest> = OnceLock::new();

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String cannot fail");
    }
    encoded
}
