use std::{collections::BTreeMap, path::PathBuf};

use crate::ui::host::{
    export_wizard_compile_host_executable_path, ExportWizardPanelAction, ExportWizardPanelRequest,
    ExportWizardPanelSession, ExportWizardPanelSessionError, ExportWizardPanelUpdate,
    ExportWizardPanelViewModel, ExportWizardPipelineOptions, ProcessCommandRunner,
    DESKTOP_EXPORT_CANCEL_BUTTON, DESKTOP_EXPORT_GENERATE_PLAN_BUTTON, DESKTOP_EXPORT_START_BUTTON,
};
use crate::ui::workbench::project::project_root_path;
use zircon_runtime::plugin::ExportProfile;

use super::*;

const EXPORT_WIZARD_JOB_ID_PREFIX: &str = "workbench.build_export_desktop";
const DEFAULT_SOURCE_ASSET_MANIFEST: &str = "assets/assets.json";

#[derive(Debug, PartialEq, Eq)]
pub(super) struct BuildExportWizardSurfaceAction<'a> {
    pub(super) profile_name: &'a str,
    pub(super) action: ExportWizardPanelAction,
}

// Owns retained app export-wizard state by profile so the pane projection can
// refresh from host state instead of rebuilding a synthetic view model each frame.
#[derive(Default)]
pub(super) struct DesktopExportWizardSessions {
    sessions: BTreeMap<String, ExportWizardPanelSession>,
    last_updates: BTreeMap<String, ExportWizardPanelUpdate>,
}

impl DesktopExportWizardSessions {
    pub(super) fn view_model(&self, profile_name: &str) -> Option<&ExportWizardPanelViewModel> {
        self.sessions
            .get(profile_name)
            .map(ExportWizardPanelSession::view_model)
    }

    pub(super) fn dispatch_profile_action(
        &mut self,
        profile_name: &str,
        action: ExportWizardPanelAction,
        options: Option<ExportWizardPipelineOptions>,
    ) -> Result<ExportWizardPanelUpdate, ExportWizardPanelSessionError> {
        self.dispatch_profile_action_with_runner(
            profile_name,
            action,
            options,
            ProcessCommandRunner,
        )
    }

    fn dispatch_profile_action_with_runner(
        &mut self,
        profile_name: &str,
        action: ExportWizardPanelAction,
        options: Option<ExportWizardPipelineOptions>,
        start_runner: impl crate::ui::host::ExportWizardCommandRunner + Send + 'static,
    ) -> Result<ExportWizardPanelUpdate, ExportWizardPanelSessionError> {
        let update = match action {
            ExportWizardPanelAction::GeneratePlan => {
                let options = required_options(action, options)?;
                self.regenerate_profile_plan(profile_name, options)?
            }
            ExportWizardPanelAction::Start => {
                let options = required_options(action, options)?;
                self.regenerate_profile_plan(profile_name, options)?;
                self.session_mut(profile_name)?
                    .handle_start_request_with_runner(start_runner)?
            }
            ExportWizardPanelAction::Cancel => self
                .session_mut(profile_name)?
                .handle_request(ExportWizardPanelRequest::Cancel)?,
            ExportWizardPanelAction::Poll => self
                .session_mut(profile_name)?
                .handle_request(ExportWizardPanelRequest::Poll)?,
        };
        self.last_updates
            .insert(profile_name.to_string(), update.clone());
        Ok(update)
    }

    fn regenerate_profile_plan(
        &mut self,
        profile_name: &str,
        options: ExportWizardPipelineOptions,
    ) -> Result<ExportWizardPanelUpdate, ExportWizardPanelSessionError> {
        let job_id = export_wizard_job_id(profile_name);
        let session = self
            .sessions
            .entry(profile_name.to_string())
            .or_insert_with(|| {
                ExportWizardPanelSession::from_options(job_id.clone(), options.clone())
            });
        session.handle_request(ExportWizardPanelRequest::generate_plan(job_id, options))
    }

    pub(super) fn poll_all(
        &mut self,
    ) -> Vec<(
        String,
        Result<ExportWizardPanelUpdate, ExportWizardPanelSessionError>,
    )> {
        let profile_names = self.sessions.keys().cloned().collect::<Vec<_>>();
        let mut updates = Vec::new();
        for profile_name in profile_names {
            let before = self
                .sessions
                .get(profile_name.as_str())
                .map(|session| session.view_model().snapshot().clone());
            let result = self
                .session_mut(profile_name.as_str())
                .and_then(|session| session.handle_request(ExportWizardPanelRequest::Poll));
            let changed = match &result {
                Ok(update) => {
                    update.events_drained > 0
                        || before.as_ref() != Some(&update.snapshot)
                        || self
                            .last_updates
                            .get(profile_name.as_str())
                            .is_some_and(|previous| previous != update)
                }
                Err(_) => true,
            };
            if changed {
                if let Ok(update) = &result {
                    self.last_updates
                        .insert(profile_name.clone(), update.clone());
                }
                updates.push((profile_name, result));
            }
        }
        updates
    }

    fn session_mut(
        &mut self,
        profile_name: &str,
    ) -> Result<&mut ExportWizardPanelSession, ExportWizardPanelSessionError> {
        self.sessions.get_mut(profile_name).ok_or_else(|| {
            ExportWizardPanelSessionError::NoActiveJob {
                job_id: export_wizard_job_id(profile_name),
            }
        })
    }
}

