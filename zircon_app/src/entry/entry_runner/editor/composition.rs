use std::error::Error;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use zircon_editor::{
    core::play::{EmbeddedPlayBackend, SharedPlayBackend},
    run_retained_host_automation, EditorGuiStartupRequest, EditorHostRunConfig,
    RetainedHostAutomationResult,
};
use zircon_runtime::asset::project::ResolvedProjectPath;

use super::ownership::{close_deadline, finish_owned_editor_host, EditorApplicationOwnership};

use super::super::super::runtime_library::{
    LoadedRuntime, RuntimeLibraryPreflight, RuntimeSession,
};
use super::play_session_factory::AppPlaySessionFactory;
use super::{
    application_open_project_intent, prepare_editor_gui_startup,
    prepare_editor_gui_startup_with_resolved_project, record_editor_host_failure,
    EditorStartupPreparation, EntryRunner,
};

/// Complete non-windowed editor composition for product authoring and integration hosts.
#[must_use = "call close or run_retained_host_automation to observe teardown failures"]
pub struct EditorApplicationComposition {
    ownership: Option<EditorApplicationOwnership>,
}

pub(super) struct ProductWorkbenchCaptureResult {
    pub(super) build_set_id: String,
    pub(super) snapshots_output_path: Option<PathBuf>,
    pub(super) visual_evidence: Option<zircon_editor::ZuiVisualEvidenceSummary>,
}

impl EditorApplicationComposition {
    fn take(mut self) -> EditorApplicationOwnership {
        self.ownership
            .take()
            .expect("Editor ownership transfers once")
    }

    pub fn open_project(project_root: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let runtime_preflight = LoadedRuntime::preflight_default()?;
        let startup_request = EditorGuiStartupRequest::project(application_open_project_intent(
            project_root.as_ref(),
        )?);
        Self::from_startup_preparation(
            prepare_editor_gui_startup(Some(startup_request))?,
            runtime_preflight,
        )
    }

    /// Opens a project from the physical identity already resolved by a process entry boundary.
    pub fn open_resolved_project(
        project_root: ResolvedProjectPath,
    ) -> Result<Self, Box<dyn Error>> {
        let runtime_preflight = LoadedRuntime::preflight_default()?;
        Self::from_startup_preparation(
            prepare_editor_gui_startup_with_resolved_project(project_root)?,
            runtime_preflight,
        )
    }

    fn from_startup_preparation(
        prepared_startup: EditorStartupPreparation,
        runtime_preflight: RuntimeLibraryPreflight,
    ) -> Result<Self, Box<dyn Error>> {
        let EditorStartupPreparation {
            entry_config,
            startup_request,
            editor_plugin_registrations,
            runtime_plugin_registrations,
            runtime_capabilities,
            ..
        } = prepared_startup;
        let mut product_composition =
            EntryRunner::compose_resolved_with_runtime_plugin_registrations(
                entry_config,
                runtime_plugin_registrations.iter().cloned(),
            )?;
        product_composition.retain_plugin_selection_outcomes(
            runtime_plugin_registrations
                .outcomes()
                .iter()
                .chain(editor_plugin_registrations.outcomes().iter())
                .cloned(),
        );
        let project_runtime_build_set = runtime_preflight.build_set_id();
        let play_backend = Arc::new(EmbeddedPlayBackend::new(Arc::new(
            AppPlaySessionFactory::new(runtime_preflight.clone(), runtime_capabilities.clone()),
        ))) as SharedPlayBackend;
        let runtime_library = match runtime_preflight.load_after_preflight() {
            Ok(library) => library,
            Err(primary) => {
                return Err(Box::new(
                    product_composition.fail_until(primary, close_deadline()),
                ))
            }
        };
        let runtime_session = match RuntimeSession::create_with_profile(runtime_library, b"editor")
        {
            Ok(session) => Arc::new(session),
            Err(failure) => {
                return Err(Box::new(
                    product_composition.fail_with_runtime_until(failure, close_deadline()),
                ))
            }
        };
        Ok(Self {
            ownership: Some(EditorApplicationOwnership {
                startup_request,
                editor_plugin_registrations: editor_plugin_registrations.into_iter().collect(),
                runtime_capabilities,
                project_runtime_build_set,
                product_composition,
                runtime_session,
                play_backend,
            }),
        })
    }

