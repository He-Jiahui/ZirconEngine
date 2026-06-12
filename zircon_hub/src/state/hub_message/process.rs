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
    ProcessId,
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
        Self::ProcessId,
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
            Self::ProcessId => "process.process-id",
        }
    }

    pub(super) fn param_count(self) -> usize {
        match self {
            Self::EditorExecutableUnavailable | Self::StartedProcess | Self::ProcessId => 1,
            Self::OpeningTargetProcess => 2,
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
            (HubLanguage::English, Self::ProcessId) => "Process {0}",
            (HubLanguage::Chinese, Self::ProcessId) => "进程 {0}",
        }
    }
}
