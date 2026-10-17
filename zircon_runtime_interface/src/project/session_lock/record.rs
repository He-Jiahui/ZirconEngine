use crate::project::{ProjectActivationOperationId, ProjectLaunchSource};
use crate::runtime_build_set::ZrRuntimeBuildSetId;

use super::ProjectSessionAdmissionRecordError;

/// Maximum UTF-8 byte length of the persisted session instance identity.
pub const MAX_PROJECT_SESSION_ADMISSION_INSTANCE_ID_BYTES: usize = 128;

/// Origin of a local desktop request. This is provenance, not an authentication claim.
/// 表示本机桌面请求的入口标签；序列化文本由 session-lock codec 固定映射，不作为身份凭证。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectSessionPrincipalV1 {
    /// 应用入口。
    Application,
    /// 命令行入口。
    Cli,
    /// Hub 入口。
    Hub,
    /// 欢迎界面入口。
    Welcome,
    /// 最近项目入口。
    Recent,
}

impl ProjectSessionPrincipalV1 {
    /// 将启动请求的来源枚举映射为持久记录中的 principal 标签。
    pub const fn from_launch_source(source: ProjectLaunchSource) -> Self {
        match source {
            ProjectLaunchSource::Application => Self::Application,
            ProjectLaunchSource::Cli => Self::Cli,
            ProjectLaunchSource::Hub => Self::Hub,
            ProjectLaunchSource::Welcome => Self::Welcome,
            ProjectLaunchSource::Recent => Self::Recent,
        }
    }

    /// 返回 codec 使用的稳定小写字段值。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Application => "application",
            Self::Cli => "cli",
            Self::Hub => "hub",
            Self::Welcome => "welcome",
            Self::Recent => "recent",
        }
    }

    /// 从稳定小写字段值解析标签；未知值返回记录错误。
    pub fn parse(value: &str) -> Result<Self, ProjectSessionAdmissionRecordError> {
        match value {
            "application" => Ok(Self::Application),
            "cli" => Ok(Self::Cli),
            "hub" => Ok(Self::Hub),
            "welcome" => Ok(Self::Welcome),
            "recent" => Ok(Self::Recent),
            _ => Err(ProjectSessionAdmissionRecordError::new(
                "invalid or missing principal",
            )),
        }
    }
}

/// 持久准入阶段；Ready 只能通过提交 generation 到达，关闭或恢复状态不能重新激活。
/// Persistent lifecycle state of an editor admission lease.
/// codec 将这些阶段编码为固定 snake_case 文本；允许的普通迁移边由 `transition_to` 校验。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectSessionAdmissionLifecycleV1 {
    /// 已建立准入记录，尚未记录预检批准。
    Claimed,
    /// 预检阶段已记录批准，尚未进入激活。
    PreflightApproved,
    /// 正在执行激活；此阶段可通过 `commit_ready` 提交会话代次。
    Activating,
    /// 已提交非零会话代次，可作为已就绪记录读取。
    Ready,
    /// 正在关闭；不能退回预检或激活阶段。
    Closing,
    /// 关闭或恢复仍需处理；消费者不能把此记录当作可就绪会话。
    RecoveryRequired,
}

impl ProjectSessionAdmissionLifecycleV1 {
    /// 返回持久 codec 使用的稳定小写字段值。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Claimed => "claimed",
            Self::PreflightApproved => "preflight_approved",
            Self::Activating => "activating",
            Self::Ready => "ready",
            Self::Closing => "closing",
            Self::RecoveryRequired => "recovery_required",
        }
    }

    /// 从 codec 字段值解析已知阶段；未知或拼写不同的值均拒绝。
    pub fn parse(value: &str) -> Result<Self, ProjectSessionAdmissionRecordError> {
        match value {
            "claimed" => Ok(Self::Claimed),
            "preflight_approved" => Ok(Self::PreflightApproved),
            "activating" => Ok(Self::Activating),
            "ready" => Ok(Self::Ready),
            "closing" => Ok(Self::Closing),
            "recovery_required" => Ok(Self::RecoveryRequired),
            _ => Err(ProjectSessionAdmissionRecordError::new(
                "invalid or missing admission lifecycle",
            )),
        }
    }

    /// 普通状态边不含 Ready；只有实际运行时会话取得 generation 后才能单独提交。
    const fn allows_transition_to(self, next: Self) -> bool {
        match (self, next) {
            (Self::Claimed, Self::PreflightApproved | Self::Closing | Self::RecoveryRequired)
            | (
                Self::PreflightApproved,
                Self::Activating | Self::Closing | Self::RecoveryRequired,
            )
            | (Self::Activating, Self::Closing | Self::RecoveryRequired)
            | (Self::Ready, Self::Closing | Self::RecoveryRequired)
            | (Self::Closing, Self::RecoveryRequired) => true,
            _ => false,
        }
    }
}

