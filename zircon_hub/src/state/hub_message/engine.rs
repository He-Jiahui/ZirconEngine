use crate::settings::HubLanguage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineMessageId {
    SourceEngineReady,
    CheckoutDirectoryMissing,
    MissingCargoToml,
    MissingRuntimeMember,
    MissingBuildTool,
    LocateCheckoutRecovery,
    SelectRepositoryRoot,
    SelectRepositoryWithRuntime,
    SelectCompleteCheckout,
    RunningBuildScript,
    StagedEditorRuntimePayload,
    SelectValidProjectWithEngine,
    CheckToolchainSettings,
    FixFirstBuildError,
    UnknownSourceEngine,
}

impl EngineMessageId {
    pub const ALL: &'static [Self] = &[
        Self::SourceEngineReady,
        Self::CheckoutDirectoryMissing,
        Self::MissingCargoToml,
        Self::MissingRuntimeMember,
        Self::MissingBuildTool,
        Self::LocateCheckoutRecovery,
        Self::SelectRepositoryRoot,
        Self::SelectRepositoryWithRuntime,
        Self::SelectCompleteCheckout,
        Self::RunningBuildScript,
        Self::StagedEditorRuntimePayload,
        Self::SelectValidProjectWithEngine,
        Self::CheckToolchainSettings,
        Self::FixFirstBuildError,
        Self::UnknownSourceEngine,
    ];

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::SourceEngineReady => "engine.source-engine-ready",
            Self::CheckoutDirectoryMissing => "engine.checkout-directory-missing",
            Self::MissingCargoToml => "engine.missing-cargo-toml",
            Self::MissingRuntimeMember => "engine.missing-runtime-member",
            Self::MissingBuildTool => "engine.missing-build-tool",
            Self::LocateCheckoutRecovery => "engine.locate-checkout-recovery",
            Self::SelectRepositoryRoot => "engine.select-repository-root",
            Self::SelectRepositoryWithRuntime => "engine.select-repository-with-runtime",
            Self::SelectCompleteCheckout => "engine.select-complete-checkout",
            Self::RunningBuildScript => "engine.running-build-script",
            Self::StagedEditorRuntimePayload => "engine.staged-editor-runtime-payload",
            Self::SelectValidProjectWithEngine => "engine.select-valid-project-with-engine",
            Self::CheckToolchainSettings => "engine.check-toolchain-settings",
            Self::FixFirstBuildError => "engine.fix-first-build-error",
            Self::UnknownSourceEngine => "engine.unknown-source-engine",
        }
    }

    pub(super) fn param_count(self) -> usize {
        match self {
            Self::UnknownSourceEngine => 1,
            _ => 0,
        }
    }

    pub(super) fn template(self, language: HubLanguage) -> &'static str {
        match (language, self) {
            (HubLanguage::English, Self::SourceEngineReady) => "Source engine is ready",
            (HubLanguage::Chinese, Self::SourceEngineReady) => "源码引擎已就绪",
            (HubLanguage::English, Self::CheckoutDirectoryMissing) => "Source checkout directory is missing",
            (HubLanguage::Chinese, Self::CheckoutDirectoryMissing) => "源码检出目录缺失",
            (HubLanguage::English, Self::MissingCargoToml) => "Source checkout is missing Cargo.toml",
            (HubLanguage::Chinese, Self::MissingCargoToml) => "源码检出缺少 Cargo.toml",
            (HubLanguage::English, Self::MissingRuntimeMember) => "Source checkout workspace is missing zircon_runtime member",
            (HubLanguage::Chinese, Self::MissingRuntimeMember) => "源码检出工作区缺少 zircon_runtime 成员",
            (HubLanguage::English, Self::MissingBuildTool) => "Source checkout is missing tools/zircon_build.py",
            (HubLanguage::Chinese, Self::MissingBuildTool) => "源码检出缺少 tools/zircon_build.py",
            (HubLanguage::English, Self::LocateCheckoutRecovery) => "Locate an existing ZirconEngine checkout or update Settings > Source Checkout",
            (HubLanguage::Chinese, Self::LocateCheckoutRecovery) => "定位已有 ZirconEngine 检出，或更新设置 > 源码检出",
            (HubLanguage::English, Self::SelectRepositoryRoot) => "Select the ZirconEngine repository root that contains the workspace Cargo.toml",
            (HubLanguage::Chinese, Self::SelectRepositoryRoot) => "选择包含工作区 Cargo.toml 的 ZirconEngine 仓库根目录",
            (HubLanguage::English, Self::SelectRepositoryWithRuntime) => "Select the ZirconEngine repository root whose Cargo workspace includes zircon_runtime",
            (HubLanguage::Chinese, Self::SelectRepositoryWithRuntime) => "选择 Cargo 工作区包含 zircon_runtime 的 ZirconEngine 仓库根目录",
            (HubLanguage::English, Self::SelectCompleteCheckout) => "Select a complete ZirconEngine checkout with tools/zircon_build.py before building",
            (HubLanguage::Chinese, Self::SelectCompleteCheckout) => "构建前选择包含 tools/zircon_build.py 的完整 ZirconEngine 检出",
            (HubLanguage::English, Self::RunningBuildScript) => "Running tools/zircon_build.py",
            (HubLanguage::Chinese, Self::RunningBuildScript) => "正在运行 tools/zircon_build.py",
            (HubLanguage::English, Self::StagedEditorRuntimePayload) => "Staged editor/runtime payload",
            (HubLanguage::Chinese, Self::StagedEditorRuntimePayload) => "已暂存编辑器/运行时载荷",
            (HubLanguage::English, Self::SelectValidProjectWithEngine) => "Select a valid project with a bound Source Engine before building",
            (HubLanguage::Chinese, Self::SelectValidProjectWithEngine) => "构建前先选择一个已绑定源码引擎的有效项目",
            (HubLanguage::English, Self::CheckToolchainSettings) => "Check Python, Cargo, and Source Checkout settings before retrying",
            (HubLanguage::Chinese, Self::CheckToolchainSettings) => "重试前检查 Python、Cargo 和源码检出设置",
            (HubLanguage::English, Self::FixFirstBuildError) => "Open Build History and fix the first reported error before retrying",
            (HubLanguage::Chinese, Self::FixFirstBuildError) => "打开构建历史并修复第一条错误后再重试",
            (HubLanguage::English, Self::UnknownSourceEngine) => "Unknown Source Engine: {0}",
            (HubLanguage::Chinese, Self::UnknownSourceEngine) => "未知源码引擎：{0}",
        }
    }
}
