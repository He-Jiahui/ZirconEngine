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

    pub(super) fn unavailable_detail(self) -> &'static str {
        match self {
            Self::InstallToDevice => "Device deployment is not connected yet.",
            Self::PackageProject => "Project packaging is not connected yet.",
            Self::BuildProject | Self::OpenEditor => "Action is available.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HubQuickAction;

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
}