/// A non-zero generation committed only after activation becomes ready.
/// 非零会话代次；`None` 表示尚未提交 Ready，不代表平台租约编号。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProjectSessionGenerationV1(u64);

impl ProjectSessionGenerationV1 {
    /// 将非零整数包装为 generation；零返回 `None`。
    pub const fn new(value: u64) -> Option<Self> {
        match value {
            0 => None,
            _ => Some(Self(value)),
        }
    }

    /// 取回构造时保存的非零整数。
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Persistent, cross-process state of the editor instance holding a project admission lease.
///
/// The record never substitutes for the operating-system lease. Consumers must require both the
/// OS lease and `Ready` lifecycle before treating a session as focusable or interactive.
/// 此值拥有跨进程字段；生命周期、epoch 和 generation 的组合由构造器校验，
/// 实际互斥租约仍由平台宿主持有。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectSessionAdmissionRecordV1 {
    process_id: u32,
    instance_id: String,
    principal: ProjectSessionPrincipalV1,
    build_set_id: ZrRuntimeBuildSetId,
    operation_id: ProjectActivationOperationId,
    lifecycle: ProjectSessionAdmissionLifecycleV1,
    checked_epoch: u64,
    session_generation: Option<ProjectSessionGenerationV1>,
    heartbeat_unix_millis: u64,
}

impl ProjectSessionAdmissionRecordV1 {
    /// 创建初始 Claimed 快照（epoch 1、无 generation）；平台租约和写盘由宿主负责。
    /// `instance_id` 必须是有界 ASCII 数字/连字符文本；其余身份和心跳值按值保存。
    pub fn claim(
        process_id: u32,
        instance_id: impl Into<String>,
        principal: ProjectSessionPrincipalV1,
        build_set_id: ZrRuntimeBuildSetId,
        operation_id: ProjectActivationOperationId,
        heartbeat_unix_millis: u64,
    ) -> Result<Self, ProjectSessionAdmissionRecordError> {
        Self::from_persisted(
            process_id,
            instance_id,
            principal,
            build_set_id,
            operation_id,
            ProjectSessionAdmissionLifecycleV1::Claimed,
            1,
            None,
            heartbeat_unix_millis,
        )
    }

    // EXEMPT(GEN-Q7): persisted session-lock construction keeps the fixed wire-record argument shape.
    #[allow(clippy::too_many_arguments)]
    // 同时用于解码持久记录和构造状态后继，集中校验跨字段不变量。
    /// 构造或恢复持久记录，并检查实例令牌、非零 epoch 与 lifecycle/generation 组合。
    /// 此入口不获取平台租约，也不校验心跳的新旧顺序。
    pub(crate) fn from_persisted(
        process_id: u32,
        instance_id: impl Into<String>,
        principal: ProjectSessionPrincipalV1,
        build_set_id: ZrRuntimeBuildSetId,
        operation_id: ProjectActivationOperationId,
        lifecycle: ProjectSessionAdmissionLifecycleV1,
        checked_epoch: u64,
        session_generation: Option<ProjectSessionGenerationV1>,
        heartbeat_unix_millis: u64,
    ) -> Result<Self, ProjectSessionAdmissionRecordError> {
        let instance_id = instance_id.into();
        validate_instance_id(&instance_id)?;
        if checked_epoch == 0 {
            return Err(ProjectSessionAdmissionRecordError::new(
                "checked_epoch must be non-zero",
            ));
        }
        if matches!(lifecycle, ProjectSessionAdmissionLifecycleV1::Ready)
            && session_generation.is_none()
        {
            return Err(ProjectSessionAdmissionRecordError::new(
                "ready admission record requires a committed session_generation",
            ));
        }
        if !matches!(
            lifecycle,
            ProjectSessionAdmissionLifecycleV1::Ready
                | ProjectSessionAdmissionLifecycleV1::Closing
                | ProjectSessionAdmissionLifecycleV1::RecoveryRequired
        ) && session_generation.is_some()
        {
            return Err(ProjectSessionAdmissionRecordError::new(
                "uncommitted admission lifecycle cannot carry a session_generation",
            ));
        }
        Ok(Self {
            process_id,
            instance_id,
            principal,
            build_set_id,
            operation_id,
            lifecycle,
            checked_epoch,
            session_generation,
            heartbeat_unix_millis,
        })
    }

    /// 返回记录所属进程的进程号。
    pub const fn process_id(&self) -> u32 {
        self.process_id
    }

    /// 借用持久实例令牌；返回值的生命周期受记录本身限制。
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// 返回记录的请求入口标签；它不是身份认证结果。
    pub const fn principal(&self) -> ProjectSessionPrincipalV1 {
        self.principal
    }

    /// 借用记录中的 Runtime 构建集合标识。
    pub fn build_set_id(&self) -> &ZrRuntimeBuildSetId {
        &self.build_set_id
    }

