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
// 模板的能力/provider 声明与静态嵌入文件清单共同定义此版本的准入元数据。
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
///
/// 表示一个模板目标及其模块配置、产品能力与运行时 provider 要求；描述符按目标分别暴露该契约。
///
/// # 约束
/// 实例由规范模板描述符构造，调用方只能读取要求，不能修改或替换其中的能力集合。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProjectTemplateTargetRequirement {
    /// 该要求适用的运行时组合目标。
    target: ZrRuntimeModuleCompositionTargetV1,
    /// 目标所需的运行时模块配置。
    module_profile: ZrRuntimeModuleProfileV1,
    /// 项目创建后必须具备的产品能力。
    required_capabilities: Box<[ProjectTemplateCapability]>,
    /// 目标选择器必须提供的 provider 稳定 ID。
    required_runtime_providers: Box<[String]>,
}

impl ProjectTemplateTargetRequirement {
    // 将静态模板声明复制为不可变拥有值，供 OnceLock 缓存并跨调用借用。
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

    /// 返回此要求适用的运行时组合目标。
    ///
    /// # 返回值
    /// 返回按值复制的目标枚举。
    pub const fn target(&self) -> ZrRuntimeModuleCompositionTargetV1 {
        self.target
    }

    /// 返回该目标所需的运行时模块配置。
    ///
    /// # 返回值
    /// 返回按值复制的模块配置枚举。
    pub const fn module_profile(&self) -> ZrRuntimeModuleProfileV1 {
        self.module_profile
    }

    /// 借用该目标要求的产品能力集合。
    ///
    /// # 返回值
    /// 返回借用的能力切片，其生命周期受此要求对象约束。
    pub fn required_capabilities(&self) -> &[ProjectTemplateCapability] {
        &self.required_capabilities
    }

    /// 借用该目标要求的稳定 provider ID。
    ///
    /// # 返回值
    /// 返回仅在此要求对象存活期间有效的迭代器；每项均借用缓存中的字符串。
    pub fn required_runtime_providers(&self) -> impl Iterator<Item = &str> {
        self.required_runtime_providers.iter().map(String::as_str)
    }
}

/// Exact source-file evidence for one embedded template entry.
///
/// 描述嵌入包中一个原始文件的路径、字节长度与 SHA-256 十六进制摘要，供清单校验与审计使用。
///
/// # 约束
/// [`ProjectTemplateDescriptor::entries`] 返回的字段来自嵌入清单；完整模板描述符的
/// 反序列化会与规范注册表逐项比较。单独反序列化本类型只恢复字段，不验证路径形态、
/// 摘要格式或源文件真实性。
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectTemplateEntryDescriptor {
    /// 模板包中的项目相对路径。
    path: String,
    /// 原始嵌入文件的字节长度。
    byte_len: u64,
    /// 原始嵌入文件的 SHA-256 小写十六进制摘要。
    sha256: String,
}

impl ProjectTemplateEntryDescriptor {
    /// 返回项目内的模板相对路径。
    ///
    /// # 返回值
    /// 返回借用的路径文本；此描述符不提供文件系统访问能力。
    pub fn path(&self) -> &str {
        &self.path
    }

    /// 返回嵌入源文件的原始字节数。
    ///
    /// # 返回值
    /// 返回渲染前静态资源的长度；身份字段改写后的清单长度可能不同。
    pub const fn byte_len(&self) -> u64 {
        self.byte_len
    }

    /// 返回嵌入源文件的 SHA-256 十六进制摘要。
    ///
    /// # 返回值
    /// 返回借用的摘要文本；该摘要标识原始模板包文件，而不是渲染后的字节。
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

/// Stable product capability required by a template before its project can be marked ready.
///
/// 供模板目标描述声明产品能力；它是准入元数据，不代表运行时 provider 已加载或项目已经就绪。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectTemplateCapability {
    /// 模板需要提供三维场景能力。
    Scene3d,
    /// 模板需要导入 OBJ 模型的能力。
    ObjModelImport,
    /// 模板需要导入 PBR 材质的能力。
    PbrMaterialImport,
    /// 模板需要导入 WGSL 着色器的能力。
    WgslShaderImport,
    /// 模板需要原生窗口宿主能力。
    NativeWindow,
}

/// Immutable identity and admission metadata for one packaged project template.
///
/// 此轻量值仅保存内置模板稳定 ID；版本、内容摘要、目标要求与文件清单都从本 crate 的规范注册表派生。
///
/// # 生命周期
/// 描述符可复制并持久化为 wire 表示；反序列化时会拒绝与当前规范注册表不一致的元数据。
/// 它不持有模板字节，也不授予项目激活权限。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProjectTemplateDescriptor {
    /// 唯一的规范注册表键。
    id: ProjectTemplateId,
}

// Wire DTO 只承载序列化字段；公开描述符仍以稳定 ID 为唯一内存身份。
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProjectTemplateDescriptorWire {
    /// 用于重新查找规范内置描述符的稳定键。
    id: ProjectTemplateId,
    /// 持久化描述符声称的模板包版本。
    version: u32,
    /// 持久化描述符声称的原始嵌入内容摘要。
    content_digest: ProjectTemplateContentDigest,
    /// 可选的创建引擎 semver 范围。
    #[serde(default)]
    engine_version_req: Option<String>,
    /// 持久化描述符声称的逐目标准入契约。
    target_requirements: Vec<ProjectTemplateTargetRequirementWire>,
    /// 持久化描述符声称的逐文件证据。
    entries: Vec<ProjectTemplateEntryDescriptor>,
}

