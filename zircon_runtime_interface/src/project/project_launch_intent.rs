use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize};

use super::{
    validate_project_name, ProjectActivationOperationId, ProjectLaunchIntentError,
    ProjectTemplateId,
};

/// The only currently accepted wire revision for a requested project launch.
/// 项目启动请求当前唯一接受的线协议版本；反序列化会拒绝其他版本。
pub const PROJECT_LAUNCH_INTENT_SCHEMA_VERSION_V1: u32 = 1;

/// Selects the project-derived capabilities that a later preflight composition may admit.
///
/// This is requested intent, not approval. The preflight receipt is the first authority allowed
/// to turn it into a concrete composition plan.
/// 本枚举只记录请求档位；具体策略由预检决定，选中档位本身不构成准入或授权。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectLaunchProfile {
    /// 请求普通启动档；是否满足条件由预检结果决定。
    #[default]
    Normal,
    /// 请求安全启动档；此值本身不包含已批准的能力集合。
    Safe,
    /// 请求恢复启动档；是否进入恢复流程由预检决定。
    Recovery,
}

/// The product surface that originated a launch operation.
/// 可序列化的请求来源记录，不构成 Hub 身份认证或项目准入授权。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectLaunchSource {
    /// Zircon 应用入口发起的请求来源标签。
    Application,
    /// 命令行入口发起的请求来源标签。
    Cli,
    /// Hub 启动入口发起的请求来源标签。
    Hub,
    /// 欢迎界面发起的请求来源标签。
    Welcome,
    /// 最近项目入口发起的请求来源标签。
    Recent,
}

/// A preflight input, deliberately separate from a canonical `ProjectIdentity`.
///
/// A requested path is user-controlled input. Only data-only preflight can resolve it and attach
/// the persisted project GUID plus manifest digest required for canonical identity.
/// 此枚举只携带待预检的打开路径，或项目名称、创建位置与模板；路径尚未解析为身份。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", deny_unknown_fields)]
pub enum ProjectLaunchTarget {
    /// 请求打开路径所指项目；路径是输入文本，后续预检负责解析并核对项目身份。
    OpenExisting { requested_path: PathBuf },
    /// 请求在位置下按模板创建项目；本类型只检查名称和位置的输入形状。
    CreateProject {
        project_name: String,
        location: PathBuf,
        template: ProjectTemplateId,
    },
}

/// Versioned, idempotency-addressable project lifecycle input shared by Hub, App, and Editor.
///
/// It represents a request only. It contains neither `ProjectIdentity` nor an activation permit,
/// and cannot be used as evidence that project-derived code is safe to execute.
/// 字段由构造器和自定义反序列化统一校验；值拥有其路径与名称，不持有文件或预检资源。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProjectLaunchIntent {
    schema_version: u32,
    operation_id: ProjectActivationOperationId,
    source: ProjectLaunchSource,
    profile: ProjectLaunchProfile,
    target: ProjectLaunchTarget,
}

impl ProjectLaunchIntent {
    /// 构造打开现有项目的请求。
    ///
    /// `operation_id` 标识本次可幂等关联的操作，`source` 与 `profile` 记录请求标签，
    /// `requested_path` 是待预检的路径输入。空白或非文本路径返回对应错误；
    /// 本方法不访问文件系统、解析规范根路径或确认项目存在。
    pub fn open_existing(
        operation_id: ProjectActivationOperationId,
        source: ProjectLaunchSource,
        profile: ProjectLaunchProfile,
        requested_path: impl Into<PathBuf>,
    ) -> Result<Self, ProjectLaunchIntentError> {
        Self::new(
            operation_id,
            source,
            profile,
            ProjectLaunchTarget::OpenExisting {
                requested_path: requested_path.into(),
            },
        )
    }

    /// 构造创建项目的请求。
    ///
    /// `project_name` 经项目名称校验，`location` 只要求是非空文本路径，
    /// `template` 记录请求模板；文件系统可写性和模板实际可用性由后续流程检查。
    /// 名称、位置或路径编码无效时返回 [ProjectLaunchIntentError]。
    pub fn create_project(
        operation_id: ProjectActivationOperationId,
        source: ProjectLaunchSource,
        profile: ProjectLaunchProfile,
        project_name: impl Into<String>,
        location: impl Into<PathBuf>,
        template: ProjectTemplateId,
    ) -> Result<Self, ProjectLaunchIntentError> {
        Self::new(
            operation_id,
            source,
            profile,
            ProjectLaunchTarget::CreateProject {
                project_name: project_name.into(),
                location: location.into(),
                template,
            },
        )
    }