    /// 返回关联到此准入尝试的操作身份。
    pub const fn operation_id(&self) -> ProjectActivationOperationId {
        self.operation_id
    }

    /// 返回持久化的准入生命周期阶段。
    pub const fn lifecycle(&self) -> ProjectSessionAdmissionLifecycleV1 {
        self.lifecycle
    }

    /// 返回状态变更 epoch；它不等同于会话 generation。
    pub const fn checked_epoch(&self) -> u64 {
        self.checked_epoch
    }

    /// 返回已提交的非零 generation；Ready 之外的阶段可保留或不携带它。
    pub const fn session_generation(&self) -> Option<ProjectSessionGenerationV1> {
        self.session_generation
    }

    /// 返回记录保存的 Unix 毫秒心跳值；该访问器不判断其时效。
    pub const fn heartbeat_unix_millis(&self) -> u64 {
        self.heartbeat_unix_millis
    }

    /// 生成合法的非 Ready 状态后继并递增 epoch；Ready 必须走 commit_ready。
    /// 只允许枚举定义的状态边，递增溢出返回记录错误；本方法不写入磁盘。
    pub fn transition_to(
        &self,
        lifecycle: ProjectSessionAdmissionLifecycleV1,
    ) -> Result<Self, ProjectSessionAdmissionRecordError> {
        if matches!(lifecycle, ProjectSessionAdmissionLifecycleV1::Ready) {
            return Err(ProjectSessionAdmissionRecordError::new(
                "ready admission lifecycle requires an explicit generation commit",
            ));
        }
        if !self.lifecycle.allows_transition_to(lifecycle) {
            return Err(ProjectSessionAdmissionRecordError::new(format!(
                "invalid admission lifecycle transition {} -> {}",
                self.lifecycle.as_str(),
                lifecycle.as_str()
            )));
        }
        self.with_lifecycle(lifecycle, self.session_generation)
    }

    /// 仅从 Activating 提交真实会话 generation，之后 Hub 才能将记录作为可寻址会话。
    /// 成功返回 epoch 加一的 Ready 快照；非 Activating 状态或 epoch 溢出会失败。
    pub fn commit_ready(
        &self,
        generation: ProjectSessionGenerationV1,
    ) -> Result<Self, ProjectSessionAdmissionRecordError> {
        if self.lifecycle != ProjectSessionAdmissionLifecycleV1::Activating {
            return Err(ProjectSessionAdmissionRecordError::new(
                "only an activating admission record can commit ready",
            ));
        }
        self.with_lifecycle(ProjectSessionAdmissionLifecycleV1::Ready, Some(generation))
    }

    /// 更新存活时间而不改变 admission epoch；Editor 持锁期间用它刷新持久心跳。
    /// 返回克隆后的记录并直接替换时间值；不递增 epoch，也不强制时间单调递增。
    pub fn with_heartbeat_unix_millis(&self, heartbeat_unix_millis: u64) -> Self {
        Self {
            process_id: self.process_id,
            instance_id: self.instance_id.clone(),
            principal: self.principal,
            build_set_id: self.build_set_id.clone(),
            operation_id: self.operation_id,
            lifecycle: self.lifecycle,
            checked_epoch: self.checked_epoch,
            session_generation: self.session_generation,
            heartbeat_unix_millis,
        }
    }

    fn with_lifecycle(
        &self,
        lifecycle: ProjectSessionAdmissionLifecycleV1,
        session_generation: Option<ProjectSessionGenerationV1>,
    ) -> Result<Self, ProjectSessionAdmissionRecordError> {
        let checked_epoch = self
            .checked_epoch
            .checked_add(1)
            .ok_or_else(|| ProjectSessionAdmissionRecordError::new("checked_epoch overflow"))?;
        Self::from_persisted(
            self.process_id,
            self.instance_id.clone(),
            self.principal,
            self.build_set_id.clone(),
            self.operation_id,
            lifecycle,
            checked_epoch,
            session_generation,
            self.heartbeat_unix_millis,
        )
    }
}

/// 持久实例令牌由 ASCII 数字与连字符组成，因而 128 字节上限也是稳定的字符上限。
/// 同时拒绝空串；此校验不解析 UUID 语义或规范化令牌。
pub(super) fn validate_instance_id(
    instance_id: &str,
) -> Result<(), ProjectSessionAdmissionRecordError> {
    if instance_id.len() > MAX_PROJECT_SESSION_ADMISSION_INSTANCE_ID_BYTES {
        return Err(ProjectSessionAdmissionRecordError::new(format!(
            "instance_id exceeds the {MAX_PROJECT_SESSION_ADMISSION_INSTANCE_ID_BYTES}-byte limit"
        )));
    }
    if instance_id.is_empty()
        || !instance_id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'-')
    {
        return Err(ProjectSessionAdmissionRecordError::new(
            "invalid or missing instance_id",
        ));
    }
    Ok(())
}