// 比较时使用具备 Eq 的 wire 形态，避免把不可信数据转换为可变的公开描述符。
#[derive(Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ProjectTemplateTargetRequirementWire {
    /// 该项声明适用的运行时组合目标。
    target: ZrRuntimeModuleCompositionTargetV1,
    /// 该目标所需的运行时模块配置。
    module_profile: ZrRuntimeModuleProfileV1,
    /// 该目标声称需要的产品能力集合。
    required_capabilities: Vec<ProjectTemplateCapability>,
    /// 该目标声称需要的 runtime provider 稳定 ID。
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
        // 按规范缓存值重建 wire 形态，顺序与字段都纳入持久化一致性比较。
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
        // 仅从 ID 派生 wire 字段，避免公开结构携带与注册表脱离的副本状态。
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
    /// 返回包含注册表命名空间、稳定 ID 与版本的可读身份。
    ///
    /// # 返回值
    /// 返回新分配的 `zircon/<id>@<version>` 字符串，适合日志和持久化引用。
    pub fn qualified_id(&self) -> String {
        format!("zircon/{}@{}", self.id.as_str(), self.version())
    }

    /// 返回此规范描述符的稳定模板 ID。
    ///
    /// # 返回值
    /// 返回按值复制的注册表键。
    pub const fn id(&self) -> ProjectTemplateId {
        self.id
    }

    /// 返回此模板 ID 对应的规范包版本。
    ///
    /// # 返回值
    /// 返回由注册表分支确定的版本号，而非从调用者提供的文件中读取。
    pub const fn version(&self) -> u32 {
        match self.id {
            ProjectTemplateId::RenderableEmpty => 1,
        }
    }

    /// 返回根据编译期嵌入文件清单惰性计算的内容摘要。
    ///
    /// # 返回值
    /// 返回可复制摘要值；首次调用会初始化并缓存本模板的摘要，之后复用该值。
    pub fn content_digest(&self) -> ProjectTemplateContentDigest {
        match self.id {
            // 摘要身份覆盖静态嵌入项；渲染时改写清单不会改变此包身份。
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

    /// 返回模板声明的创建引擎 semver 要求。
    ///
    /// # 返回值
    /// 返回静态注册表字符串；`None` 表示此模板没有版本范围要求。
    pub const fn engine_version_req(&self) -> Option<&'static str> {
        match self.id {
            ProjectTemplateId::RenderableEmpty => Some(RENDERABLE_EMPTY_ENGINE_VERSION_REQ),
        }
    }

    /// 返回模板对每个运行时目标的不可变要求。
    ///
    /// # 返回值
    /// 返回静态借用的目标要求切片；首次调用会惰性构造并缓存规范要求。
    ///
    /// # 约束
    /// 返回值是描述性准入输入，调用方仍须结合实际 provider 与宿主策略作出准入决定。
    pub fn target_requirements(&self) -> &'static [ProjectTemplateTargetRequirement] {
        match self.id {
            // 两个产品目标共享模板 capability/provider 要求，但保留各自 target/profile 配对。
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

    /// 返回嵌入包中各文件的规范身份元数据。
    ///
    /// # 返回值
    /// 返回按项目相对路径排序并惰性缓存的静态切片；长度与摘要针对原始嵌入字节。
    pub fn entries(&self) -> &'static [ProjectTemplateEntryDescriptor] {
        match self.id {
            // 派生原始嵌入字节的长度和 SHA-256 后按相对路径排序以稳定 wire 表示。
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
    ///
    /// 只比较模板声明的引擎版本要求与给定运行版本，不读取项目内容或验证其他目标能力。
    ///
    /// # 参数
    /// - `running_engine`：执行创建流程的引擎版本。
    ///
    /// # 返回值
    /// 返回版本范围评估结果；无法解析模板要求时返回对应兼容性错误。
    ///
    /// # 使用时机与生命周期
    /// 创建 receipt 时由 [`super::ProjectTemplateReceipt::try_new`] 使用；兼容结果本身不授予激活权限。
    pub fn assess_engine_compatibility(
        &self,
        running_engine: &ProjectEngineVersion,
    ) -> Result<ProjectEngineCompatibility, ProjectEngineCompatibilityError> {
        assess_project_engine_compatibility(self.engine_version_req(), running_engine)
    }
}

/// 按稳定 ID 取得内置模板的规范描述；版本、摘要和准入要求均由本 crate 计算。
///
/// # 参数
/// - `id`：内置模板注册表中的稳定键。
///
/// # 返回值
/// 返回轻量规范句柄；不会复制或加载模板文件字节。
pub fn project_template_descriptor(id: ProjectTemplateId) -> ProjectTemplateDescriptor {
    ProjectTemplateDescriptor { id }
}

static RENDERABLE_EMPTY_CONTENT_DIGEST: OnceLock<ProjectTemplateContentDigest> = OnceLock::new();

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(64);
    // 以固定小写十六进制编码，保证相同原始字节得到稳定的 descriptor 字段。
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String cannot fail");
    }
    encoded
}