    /// 返回构造时写入的协议版本。
    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// 按值返回此请求的操作身份；该身份在 retarget 时保持不变。
    pub const fn operation_id(&self) -> ProjectActivationOperationId {
        self.operation_id
    }

    /// 按值返回发起入口标签；此标签不是认证信息。
    pub const fn source(&self) -> ProjectLaunchSource {
        self.source
    }

    /// 按值返回所请求的预检档位；最终策略以预检结果为准。
    pub const fn profile(&self) -> ProjectLaunchProfile {
        self.profile
    }

    /// 借用目标输入；返回引用的生命周期受 `self` 限制。
    pub fn target(&self) -> &ProjectLaunchTarget {
        &self.target
    }

    /// Keeps the initiating operation and policy when an input has become an existing root.
    /// 创建成功后可沿用原操作标识转为打开请求；调用方须传入实际创建的项目根。
    /// 返回的新值将目标替换为打开请求，并重新执行路径形状校验；不会自行创建或解析该路径。
    pub fn retarget_open_existing_project(
        &self,
        requested_path: impl Into<PathBuf>,
    ) -> Result<Self, ProjectLaunchIntentError> {
        Self::open_existing(self.operation_id, self.source, self.profile, requested_path)
    }

    /// 统一验证目标输入并写入当前协议版本；不执行文件系统准入检查。
    fn new(
        operation_id: ProjectActivationOperationId,
        source: ProjectLaunchSource,
        profile: ProjectLaunchProfile,
        target: ProjectLaunchTarget,
    ) -> Result<Self, ProjectLaunchIntentError> {
        validate_target(&target)?;
        Ok(Self {
            schema_version: PROJECT_LAUNCH_INTENT_SCHEMA_VERSION_V1,
            operation_id,
            source,
            profile,
            target,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
/// 自定义反序列化的内部线结构；未知字段在转换为公开请求前被拒绝。
struct ProjectLaunchIntentWire {
    schema_version: u32,
    operation_id: ProjectActivationOperationId,
    source: ProjectLaunchSource,
    profile: ProjectLaunchProfile,
    target: ProjectLaunchTarget,
}

impl<'de> Deserialize<'de> for ProjectLaunchIntent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ProjectLaunchIntentWire::deserialize(deserializer)?;
        if wire.schema_version != PROJECT_LAUNCH_INTENT_SCHEMA_VERSION_V1 {
            return Err(serde::de::Error::custom(
                ProjectLaunchIntentError::UnsupportedSchemaVersion {
                    expected: PROJECT_LAUNCH_INTENT_SCHEMA_VERSION_V1,
                    actual: wire.schema_version,
                },
            ));
        }
        Self::new(wire.operation_id, wire.source, wire.profile, wire.target)
            .map_err(serde::de::Error::custom)
    }
}

/// 只检查目标内可由值本身判定的名称和路径形状，不解析磁盘状态。
fn validate_target(target: &ProjectLaunchTarget) -> Result<(), ProjectLaunchIntentError> {
    match target {
        ProjectLaunchTarget::OpenExisting { requested_path } => {
            validate_path(requested_path, ProjectLaunchIntentError::EmptyOpenPath)
        }
        ProjectLaunchTarget::CreateProject {
            project_name,
            location,
            ..
        } => {
            validate_project_name(project_name)?;
            validate_path(location, ProjectLaunchIntentError::EmptyCreateLocation)
        }
    }
}

/// 拒绝空白或不能表示为文本的路径；保留其他路径内容，不做规范化或存在性检查。
fn validate_path(
    path: &PathBuf,
    empty_error: ProjectLaunchIntentError,
) -> Result<(), ProjectLaunchIntentError> {
    if path.as_os_str().is_empty() {
        return Err(empty_error);
    }
    let Some(value) = path.to_str() else {
        return Err(ProjectLaunchIntentError::NonTextPath { path: path.clone() });
    };
    if value.trim().is_empty() {
        return Err(empty_error);
    }
    Ok(())
}