    /// Transfers bootstrap ownership into the editor's production retained-host automation path.
    pub fn run_retained_host_automation(
        self,
        bindings: &[zircon_editor::ui::binding::EditorUiBinding],
    ) -> Result<RetainedHostAutomationResult, Box<dyn Error>> {
        let EditorApplicationOwnership {
            startup_request,
            editor_plugin_registrations,
            runtime_capabilities,
            project_runtime_build_set,
            runtime_session,
            play_backend,
            product_composition,
        } = self.take();
        let core = product_composition.core().clone();
        let runtime_teardown_failure = runtime_session.teardown_failure_state();
        let product_failure_ledger = runtime_teardown_failure.failure_ledger();
        let retained_play_backend = play_backend.clone();
        let result: Result<_, Box<dyn Error + Send + Sync>> = (|| {
            let runtime_gateway = runtime_session.editor_gateway(runtime_capabilities)?;
            let config = EditorHostRunConfig::new()
                .with_startup_request(startup_request)
                .with_project_runtime_build_set(project_runtime_build_set)
                .with_play_backend(play_backend)
                .with_editor_plugin_registrations(editor_plugin_registrations);
            run_retained_host_automation(core.clone(), runtime_gateway, config, bindings)
        })();
        record_editor_host_failure(&product_failure_ledger, &result);
        drop(core);
        finish_owned_editor_host(
            "editor_application_composition",
            result,
            product_composition,
            runtime_session,
            retained_play_backend,
            &product_failure_ledger,
            close_deadline(),
        )
    }

    /// Exports product workbench states and/or native evidence through this App's preflighted
    /// runtime identity and live Core. The RuntimeSession remains alive until both operations
    /// finish and normal composition teardown has been checked.
    pub(super) fn run_product_workbench_capture(
        self,
        repo_root: &Path,
        snapshots_output_path: Option<&Path>,
        capture_zui_visual_evidence: bool,
    ) -> Result<ProductWorkbenchCaptureResult, Box<dyn Error>> {
        let EditorApplicationOwnership {
            startup_request: _,
            editor_plugin_registrations: _,
            runtime_capabilities: _,
            project_runtime_build_set,
            runtime_session,
            play_backend,
            product_composition,
        } = self.take();
        let build_set_id = project_runtime_build_set.as_str().to_owned();
        let core = product_composition.core().clone();
        let runtime_teardown_failure = runtime_session.teardown_failure_state();
        let product_failure_ledger = runtime_teardown_failure.failure_ledger();
        let capture_result: Result<ProductWorkbenchCaptureResult, Box<dyn Error + Send + Sync>> =
            (|| {
                if let Some(output_path) = snapshots_output_path {
                    zircon_editor::export_zui_workbench_product_snapshots_with_context(
                        repo_root,
                        output_path,
                        &core,
                        &project_runtime_build_set,
                    )
                    .map_err(io::Error::other)?;
                }
                let visual_evidence = if capture_zui_visual_evidence {
                    Some(
                        zircon_editor::export_zui_visual_evidence_with_context(
                            repo_root,
                            &core,
                            &project_runtime_build_set,
                        )
                        .map_err(io::Error::other)?,
                    )
                } else {
                    None
                };
                Ok(ProductWorkbenchCaptureResult {
                    build_set_id,
                    snapshots_output_path: snapshots_output_path.map(Path::to_path_buf),
                    visual_evidence,
                })
            })();
        record_editor_host_failure(&product_failure_ledger, &capture_result);
        drop(core);
        finish_owned_editor_host(
            "editor_application_composition_product_capture",
            capture_result,
            product_composition,
            runtime_session,
            play_backend,
            &product_failure_ledger,
            close_deadline(),
        )
    }

    /// Releases every gateway owner and reports a runtime session teardown failure.
    pub fn close(self) -> Result<(), Box<dyn Error>> {
        let EditorApplicationOwnership {
            startup_request: _,
            editor_plugin_registrations: _,
            runtime_capabilities: _,
            project_runtime_build_set: _,
            runtime_session,
            play_backend,
            product_composition,
        } = self.take();
        let runtime_teardown_failure = runtime_session.teardown_failure_state();
        let product_failure_ledger = runtime_teardown_failure.failure_ledger();
        finish_owned_editor_host(
            "editor_application_composition",
            Ok(()),
            product_composition,
            runtime_session,
            play_backend,
            &product_failure_ledger,
            close_deadline(),
        )
    }
}

#[cfg(test)]
#[path = "tests/composition.rs"]
mod tests;

impl Drop for EditorApplicationComposition {
    fn drop(&mut self) {
        if let Some(ownership) = self.ownership.take() {
            ownership.retain_unclosed();
        }
    }
}
