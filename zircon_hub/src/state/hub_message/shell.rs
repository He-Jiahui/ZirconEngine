use crate::settings::HubLanguage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellMessageId {
    PayloadRequiredForAction,
    InvalidPayloadForAction,
    UnknownRecentProjectTarget,
    OpenedPath,
    HubReady,
    RefreshingCatalogs,
    NoRecoveryRequired,
    ReviewActionTarget,
    ReviewActionPayload,
    StateRefreshAfterCommand,
    CheckActionTarget,
    CheckConfigPath,
    BackgroundTaskPanicked,
    VisualVerificationError,
    VisualVerificationWarning,
    VisualVerificationSuccess,
    ReviewSettingsBeforeContinuing,
    CheckHighlightedWorkflowTarget,
}

impl ShellMessageId {
    pub const ALL: &'static [Self] = &[
        Self::PayloadRequiredForAction,
        Self::InvalidPayloadForAction,
        Self::UnknownRecentProjectTarget,
        Self::OpenedPath,
        Self::HubReady,
        Self::RefreshingCatalogs,
        Self::NoRecoveryRequired,
        Self::ReviewActionTarget,
        Self::ReviewActionPayload,
        Self::StateRefreshAfterCommand,
        Self::CheckActionTarget,
        Self::CheckConfigPath,
        Self::BackgroundTaskPanicked,
        Self::VisualVerificationError,
        Self::VisualVerificationWarning,
        Self::VisualVerificationSuccess,
        Self::ReviewSettingsBeforeContinuing,
        Self::CheckHighlightedWorkflowTarget,
    ];

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::PayloadRequiredForAction => "shell.payload-required-for-action",
            Self::InvalidPayloadForAction => "shell.invalid-payload-for-action",
            Self::UnknownRecentProjectTarget => "shell.unknown-recent-project-target",
            Self::OpenedPath => "shell.opened-path",
            Self::HubReady => "shell.hub-ready",
            Self::RefreshingCatalogs => "shell.refreshing-catalogs",
            Self::NoRecoveryRequired => "shell.no-recovery-required",
            Self::ReviewActionTarget => "shell.review-action-target",
            Self::ReviewActionPayload => "shell.review-action-payload",
            Self::StateRefreshAfterCommand => "shell.state-refresh-after-command",
            Self::CheckActionTarget => "shell.check-action-target",
            Self::CheckConfigPath => "shell.check-config-path",
            Self::BackgroundTaskPanicked => "shell.background-task-panicked",
            Self::VisualVerificationError => "shell.visual-verification-error",
            Self::VisualVerificationWarning => "shell.visual-verification-warning",
            Self::VisualVerificationSuccess => "shell.visual-verification-success",
            Self::ReviewSettingsBeforeContinuing => "shell.review-settings-before-continuing",
            Self::CheckHighlightedWorkflowTarget => "shell.check-highlighted-workflow-target",
        }
    }

    pub(super) fn param_count(self) -> usize {
        match self {
            Self::PayloadRequiredForAction | Self::OpenedPath | Self::BackgroundTaskPanicked => 1,
            Self::InvalidPayloadForAction | Self::UnknownRecentProjectTarget => 2,
            _ => 0,
        }
    }

    pub(super) fn template(self, language: HubLanguage) -> &'static str {
        match (language, self) {
            (HubLanguage::English, Self::PayloadRequiredForAction) => {
                "Payload is required for Hub action: {0}"
            }
            (HubLanguage::Chinese, Self::PayloadRequiredForAction) => "Hub 操作缺少 payload：{0}",
            (HubLanguage::English, Self::InvalidPayloadForAction) => {
                "Invalid payload for Hub action {0}: {1}"
            }
            (HubLanguage::Chinese, Self::InvalidPayloadForAction) => {
                "Hub 操作 payload 无效（{0}）：{1}"
            }
            (HubLanguage::English, Self::UnknownRecentProjectTarget) => {
                "Unknown recent project target for {0}: {1}"
            }
            (HubLanguage::Chinese, Self::UnknownRecentProjectTarget) => {
                "未知最近项目目标（{0}）：{1}"
            }
            (HubLanguage::English, Self::OpenedPath) => "Opened {0}",
            (HubLanguage::Chinese, Self::OpenedPath) => "已打开 {0}",
            (HubLanguage::English, Self::HubReady) => "Hub is ready",
            (HubLanguage::Chinese, Self::HubReady) => "Hub 已就绪",
            (HubLanguage::English, Self::RefreshingCatalogs) => {
                "Refreshing projects, source engines, and build workflows"
            }
            (HubLanguage::Chinese, Self::RefreshingCatalogs) => {
                "正在刷新项目、源码引擎和构建工作流"
            }
            (HubLanguage::English, Self::NoRecoveryRequired) => "No recovery action is required",
            (HubLanguage::Chinese, Self::NoRecoveryRequired) => "无需恢复操作",
            (HubLanguage::English, Self::ReviewActionTarget) => {
                "Review the action target and retry from Hub"
            }
            (HubLanguage::Chinese, Self::ReviewActionTarget) => "检查操作目标后从 Hub 重试",
            (HubLanguage::English, Self::ReviewActionPayload) => {
                "Review the action payload and retry from Hub"
            }
            (HubLanguage::Chinese, Self::ReviewActionPayload) => "检查操作 payload 后从 Hub 重试",
            (HubLanguage::English, Self::StateRefreshAfterCommand) => {
                "State will still refresh after each command completes"
            }
            (HubLanguage::Chinese, Self::StateRefreshAfterCommand) => "命令完成后仍会刷新状态",
            (HubLanguage::English, Self::CheckActionTarget) => {
                "Check the action target and try again"
            }
            (HubLanguage::Chinese, Self::CheckActionTarget) => "检查操作目标后重试",
            (HubLanguage::English, Self::CheckConfigPath) => {
                "Check the Hub config path and retry the action"
            }
            (HubLanguage::Chinese, Self::CheckConfigPath) => "检查 Hub 配置路径后重试操作",
            (HubLanguage::English, Self::BackgroundTaskPanicked) => "Background task panicked: {0}",
            (HubLanguage::Chinese, Self::BackgroundTaskPanicked) => "后台任务已中止：{0}",
            (HubLanguage::English, Self::VisualVerificationError) => {
                "Visual verification error state"
            }
            (HubLanguage::Chinese, Self::VisualVerificationError) => "视觉验证错误状态",
            (HubLanguage::English, Self::VisualVerificationWarning) => {
                "Visual verification warning state"
            }
            (HubLanguage::Chinese, Self::VisualVerificationWarning) => "视觉验证警告状态",
            (HubLanguage::English, Self::VisualVerificationSuccess) => {
                "Visual verification success state"
            }
            (HubLanguage::Chinese, Self::VisualVerificationSuccess) => "视觉验证成功状态",
            (HubLanguage::English, Self::ReviewSettingsBeforeContinuing) => {
                "Review settings before continuing"
            }
            (HubLanguage::Chinese, Self::ReviewSettingsBeforeContinuing) => "继续前检查设置",
            (HubLanguage::English, Self::CheckHighlightedWorkflowTarget) => {
                "Check the highlighted workflow target before retrying"
            }
            (HubLanguage::Chinese, Self::CheckHighlightedWorkflowTarget) => {
                "重试前检查高亮的工作流目标"
            }
        }
    }
}
