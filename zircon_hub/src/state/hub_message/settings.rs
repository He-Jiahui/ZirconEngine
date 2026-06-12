use crate::settings::HubLanguage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsMessageId {
    InitialDirectoryMustBeAbsolute,
    NoFolderSelected,
    ChooseFolderOrKeepCurrent,
    ChooseExistingFolderOrManual,
    CheckValuesAndSave,
    DraftRestoredSaved,
    DraftRestoredDefaults,
    FolderFieldRequired,
    UnknownFolderField,
    PythonRequired,
    CargoRequired,
    RustupRequired,
    DefaultProjectDirRequired,
    DefaultSourceDirRequired,
    DefaultBuildOutputDirRequired,
    DefaultDeviceInstallDirRequired,
    ProjectNameRequired,
    UnknownLanguage,
    UnknownBuildProfile,
    SettingsSavedPath,
}

impl SettingsMessageId {
    pub const ALL: &'static [Self] = &[
        Self::InitialDirectoryMustBeAbsolute,
        Self::NoFolderSelected,
        Self::ChooseFolderOrKeepCurrent,
        Self::ChooseExistingFolderOrManual,
        Self::CheckValuesAndSave,
        Self::DraftRestoredSaved,
        Self::DraftRestoredDefaults,
        Self::FolderFieldRequired,
        Self::UnknownFolderField,
        Self::PythonRequired,
        Self::CargoRequired,
        Self::RustupRequired,
        Self::DefaultProjectDirRequired,
        Self::DefaultSourceDirRequired,
        Self::DefaultBuildOutputDirRequired,
        Self::DefaultDeviceInstallDirRequired,
        Self::ProjectNameRequired,
        Self::UnknownLanguage,
        Self::UnknownBuildProfile,
        Self::SettingsSavedPath,
    ];

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::InitialDirectoryMustBeAbsolute => "settings.initial-directory-must-be-absolute",
            Self::NoFolderSelected => "settings.no-folder-selected",
            Self::ChooseFolderOrKeepCurrent => "settings.choose-folder-or-keep-current",
            Self::ChooseExistingFolderOrManual => "settings.choose-existing-folder-or-manual",
            Self::CheckValuesAndSave => "settings.check-values-and-save",
            Self::DraftRestoredSaved => "settings.draft-restored-saved",
            Self::DraftRestoredDefaults => "settings.draft-restored-defaults",
            Self::FolderFieldRequired => "settings.folder-field-required",
            Self::UnknownFolderField => "settings.unknown-folder-field",
            Self::PythonRequired => "settings.python-required",
            Self::CargoRequired => "settings.cargo-required",
            Self::RustupRequired => "settings.rustup-required",
            Self::DefaultProjectDirRequired => "settings.default-project-dir-required",
            Self::DefaultSourceDirRequired => "settings.default-source-dir-required",
            Self::DefaultBuildOutputDirRequired => "settings.default-build-output-dir-required",
            Self::DefaultDeviceInstallDirRequired => "settings.default-device-install-dir-required",
            Self::ProjectNameRequired => "settings.project-name-required",
            Self::UnknownLanguage => "settings.unknown-language",
            Self::UnknownBuildProfile => "settings.unknown-build-profile",
            Self::SettingsSavedPath => "settings.saved-path",
        }
    }

    pub(super) fn param_count(self) -> usize {
        match self {
            Self::InitialDirectoryMustBeAbsolute
            | Self::UnknownFolderField
            | Self::UnknownLanguage
            | Self::UnknownBuildProfile
            | Self::SettingsSavedPath => 1,
            _ => 0,
        }
    }

    pub(super) fn template(self, language: HubLanguage) -> &'static str {
        match (language, self) {
            (HubLanguage::English, Self::InitialDirectoryMustBeAbsolute) => {
                "Initial directory must be an absolute path: {0}"
            }
            (HubLanguage::Chinese, Self::InitialDirectoryMustBeAbsolute) => {
                "初始目录必须是绝对路径：{0}"
            }
            (HubLanguage::English, Self::NoFolderSelected) => "No folder was selected",
            (HubLanguage::Chinese, Self::NoFolderSelected) => "未选择文件夹",
            (HubLanguage::English, Self::ChooseFolderOrKeepCurrent) => {
                "Choose a folder or keep the current setting"
            }
            (HubLanguage::Chinese, Self::ChooseFolderOrKeepCurrent) => "选择文件夹或保留当前设置",
            (HubLanguage::English, Self::ChooseExistingFolderOrManual) => {
                "Choose an existing local folder or type the path manually"
            }
            (HubLanguage::Chinese, Self::ChooseExistingFolderOrManual) => {
                "选择已有本地文件夹或手动输入路径"
            }
            (HubLanguage::English, Self::CheckValuesAndSave) => {
                "Check Settings values and save again"
            }
            (HubLanguage::Chinese, Self::CheckValuesAndSave) => "检查设置值后重新保存",
            (HubLanguage::English, Self::DraftRestoredSaved) => "Draft restored to saved settings",
            (HubLanguage::Chinese, Self::DraftRestoredSaved) => "草稿已恢复为已保存设置",
            (HubLanguage::English, Self::DraftRestoredDefaults) => {
                "Draft restored to built-in defaults"
            }
            (HubLanguage::Chinese, Self::DraftRestoredDefaults) => "草稿已恢复为内置默认值",
            (HubLanguage::English, Self::FolderFieldRequired) => {
                "Settings folder field is required"
            }
            (HubLanguage::Chinese, Self::FolderFieldRequired) => "需要设置文件夹字段",
            (HubLanguage::English, Self::UnknownFolderField) => {
                "Unknown settings folder field: {0}"
            }
            (HubLanguage::Chinese, Self::UnknownFolderField) => "未知设置文件夹字段：{0}",
            (HubLanguage::English, Self::PythonRequired) => "Python executable is required",
            (HubLanguage::Chinese, Self::PythonRequired) => "需要 Python 可执行文件",
            (HubLanguage::English, Self::CargoRequired) => "Cargo executable is required",
            (HubLanguage::Chinese, Self::CargoRequired) => "需要 Cargo 可执行文件",
            (HubLanguage::English, Self::RustupRequired) => "Rustup executable is required",
            (HubLanguage::Chinese, Self::RustupRequired) => "需要 Rustup 可执行文件",
            (HubLanguage::English, Self::DefaultProjectDirRequired) => {
                "Default project directory is required"
            }
            (HubLanguage::Chinese, Self::DefaultProjectDirRequired) => "需要默认项目目录",
            (HubLanguage::English, Self::DefaultSourceDirRequired) => {
                "Default source directory is required"
            }
            (HubLanguage::Chinese, Self::DefaultSourceDirRequired) => "需要默认源码目录",
            (HubLanguage::English, Self::DefaultBuildOutputDirRequired) => {
                "Default build output directory is required"
            }
            (HubLanguage::Chinese, Self::DefaultBuildOutputDirRequired) => "需要默认构建输出目录",
            (HubLanguage::English, Self::DefaultDeviceInstallDirRequired) => {
                "Default device install directory is required"
            }
            (HubLanguage::Chinese, Self::DefaultDeviceInstallDirRequired) => "需要默认设备安装目录",
            (HubLanguage::English, Self::ProjectNameRequired) => "Project name must not be empty",
            (HubLanguage::Chinese, Self::ProjectNameRequired) => "项目名称不能为空",
            (HubLanguage::English, Self::UnknownLanguage) => "Unknown Hub language: {0}",
            (HubLanguage::Chinese, Self::UnknownLanguage) => "未知 Hub 语言：{0}",
            (HubLanguage::English, Self::UnknownBuildProfile) => "Unknown build profile: {0}",
            (HubLanguage::Chinese, Self::UnknownBuildProfile) => "未知构建配置：{0}",
            (HubLanguage::English, Self::SettingsSavedPath) => "{0}",
            (HubLanguage::Chinese, Self::SettingsSavedPath) => "{0}",
        }
    }
}
