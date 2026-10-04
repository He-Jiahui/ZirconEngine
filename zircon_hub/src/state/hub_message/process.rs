use crate::settings::HubLanguage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessMessageId {
    LaunchingEditorProcess,
    SelectProjectOrLaunchEmpty,
    ChooseValidProjectForEditor,
    BuildPayloadBeforeOpeningProject,
    BuildPayloadBeforeLaunching,
    VerifyEditorAndProjectPath,
    VerifyEditorExecutable,
    EditorExecutableUnavailable,
    StartedProcess,
    OpeningTargetProcess,
    FocusedExistingEditor,
    ProcessId,
    EditorProcessExitCode,
    EditorProcessExitSignal,
    EditorProcessExitUnknown,
    EditorProcessStopped,
    EditorProcessStoppedBeforeReady,
    EditorProcessFailedBeforeReady,
    EditorProcessCleanupIncomplete,
    EditorTerminalAttemptId,
    EditorLaunchShutdownIncomplete,
    EditorChildShutdownIncomplete,
    EditorTerminalProjectionIncomplete,
}

impl ProcessMessageId {
    pub const ALL: &'static [Self] = &[
        Self::LaunchingEditorProcess,
        Self::SelectProjectOrLaunchEmpty,
        Self::ChooseValidProjectForEditor,
        Self::BuildPayloadBeforeOpeningProject,
        Self::BuildPayloadBeforeLaunching,
        Self::VerifyEditorAndProjectPath,
        Self::VerifyEditorExecutable,
        Self::EditorExecutableUnavailable,
        Self::StartedProcess,
        Self::OpeningTargetProcess,
        Self::FocusedExistingEditor,
        Self::ProcessId,
        Self::EditorProcessExitCode,
        Self::EditorProcessExitSignal,
        Self::EditorProcessExitUnknown,
        Self::EditorProcessStopped,
        Self::EditorProcessStoppedBeforeReady,
        Self::EditorProcessFailedBeforeReady,
        Self::EditorProcessCleanupIncomplete,
        Self::EditorTerminalAttemptId,
        Self::EditorLaunchShutdownIncomplete,
        Self::EditorChildShutdownIncomplete,
        Self::EditorTerminalProjectionIncomplete,
    ];

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::LaunchingEditorProcess => "process.launching-editor-process",
            Self::SelectProjectOrLaunchEmpty => "process.select-project-or-launch-empty",
            Self::ChooseValidProjectForEditor => "process.choose-valid-project-for-editor",
            Self::BuildPayloadBeforeOpeningProject => {
                "process.build-payload-before-opening-project"
            }
            Self::BuildPayloadBeforeLaunching => "process.build-payload-before-launching",
            Self::VerifyEditorAndProjectPath => "process.verify-editor-and-project-path",
            Self::VerifyEditorExecutable => "process.verify-editor-executable",
            Self::EditorExecutableUnavailable => "process.editor-executable-unavailable",
            Self::StartedProcess => "process.started-process",
            Self::OpeningTargetProcess => "process.opening-target-process",
            Self::FocusedExistingEditor => "process.focused-existing-editor",
            Self::ProcessId => "process.process-id",
            Self::EditorProcessExitCode => "process.editor-process-exit-code",
            Self::EditorProcessExitSignal => "process.editor-process-exit-signal",
            Self::EditorProcessExitUnknown => "process.editor-process-exit-unknown",
            Self::EditorProcessStopped => "process.editor-process-stopped",
            Self::EditorProcessStoppedBeforeReady => "process.editor-process-stopped-before-ready",
            Self::EditorProcessFailedBeforeReady => "process.editor-process-failed-before-ready",
            Self::EditorProcessCleanupIncomplete => "process.editor-process-cleanup-incomplete",
            Self::EditorTerminalAttemptId => "process.editor-terminal-attempt-id",
            Self::EditorLaunchShutdownIncomplete => "process.editor-launch-shutdown-incomplete",
            Self::EditorChildShutdownIncomplete => "process.editor-child-shutdown-incomplete",
            Self::EditorTerminalProjectionIncomplete => {
                "process.editor-terminal-projection-incomplete"
            }
        }
    }

    pub(super) fn param_count(self) -> usize {
        match self {
            Self::EditorExecutableUnavailable
            | Self::StartedProcess
            | Self::FocusedExistingEditor
            | Self::ProcessId
            | Self::EditorProcessExitUnknown
            | Self::EditorProcessStopped
            | Self::EditorProcessStoppedBeforeReady
            | Self::EditorProcessFailedBeforeReady
            | Self::EditorProcessCleanupIncomplete => 1,
            Self::EditorTerminalAttemptId => 1,
            Self::EditorLaunchShutdownIncomplete => 1,
            Self::OpeningTargetProcess
            | Self::EditorProcessExitCode
            | Self::EditorProcessExitSignal => 2,
            Self::EditorTerminalProjectionIncomplete => 2,
            _ => 0,
        }
    }

    pub(super) fn template(self, language: HubLanguage) -> &'static str {
        match (language, self) {
            (HubLanguage::English, Self::LaunchingEditorProcess) => "Launching staged editor process",
            (HubLanguage::Chinese, Self::LaunchingEditorProcess) => "正在启动暂存编辑器进程",
            (HubLanguage::English, Self::SelectProjectOrLaunchEmpty) => "Select an available project or launch Editor without a project",
            (HubLanguage::Chinese, Self::SelectProjectOrLaunchEmpty) => "选择一个可用项目，或不带项目启动编辑器",
            (HubLanguage::English, Self::ChooseValidProjectForEditor) => "Choose a valid Zircon project before opening it in Editor",
            (HubLanguage::Chinese, Self::ChooseValidProjectForEditor) => "在编辑器中打开前先选择一个有效的 Zircon 项目",
            (HubLanguage::English, Self::BuildPayloadBeforeOpeningProject) => "Build the editor/runtime payload or fix Source Engine settings before opening the project",
            (HubLanguage::Chinese, Self::BuildPayloadBeforeOpeningProject) => "打开项目前先构建编辑器/运行时载荷，或修复源码引擎设置",
            (HubLanguage::English, Self::BuildPayloadBeforeLaunching) => "Build the editor/runtime payload or fix Source Engine settings before launching",
            (HubLanguage::Chinese, Self::BuildPayloadBeforeLaunching) => "启动前先构建编辑器/运行时载荷，或修复源码引擎设置",
            (HubLanguage::English, Self::VerifyEditorAndProjectPath) => "Verify the staged zircon_editor executable exists and the project path is accessible",
            (HubLanguage::Chinese, Self::VerifyEditorAndProjectPath) => "确认暂存的 zircon_editor 可执行文件存在，且项目路径可访问",
            (HubLanguage::English, Self::VerifyEditorExecutable) => "Verify the staged zircon_editor executable exists",
            (HubLanguage::Chinese, Self::VerifyEditorExecutable) => "确认暂存的 zircon_editor 可执行文件存在",
            (HubLanguage::English, Self::EditorExecutableUnavailable) => "Editor executable is not available: {0}",
            (HubLanguage::Chinese, Self::EditorExecutableUnavailable) => "编辑器可执行文件不可用：{0}",
            (HubLanguage::English, Self::StartedProcess) => "Started process {0}",
            (HubLanguage::Chinese, Self::StartedProcess) => "已启动进程 {0}",
            (HubLanguage::English, Self::OpeningTargetProcess) => "Opening {0} (process {1})",
            (HubLanguage::Chinese, Self::OpeningTargetProcess) => "正在打开 {0}（进程 {1}）",
            (HubLanguage::English, Self::FocusedExistingEditor) => {
                "Focused existing editor process {0}"
            }
            (HubLanguage::Chinese, Self::FocusedExistingEditor) => {
                "已聚焦现有编辑器进程 {0}"
            }
            (HubLanguage::English, Self::ProcessId) => "Process {0}",
            (HubLanguage::Chinese, Self::ProcessId) => "进程 {0}",
            (HubLanguage::English, Self::EditorProcessExitCode) => "Editor process {0} exited with code {1}",
            (HubLanguage::Chinese, Self::EditorProcessExitCode) => "编辑器进程 {0} 已退出，退出码 {1}",
            (HubLanguage::English, Self::EditorProcessExitSignal) => "Editor process {0} exited after signal {1}",
            (HubLanguage::Chinese, Self::EditorProcessExitSignal) => "编辑器进程 {0} 因信号 {1} 退出",
            (HubLanguage::English, Self::EditorProcessExitUnknown) => "Editor process {0} exited without an exit code",
            (HubLanguage::Chinese, Self::EditorProcessExitUnknown) => "编辑器进程 {0} 已退出，未提供退出码",
            (HubLanguage::English, Self::EditorProcessStopped) => "Hub stopped Editor process {0}",
            (HubLanguage::Chinese, Self::EditorProcessStopped) => "Hub 已停止编辑器进程 {0}",
            (HubLanguage::English, Self::EditorProcessStoppedBeforeReady) => "Hub stopped Editor process {0} before Ready",
            (HubLanguage::Chinese, Self::EditorProcessStoppedBeforeReady) => "Hub 已在 Ready 前停止编辑器进程 {0}",
            (HubLanguage::English, Self::EditorProcessFailedBeforeReady) => "Editor process {0} failed before Ready",
            (HubLanguage::Chinese, Self::EditorProcessFailedBeforeReady) => "编辑器进程 {0} 在 Ready 前失败",
            (HubLanguage::English, Self::EditorProcessCleanupIncomplete) => "Editor process {0} cleanup did not complete",
            (HubLanguage::Chinese, Self::EditorProcessCleanupIncomplete) => "编辑器进程 {0} 清理未完成",
            (HubLanguage::English, Self::EditorTerminalAttemptId) => "Editor launch attempt {0}",
            (HubLanguage::Chinese, Self::EditorTerminalAttemptId) => "编辑器启动尝试 {0}",
            (HubLanguage::English, Self::EditorLaunchShutdownIncomplete) => "Editor launch task {0} did not stop before Hub shutdown",
            (HubLanguage::Chinese, Self::EditorLaunchShutdownIncomplete) => "编辑器启动任务 {0} 未在 Hub 关闭前停止",
            (HubLanguage::English, Self::EditorChildShutdownIncomplete) => "Editor child cleanup did not finish before Hub shutdown",
            (HubLanguage::Chinese, Self::EditorChildShutdownIncomplete) => "编辑器子进程清理未在 Hub 关闭前完成",
            (HubLanguage::English, Self::EditorTerminalProjectionIncomplete) => "Editor attempt {0} process {1} was reaped, but its terminal history was not saved before Hub shutdown",
            (HubLanguage::Chinese, Self::EditorTerminalProjectionIncomplete) => "编辑器尝试 {0} 的进程 {1} 已回收，但终止历史未在 Hub 关闭前保存",
        }
    }
}
