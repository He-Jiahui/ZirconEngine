#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HubQuickAction {
    BuildProject,
    InstallToDevice,
    PackageProject,
    OpenEditor,
}

impl HubQuickAction {
    pub(super) fn id(self) -> &'static str {
        match self {
            Self::BuildProject => "build-project",
            Self::InstallToDevice => "install-device",
            Self::PackageProject => "package-project",
            Self::OpenEditor => "open-editor",
        }
    }

    pub(super) fn from_id(id: &str) -> Option<Self> {
        match id.trim() {
            "build-project" => Some(Self::BuildProject),
            "install-device" => Some(Self::InstallToDevice),
            "package-project" => Some(Self::PackageProject),
            "open-editor" => Some(Self::OpenEditor),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HubPageHeaderAction {
    RefreshSources,
    OpenOutput,
    PackageProject,
    RequestReview,
    RefreshAssets,
    RefreshPlugins,
    RefreshLearn,
    ResetSettings,
    OpenEditor,
    BuildProject,
    DeployPreview,
    OpenSourceControl,
    AddAsset,
    AddPlugin,
    AddGuide,
    SaveSettings,
}

impl HubPageHeaderAction {
    pub(super) fn from_id(id: &str) -> Option<Self> {
        match id.trim() {
            "refresh-sources" => Some(Self::RefreshSources),
            "open-output" => Some(Self::OpenOutput),
            "package-project" => Some(Self::PackageProject),
            "request-review" => Some(Self::RequestReview),
            "refresh-assets" => Some(Self::RefreshAssets),
            "refresh-plugins" => Some(Self::RefreshPlugins),
            "refresh-learn" => Some(Self::RefreshLearn),
            "reset-settings" => Some(Self::ResetSettings),
            "open-editor" => Some(Self::OpenEditor),
            "build-project" => Some(Self::BuildProject),
            "deploy-preview" => Some(Self::DeployPreview),
            "open-source-control" => Some(Self::OpenSourceControl),
            "add-asset" => Some(Self::AddAsset),
            "add-plugin" => Some(Self::AddPlugin),
            "add-guide" => Some(Self::AddGuide),
            "save-settings" => Some(Self::SaveSettings),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HubPageHeaderAction, HubQuickAction};

    #[test]
    fn quick_action_parses_known_ids() {
        assert_eq!(
            HubQuickAction::from_id(HubQuickAction::BuildProject.id()),
            Some(HubQuickAction::BuildProject)
        );
        assert_eq!(
            HubQuickAction::from_id("install-device"),
            Some(HubQuickAction::InstallToDevice)
        );
        assert_eq!(HubQuickAction::from_id("unknown"), None);
    }

    #[test]
    fn page_header_action_parses_known_ids() {
        assert_eq!(
            HubPageHeaderAction::from_id("refresh-sources"),
            Some(HubPageHeaderAction::RefreshSources)
        );
        assert_eq!(
            HubPageHeaderAction::from_id("open-source-control"),
            Some(HubPageHeaderAction::OpenSourceControl)
        );
        assert_eq!(
            HubPageHeaderAction::from_id("save-settings"),
            Some(HubPageHeaderAction::SaveSettings)
        );
        assert_eq!(HubPageHeaderAction::from_id("unknown"), None);
    }
}
