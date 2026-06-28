#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HubActionId {
    ShowPage,
    ShowProjectSubpage,
    SearchProjects,
    SetProjectFilter,
    SetProjectSort,
    SetProjectViewMode,
    SelectProject,
    OpenProjectDetail,
    ViewAllProjects,
    NewProject,
    UpdateNewProjectDraft,
    SelectEngine,
    UpdateSettingsDraft,
    SaveSettings,
    DiscardSettingsDraft,
    RestoreDefaultSettings,
    BrowseSettingsFolder,
    CreateProject,
    ImportProject,
    PinProject,
    UnpinProject,
    RemoveFromHub,
    RequestDelete,
    CancelDelete,
    ConfirmDelete,
    OpenResource,
    OpenOutputFolder,
    BuildProject,
    PackageProject,
    InstallDevice,
    OpenEditor,
}

impl HubActionId {
    pub(crate) const ALL: [HubActionId; 31] = [
        Self::ShowPage,
        Self::ShowProjectSubpage,
        Self::SearchProjects,
        Self::SetProjectFilter,
        Self::SetProjectSort,
        Self::SetProjectViewMode,
        Self::SelectProject,
        Self::OpenProjectDetail,
        Self::ViewAllProjects,
        Self::NewProject,
        Self::UpdateNewProjectDraft,
        Self::SelectEngine,
        Self::UpdateSettingsDraft,
        Self::SaveSettings,
        Self::DiscardSettingsDraft,
        Self::RestoreDefaultSettings,
        Self::BrowseSettingsFolder,
        Self::CreateProject,
        Self::ImportProject,
        Self::PinProject,
        Self::UnpinProject,
        Self::RemoveFromHub,
        Self::RequestDelete,
        Self::CancelDelete,
        Self::ConfirmDelete,
        Self::OpenResource,
        Self::OpenOutputFolder,
        Self::BuildProject,
        Self::PackageProject,
        Self::InstallDevice,
        Self::OpenEditor,
    ];

    /// Canonical wire action id table. Archived aliases belong in `from_str`.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::ShowPage => "show-page",
            Self::ShowProjectSubpage => "show-project-subpage",
            Self::SearchProjects => "search-projects",
            Self::SetProjectFilter => "set-project-filter",
            Self::SetProjectSort => "set-project-sort",
            Self::SetProjectViewMode => "set-project-view-mode",
            Self::SelectProject => "select-project",
            Self::OpenProjectDetail => "open-project-detail",
            Self::ViewAllProjects => "view-all-projects",
            Self::NewProject => "new-project",
            Self::UpdateNewProjectDraft => "update-new-project-draft",
            Self::SelectEngine => "select-engine",
            Self::UpdateSettingsDraft => "update-settings-draft",
            Self::SaveSettings => "save-settings",
            Self::DiscardSettingsDraft => "discard-settings-draft",
            Self::RestoreDefaultSettings => "restore-default-settings",
            Self::BrowseSettingsFolder => "browse-settings-folder",
            Self::CreateProject => "create-project",
            Self::ImportProject => "import-project",
            Self::PinProject => "pin-project",
            Self::UnpinProject => "unpin-project",
            Self::RemoveFromHub => "remove-from-hub",
            Self::RequestDelete => "request-delete",
            Self::CancelDelete => "cancel-delete",
            Self::ConfirmDelete => "confirm-delete",
            Self::OpenResource => "open-resource",
            Self::OpenOutputFolder => "open-output-folder",
            Self::BuildProject => "build-project",
            Self::PackageProject => "package-project",
            Self::InstallDevice => "install-device",
            Self::OpenEditor => "open-editor",
        }
    }

    pub(crate) fn from_str(value: &str) -> Option<Self> {
        let value = value.trim();
        if let Some(action) = Self::ALL
            .into_iter()
            .find(|action| action.as_str() == value)
        {
            return Some(action);
        }

        match value {
            "page" => Some(Self::ShowPage),
            "project-subpage" => Some(Self::ShowProjectSubpage),
            "open-project" => Some(Self::SelectProject),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_action_id_round_trips_between_as_str_and_from_str() {
        assert_eq!(HubActionId::ALL.len(), 31);
        for action in HubActionId::ALL {
            assert_eq!(HubActionId::from_str(action.as_str()), Some(action));
        }
    }

    #[test]
    fn archived_aliases_and_whitespace_resolve_to_canonical_actions() {
        assert_eq!(HubActionId::from_str("page"), Some(HubActionId::ShowPage));
        assert_eq!(
            HubActionId::from_str("project-subpage"),
            Some(HubActionId::ShowProjectSubpage)
        );
        assert_eq!(
            HubActionId::from_str("open-project"),
            Some(HubActionId::SelectProject)
        );
        assert_eq!(
            HubActionId::from_str(" build-project "),
            Some(HubActionId::BuildProject)
        );
        assert_eq!(HubActionId::from_str("upload-to-cloud"), None);
    }
}
