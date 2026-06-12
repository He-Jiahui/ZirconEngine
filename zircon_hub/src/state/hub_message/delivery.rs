use crate::settings::HubLanguage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliveryMessageId {
    FileCountDetail,
    PackageLogExcerpt,
    InstallLogExcerpt,
    CopyingProjectToPackage,
    PreparingPackageInstall,
    ProjectRootUnavailable,
    PackageOutputRootRequired,
    PackageOutputOutsideProject,
    CheckPackageOutputRecovery,
    PackageDirectoryUnavailable,
    PackageDirectoryAlreadyExists,
    DeviceInstallRequired,
    DeviceInstallOutsidePackage,
    DeviceInstallAlreadyExists,
    CheckInstallOutputRecovery,
    OutputFolderDoesNotExist,
    OpenOutputTargetRequired,
    OpenContainingFolderRecovery,
    ChooseRecordedOutputRecovery,
    RunWorkflowAgainRecovery,
    OpenFolderManuallyRecovery,
    OutputPathMustBeAbsolute,
    OutputDirectoryMustBeAbsolute,
}

impl DeliveryMessageId {
    pub const ALL: &'static [Self] = &[
        Self::FileCountDetail,
        Self::PackageLogExcerpt,
        Self::InstallLogExcerpt,
        Self::CopyingProjectToPackage,
        Self::PreparingPackageInstall,
        Self::ProjectRootUnavailable,
        Self::PackageOutputRootRequired,
        Self::PackageOutputOutsideProject,
        Self::CheckPackageOutputRecovery,
        Self::PackageDirectoryUnavailable,
        Self::PackageDirectoryAlreadyExists,
        Self::DeviceInstallRequired,
        Self::DeviceInstallOutsidePackage,
        Self::DeviceInstallAlreadyExists,
        Self::CheckInstallOutputRecovery,
        Self::OutputFolderDoesNotExist,
        Self::OpenOutputTargetRequired,
        Self::OpenContainingFolderRecovery,
        Self::ChooseRecordedOutputRecovery,
        Self::RunWorkflowAgainRecovery,
        Self::OpenFolderManuallyRecovery,
        Self::OutputPathMustBeAbsolute,
        Self::OutputDirectoryMustBeAbsolute,
    ];

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::FileCountDetail => "delivery.file-count-detail",
            Self::PackageLogExcerpt => "delivery.package-log-excerpt",
            Self::InstallLogExcerpt => "delivery.install-log-excerpt",
            Self::CopyingProjectToPackage => "delivery.copying-project-to-package",
            Self::PreparingPackageInstall => "delivery.preparing-package-install",
            Self::ProjectRootUnavailable => "delivery.project-root-unavailable",
            Self::PackageOutputRootRequired => "delivery.package-output-root-required",
            Self::PackageOutputOutsideProject => "delivery.package-output-outside-project",
            Self::CheckPackageOutputRecovery => "delivery.check-package-output-recovery",
            Self::PackageDirectoryUnavailable => "delivery.package-directory-unavailable",
            Self::PackageDirectoryAlreadyExists => "delivery.package-directory-already-exists",
            Self::DeviceInstallRequired => "delivery.device-install-required",
            Self::DeviceInstallOutsidePackage => "delivery.device-install-outside-package",
            Self::DeviceInstallAlreadyExists => "delivery.device-install-already-exists",
            Self::CheckInstallOutputRecovery => "delivery.check-install-output-recovery",
            Self::OutputFolderDoesNotExist => "delivery.output-folder-does-not-exist",
            Self::OpenOutputTargetRequired => "delivery.open-output-target-required",
            Self::OpenContainingFolderRecovery => "delivery.open-containing-folder-recovery",
            Self::ChooseRecordedOutputRecovery => "delivery.choose-recorded-output-recovery",
            Self::RunWorkflowAgainRecovery => "delivery.run-workflow-again-recovery",
            Self::OpenFolderManuallyRecovery => "delivery.open-folder-manually-recovery",
            Self::OutputPathMustBeAbsolute => "delivery.output-path-must-be-absolute",
            Self::OutputDirectoryMustBeAbsolute => "delivery.output-directory-must-be-absolute",
        }
    }

    pub(super) fn param_count(self) -> usize {
        match self {
            Self::FileCountDetail | Self::PackageLogExcerpt | Self::InstallLogExcerpt => 3,
            Self::PackageDirectoryAlreadyExists
            | Self::DeviceInstallAlreadyExists
            | Self::OutputFolderDoesNotExist
            | Self::OutputPathMustBeAbsolute
            | Self::OutputDirectoryMustBeAbsolute => 1,
            _ => 0,
        }
    }

    pub(super) fn template(self, language: HubLanguage) -> &'static str {
        match (language, self) {
            (HubLanguage::English, Self::FileCountDetail) => "{0} -> {1} ({2} files)",
            (HubLanguage::Chinese, Self::FileCountDetail) => "{0} -> {1}（{2} 个文件）",
            (HubLanguage::English, Self::PackageLogExcerpt) => "Packaged {0} to {1} ({2} files)",
            (HubLanguage::Chinese, Self::PackageLogExcerpt) => "已打包 {0} 到 {1}（{2} 个文件）",
            (HubLanguage::English, Self::InstallLogExcerpt) => "Installed {0} to {1} ({2} files)",
            (HubLanguage::Chinese, Self::InstallLogExcerpt) => "已安装 {0} 到 {1}（{2} 个文件）",
            (HubLanguage::English, Self::CopyingProjectToPackage) => "Copying project into package output",
            (HubLanguage::Chinese, Self::CopyingProjectToPackage) => "正在复制项目到包输出目录",
            (HubLanguage::English, Self::PreparingPackageInstall) => "Preparing package and copying to local device install directory",
            (HubLanguage::Chinese, Self::PreparingPackageInstall) => "正在准备包并复制到本地设备安装目录",
            (HubLanguage::English, Self::ProjectRootUnavailable) => "Project root is not available for packaging",
            (HubLanguage::Chinese, Self::ProjectRootUnavailable) => "项目根目录不可用于打包",
            (HubLanguage::English, Self::PackageOutputRootRequired) => "Package output root is required",
            (HubLanguage::Chinese, Self::PackageOutputRootRequired) => "需要包输出根目录",
            (HubLanguage::English, Self::PackageOutputOutsideProject) => "Package output root must be outside the project directory",
            (HubLanguage::Chinese, Self::PackageOutputOutsideProject) => "包输出根目录必须位于项目目录外",
            (HubLanguage::English, Self::CheckPackageOutputRecovery) => "Check that the project root exists and the package output is outside the project",
            (HubLanguage::Chinese, Self::CheckPackageOutputRecovery) => "检查项目根目录是否存在，并确保包输出目录位于项目外",
            (HubLanguage::English, Self::PackageDirectoryUnavailable) => "Package directory is not available",
            (HubLanguage::Chinese, Self::PackageDirectoryUnavailable) => "包目录不可用",
            (HubLanguage::English, Self::PackageDirectoryAlreadyExists) => "Package directory already exists: {0}",
            (HubLanguage::Chinese, Self::PackageDirectoryAlreadyExists) => "包目录已存在：{0}",
            (HubLanguage::English, Self::DeviceInstallRequired) => "Device install directory is required",
            (HubLanguage::Chinese, Self::DeviceInstallRequired) => "需要设备安装目录",
            (HubLanguage::English, Self::DeviceInstallOutsidePackage) => "Device install directory must be outside the package directory",
            (HubLanguage::Chinese, Self::DeviceInstallOutsidePackage) => "设备安装目录必须位于包目录外",
            (HubLanguage::English, Self::DeviceInstallAlreadyExists) => "Device install already exists: {0}",
            (HubLanguage::Chinese, Self::DeviceInstallAlreadyExists) => "设备安装已存在：{0}",
            (HubLanguage::English, Self::CheckInstallOutputRecovery) => "Check the package output and configured local device install directory before retrying",
            (HubLanguage::Chinese, Self::CheckInstallOutputRecovery) => "重试前检查包输出和已配置的本地设备安装目录",
            (HubLanguage::English, Self::OutputFolderDoesNotExist) => "Output folder does not exist: {0}",
            (HubLanguage::Chinese, Self::OutputFolderDoesNotExist) => "输出文件夹不存在：{0}",
            (HubLanguage::English, Self::OpenOutputTargetRequired) => "Open Output target is required",
            (HubLanguage::Chinese, Self::OpenOutputTargetRequired) => "需要打开输出目标",
            (HubLanguage::English, Self::OpenContainingFolderRecovery) => "Open the containing folder from the file system and verify shell integration",
            (HubLanguage::Chinese, Self::OpenContainingFolderRecovery) => "从文件系统打开所在文件夹，并检查系统外壳集成",
            (HubLanguage::English, Self::ChooseRecordedOutputRecovery) => "Choose a recorded package, install, or build output before opening the folder",
            (HubLanguage::Chinese, Self::ChooseRecordedOutputRecovery) => "打开文件夹前先选择已记录的包、安装或构建输出",
            (HubLanguage::English, Self::RunWorkflowAgainRecovery) => "Run the build, package, or install workflow again and then open its output folder",
            (HubLanguage::Chinese, Self::RunWorkflowAgainRecovery) => "重新运行构建、打包或安装工作流后再打开输出文件夹",
            (HubLanguage::English, Self::OpenFolderManuallyRecovery) => "Open the folder manually from the file system and verify shell integration",
            (HubLanguage::Chinese, Self::OpenFolderManuallyRecovery) => "从文件系统手动打开文件夹，并检查系统外壳集成",
            (HubLanguage::English, Self::OutputPathMustBeAbsolute) => "Output path must be an absolute path: {0}",
            (HubLanguage::Chinese, Self::OutputPathMustBeAbsolute) => "输出路径必须是绝对路径：{0}",
            (HubLanguage::English, Self::OutputDirectoryMustBeAbsolute) => "Output directory must be an absolute path: {0}",
            (HubLanguage::Chinese, Self::OutputDirectoryMustBeAbsolute) => "输出目录必须是绝对路径：{0}",
        }
    }
}
