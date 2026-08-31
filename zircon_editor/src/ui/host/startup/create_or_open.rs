use std::borrow::Cow;
use std::fmt::Display;
use std::path::Path;

use crate::core::project::NewProjectDraft;
use crate::ui::host::project_access::project_open_is_degraded;
use crate::ui::workbench::project::EditorProjectDocument;
use crate::ui::workbench::startup::{EditorSessionMode, EditorStartupSessionDocument};
use zircon_runtime::asset::project::ProjectManager;

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;

impl EditorUiHost {
    pub(in crate::ui::host) fn remember_prepared_project(
        &self,
        document: EditorProjectDocument,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        let status_message = project_open_status_message(&document);
        self.dismiss_welcome_page()?;

        Ok(EditorStartupSessionDocument {
            mode: EditorSessionMode::Project,
            project: Some(document),
            open_builtin_view: None,
            recent_projects: self.recent_projects_snapshot()?,
            draft: NewProjectDraft::renderable_empty_default(),
            creation_validation: String::new(),
            can_open_existing: false,
            status_message,
        })
    }
}

pub(super) fn project_open_status_message(document: &EditorProjectDocument) -> String {
    project_activation_status_message("Project opened", document)
}

pub(super) fn restored_project_status_message(document: &EditorProjectDocument) -> String {
    project_activation_status_message("Restored recent project", document)
}

fn project_activation_status_message(action: &str, document: &EditorProjectDocument) -> String {
    let action = project_activation_action(
        action,
        document.project_info.asset_count,
        document.project_info.ready_asset_count,
        document.project_info.failed_asset_count,
        document.project_settings.startup_status(),
    );
    let diagnostic = document
        .workspace_restore_diagnostics
        .first()
        .map(|diagnostic| (diagnostic.path.as_path(), diagnostic.message.as_str()));
    project_activation_summary(
        &action,
        &document.project_info.name,
        &document.project_info.default_scene_uri,
        document.project_info.asset_count,
        document.project_info.ready_asset_count,
        document.project_info.failed_asset_count,
        document.project_info.registry_diagnostic_count,
        document.project_settings.startup_status(),
        diagnostic,
    )
}

#[allow(clippy::too_many_arguments)]
fn project_activation_summary(
    action: &str,
    project_name: &str,
    default_scene_uri: impl Display,
    asset_count: usize,
    ready_asset_count: usize,
    failed_asset_count: usize,
    registry_diagnostic_count: usize,
    settings_source: &str,
    workspace_restore_diagnostic: Option<(&Path, &str)>,
) -> String {
    match workspace_restore_diagnostic {
        Some((diagnostic_path, diagnostic_message)) => format!(
            "{action}: {project_name} (scene={default_scene_uri} assets={asset_count} ready={ready_asset_count} failed={failed_asset_count} registry_diagnostics={registry_diagnostic_count} project_settings={settings_source}); using default layout after workspace restore failed from {}: {diagnostic_message}",
            diagnostic_path.display(),
        ),
        None => format!(
            "{action}: {project_name} (scene={default_scene_uri} assets={asset_count} ready={ready_asset_count} failed={failed_asset_count} registry_diagnostics={registry_diagnostic_count} project_settings={settings_source})",
        ),
    }
}

fn project_activation_action<'a>(
    action: &'a str,
    registry_asset_count: usize,
    registry_ready_asset_count: usize,
    registry_failed_asset_count: usize,
    settings_source: &str,
) -> Cow<'a, str> {
    if project_open_is_degraded(
        registry_asset_count,
        registry_ready_asset_count,
        registry_failed_asset_count,
        settings_source,
    ) {
        Cow::Owned(format!("{action} (degraded)"))
    } else {
        Cow::Borrowed(action)
    }
}

#[cfg(test)]
mod tests {
    use super::project_activation_action;

    #[test]
    fn opened_project_is_not_reopened_just_to_update_recents() {
        let source = include_str!("create_or_open.rs");
        let reopening_call = ["self", ".update_recent_project(&document.root_path)"].concat();
        assert!(!source.contains(&reopening_call));
    }

    #[test]
    fn degraded_project_inputs_are_visible_in_the_startup_status_action() {
        assert_eq!(
            project_activation_action("Project opened", 4, 4, 0, "persisted-v1"),
            "Project opened"
        );
        assert_eq!(
            project_activation_action("Project opened", 4, 3, 0, "persisted-v1"),
            "Project opened (degraded)"
        );
        assert_eq!(
            project_activation_action("Restored recent project", 4, 4, 0, "degraded-missing"),
            "Restored recent project (degraded)"
        );
    }
}

#[cfg(test)]
#[path = "create_or_open/single_buffer_activation_status_tests.rs"]
mod single_buffer_activation_status_tests;
