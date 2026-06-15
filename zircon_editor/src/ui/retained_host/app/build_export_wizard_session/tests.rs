use super::*;
use crate::ui::host::{
    ExportWizardCommandExecution, ExportWizardCommandRunner, ExportWizardJobStatus,
    ExportWizardPipelineStageCommand, DESKTOP_EXPORT_START_BUTTON,
};
use zircon_runtime::plugin::ExportPipelineStage;

#[derive(Clone, Copy, Debug, Default)]
struct ImmediateSuccessRunner;

impl ExportWizardCommandRunner for ImmediateSuccessRunner {
    fn run(
        &mut self,
        _command: &ExportWizardPipelineStageCommand,
    ) -> Result<ExportWizardCommandExecution, String> {
        Ok(ExportWizardCommandExecution {
            exit_code: Some(0),
            stdout_lines: Vec::new(),
            stderr_lines: Vec::new(),
        })
    }
}

#[test]
fn build_export_wizard_surface_action_maps_panel_buttons_to_session_actions() {
    assert_eq!(
        build_export_wizard_surface_action(
            DESKTOP_EXPORT_GENERATE_PLAN_BUTTON,
            "workbench.build_export.plan.desktop_windows",
        ),
        Some(BuildExportWizardSurfaceAction {
            profile_name: "desktop_windows",
            action: ExportWizardPanelAction::GeneratePlan,
        })
    );
    assert_eq!(
        build_export_wizard_surface_action(
            DESKTOP_EXPORT_START_BUTTON,
            "workbench.build_export.execute.desktop_windows",
        ),
        Some(BuildExportWizardSurfaceAction {
            profile_name: "desktop_windows",
            action: ExportWizardPanelAction::Start,
        })
    );
    assert_eq!(
        build_export_wizard_surface_action(
            DESKTOP_EXPORT_CANCEL_BUTTON,
            "workbench.build_export.cancel.desktop_windows",
        ),
        Some(BuildExportWizardSurfaceAction {
            profile_name: "desktop_windows",
            action: ExportWizardPanelAction::Cancel,
        })
    );
    assert_eq!(
        build_export_wizard_surface_action(
            DESKTOP_EXPORT_START_BUTTON,
            "workbench.build_export.output.choose.desktop_windows",
        ),
        None
    );
}

#[test]
fn desktop_export_wizard_sessions_project_view_model_after_generate_plan() {
    let mut sessions = DesktopExportWizardSessions::default();
    let update = sessions
        .dispatch_profile_action(
            "desktop_windows",
            ExportWizardPanelAction::GeneratePlan,
            Some(ready_options("desktop_windows")),
        )
        .expect("generate plan should create an inactive session");

    assert_eq!(update.action, ExportWizardPanelAction::GeneratePlan);
    assert_eq!(update.snapshot.profile, "desktop_windows");
    assert_eq!(update.snapshot.status, ExportWizardJobStatus::Pending);

    let view_model = sessions
        .view_model("desktop_windows")
        .expect("generate plan should store a view model for the profile");
    assert!(view_model.plan_ready());
    assert!(view_model.controls().can_start);
    assert!(sessions.poll_all().is_empty());
}

#[test]
fn desktop_export_wizard_sessions_start_refreshes_existing_plan_options() {
    let mut sessions = DesktopExportWizardSessions::default();
    let first_options = ready_options_with_out("desktop_windows", "D:\\zircon-export-old");
    let second_options = ready_options_with_out("desktop_windows", "D:\\zircon-export-new");

    sessions
        .dispatch_profile_action(
            "desktop_windows",
            ExportWizardPanelAction::GeneratePlan,
            Some(first_options.clone()),
        )
        .expect("generate plan should create the first inactive session");
    assert_eq!(
        sessions
            .sessions
            .get("desktop_windows")
            .expect("profile session should exist")
            .plan()
            .out,
        first_options.out
    );

    let start_update = sessions
        .dispatch_profile_action_with_runner(
            "desktop_windows",
            ExportWizardPanelAction::Start,
            Some(second_options.clone()),
            ImmediateSuccessRunner,
        )
        .expect("start should refresh the existing plan before launching");

    assert_eq!(start_update.action, ExportWizardPanelAction::Start);
    assert_eq!(start_update.snapshot.out, second_options.out);
    let session = sessions
        .sessions
        .get_mut("desktop_windows")
        .expect("profile session should remain stored after start");
    assert_eq!(session.plan().out, second_options.out);
    assert!(session
        .plan()
        .command(ExportPipelineStage::CookAssets)
        .expect("cook assets stage should exist")
        .consumed_artifacts
        .iter()
        .any(|artifact| artifact.path == "D:\\zircon-export-new\\assets\\assets.json"));
    assert!(session
        .plan()
        .command(ExportPipelineStage::PlatformBundle)
        .expect("platform bundle stage should exist")
        .consumed_artifacts
        .iter()
        .any(|artifact| artifact.path == "D:\\zircon-export-new\\host\\zircon_game.exe"));
    assert_eq!(
        session
            .finish_job()
            .expect("test export job should join cleanly")
            .expect("test export job should have started")
            .status,
        ExportWizardJobStatus::Finished
    );
}

#[test]
fn export_wizard_default_host_executable_points_to_compile_host_output() {
    let profile = build_export_actions::desktop_export_profile("desktop_windows")
        .expect("desktop_windows profile should exist");
    let host = export_wizard_default_host_executable("D:\\zircon-export", &profile, None);

    assert!(host.contains("stages"));
    assert!(host.contains("compile_host"));
    assert!(host.contains("target"));
    assert!(host.contains("debug"));
    assert!(host.ends_with(&format!("zircon_runtime{}", std::env::consts::EXE_SUFFIX)));
    assert!(!host.contains("zircon_game"));
}

#[test]
fn export_wizard_engine_repo_root_contains_python_module_entrypoint() {
    let repo_root = export_wizard_engine_repo_root();

    assert!(
        repo_root
            .join("zircon_export")
            .join("__main__.py")
            .is_file(),
        "expected repo root {:?} to contain zircon_export/__main__.py",
        repo_root
    );
}

fn ready_options(profile_name: &str) -> ExportWizardPipelineOptions {
    ready_options_with_out(profile_name, "D:\\zircon-export")
}

fn ready_options_with_out(profile_name: &str, out: &str) -> ExportWizardPipelineOptions {
    let mut options = ExportWizardPipelineOptions::new(profile_name, "zircon-project.toml", out);
    options.source_asset_manifest = Some(format!("{out}\\assets\\assets.json"));
    options.host_executable = Some(format!("{out}\\host\\zircon_game.exe"));
    options
}