impl RetainedEditorHost {
    pub(super) fn dispatch_build_export_surface_action(
        &mut self,
        control_id: &str,
        action_id: &str,
    ) {
        let Some(surface_action) = build_export_wizard_surface_action(control_id, action_id) else {
            self.dispatch_build_export_action(action_id);
            return;
        };

        let options = match surface_action.action {
            ExportWizardPanelAction::GeneratePlan | ExportWizardPanelAction::Start => {
                match self.export_wizard_options(surface_action.profile_name) {
                    Ok(options) => Some(options),
                    Err(error) => {
                        self.set_status_line(format!("Build/export wizard action failed: {error}"));
                        return;
                    }
                }
            }
            ExportWizardPanelAction::Cancel | ExportWizardPanelAction::Poll => None,
        };

        match self.desktop_export_wizard_sessions.dispatch_profile_action(
            surface_action.profile_name,
            surface_action.action,
            options,
        ) {
            Ok(update) => {
                self.apply_export_wizard_update(surface_action.profile_name, &update);
            }
            Err(error) => {
                self.set_status_line(format!("Build/export wizard action failed: {error}"));
            }
        }
    }

    pub(super) fn poll_desktop_export_wizard_sessions(&mut self) {
        let updates = self.desktop_export_wizard_sessions.poll_all();
        if updates.is_empty() {
            return;
        }
        for (profile_name, result) in updates {
            match result {
                Ok(update) => {
                    if update.events_drained > 0 || update.snapshot.is_terminal() {
                        self.set_status_line(export_wizard_status_message(
                            profile_name.as_str(),
                            &update,
                        ));
                    }
                }
                Err(error) => self.set_status_line(format!(
                    "Build/export wizard poll failed for {profile_name}: {error}"
                )),
            }
        }
        self.mark_layout_dirty();
    }

    fn apply_export_wizard_update(&mut self, profile_name: &str, update: &ExportWizardPanelUpdate) {
        self.mark_layout_dirty();
        self.set_status_line(export_wizard_status_message(profile_name, update));
    }

    fn export_wizard_options(
        &self,
        profile_name: &str,
    ) -> Result<ExportWizardPipelineOptions, String> {
        let project_path = self.runtime.editor_snapshot().project_path;
        let project_root = project_root_path(&project_path).map_err(|error| error.to_string())?;
        let manifest_path = project_root.join("zircon-project.toml");
        let output_root = self.effective_desktop_export_output_root(&project_root, profile_name);
        let profile = build_export_actions::desktop_export_profile(profile_name)
            .ok_or_else(|| format!("unknown desktop export profile `{profile_name}`"))?;
        let mut options = ExportWizardPipelineOptions::new(
            profile_name,
            manifest_path.display().to_string(),
            output_root.display().to_string(),
        );
        options.repo_root = Some(export_wizard_engine_repo_root().display().to_string());
        options.source_asset_manifest = Some(
            output_root
                .join(DEFAULT_SOURCE_ASSET_MANIFEST)
                .display()
                .to_string(),
        );
        options.host_executable = Some(export_wizard_default_host_executable(
            &options.out,
            &profile,
            options.target_dir.as_deref(),
        ));
        options.target_platform =
            Some(build_export_actions::export_platform_label(profile.target_platform).to_string());
        Ok(options)
    }
}

fn export_wizard_default_host_executable(
    out: &str,
    profile: &ExportProfile,
    target_dir: Option<&str>,
) -> String {
    export_wizard_compile_host_executable_path(out, profile, target_dir)
}

fn export_wizard_engine_repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should be inside the engine repository")
        .to_path_buf()
}

pub(super) fn build_export_wizard_surface_action<'a>(
    control_id: &str,
    action_id: &'a str,
) -> Option<BuildExportWizardSurfaceAction<'a>> {
    let action = match build_export_actions::parse_build_export_action(action_id)? {
        build_export_actions::BuildExportAction::GeneratePlan { profile_name }
            if control_id == DESKTOP_EXPORT_GENERATE_PLAN_BUTTON =>
        {
            BuildExportWizardSurfaceAction {
                profile_name,
                action: ExportWizardPanelAction::GeneratePlan,
            }
        }
        build_export_actions::BuildExportAction::Execute { profile_name }
            if control_id == DESKTOP_EXPORT_START_BUTTON =>
        {
            BuildExportWizardSurfaceAction {
                profile_name,
                action: ExportWizardPanelAction::Start,
            }
        }
        build_export_actions::BuildExportAction::Cancel { profile_name }
            if control_id == DESKTOP_EXPORT_CANCEL_BUTTON =>
        {
            BuildExportWizardSurfaceAction {
                profile_name,
                action: ExportWizardPanelAction::Cancel,
            }
        }
        _ => return None,
    };
    Some(action)
}

fn required_options(
    action: ExportWizardPanelAction,
    options: Option<ExportWizardPipelineOptions>,
) -> Result<ExportWizardPipelineOptions, ExportWizardPanelSessionError> {
    options.ok_or(ExportWizardPanelSessionError::ActionDisabled {
        action,
        reason: "pipeline options are required",
    })
}

fn export_wizard_job_id(profile_name: &str) -> String {
    format!("{EXPORT_WIZARD_JOB_ID_PREFIX}.{profile_name}")
}

fn export_wizard_status_message(profile_name: &str, update: &ExportWizardPanelUpdate) -> String {
    match update.action {
        ExportWizardPanelAction::GeneratePlan => {
            format!("Desktop export wizard plan for {profile_name} refreshed")
        }
        ExportWizardPanelAction::Start => {
            format!("Desktop export wizard {profile_name} started")
        }
        ExportWizardPanelAction::Cancel => {
            format!("Desktop export wizard {profile_name} cancel requested")
        }
        ExportWizardPanelAction::Poll => {
            format!(
                "Desktop export wizard {profile_name} {:?}",
                update.snapshot.status
            )
        }
    }
}

#[cfg(test)]
mod tests;
