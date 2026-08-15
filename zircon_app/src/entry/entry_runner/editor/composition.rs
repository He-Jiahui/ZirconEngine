use std::error::Error;
use std::path::Path;
use std::sync::Arc;

use zircon_editor::{
    run_retained_host_automation, EditorGuiStartupRequest, EditorHostRunConfig,
    RetainedHostAutomationResult,
};
use zircon_runtime::asset::project::{ProjectManager, ResolvedProjectPath};
use zircon_runtime::core::CoreHandle;

use super::super::super::runtime_library::{LoadedRuntime, RuntimeSession};
use super::{
    editor_startup_diagnostic_error, prepare_editor_gui_startup,
    prepare_editor_gui_startup_with_resolved_project, EditorStartupPreparation, EntryRunner,
};

/// Complete non-windowed editor composition for product authoring and integration hosts.
pub struct EditorApplicationComposition {
    startup_request: Option<EditorGuiStartupRequest>,
    prepared_project: ProjectManager,
    editor_plugin_registrations: Vec<zircon_editor::EditorPluginRegistrationReport>,
    runtime_capabilities: zircon_editor::RuntimeCapabilities,
    runtime_session: Arc<RuntimeSession>,
    core: CoreHandle,
}

impl EditorApplicationComposition {
    pub fn open_project(project_root: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let startup_request = EditorGuiStartupRequest::open_project(project_root.as_ref());
        Self::from_prepared_startup(prepare_editor_gui_startup(Some(startup_request))?)
    }

    /// Opens a project from the physical identity already resolved by a process entry boundary.
    pub fn open_resolved_project(
        project_root: ResolvedProjectPath,
    ) -> Result<Self, Box<dyn Error>> {
        Self::from_prepared_startup(prepare_editor_gui_startup_with_resolved_project(
            project_root,
        )?)
    }

    fn from_prepared_startup(
        prepared_startup: EditorStartupPreparation,
    ) -> Result<Self, Box<dyn Error>> {
        let EditorStartupPreparation {
            entry_config,
            startup_request,
            prepared_project,
            editor_plugin_registrations,
            runtime_plugin_registrations,
            runtime_capabilities,
            ..
        } = prepared_startup;
        let prepared_project = prepared_project.ok_or_else(|| {
            std::io::Error::other("project composition preparation did not open its project")
        })?;
        let core = EntryRunner::bootstrap_with_runtime_plugin_registrations(
            entry_config,
            runtime_plugin_registrations.clone(),
        )?;
        let runtime_library = LoadedRuntime::linked()?;
        let runtime_session = Arc::new(RuntimeSession::create_linked_with_profile_and_project(
            runtime_library,
            b"editor",
            None,
            runtime_plugin_registrations,
        )?);
        Ok(Self {
            startup_request,
            prepared_project,
            editor_plugin_registrations,
            runtime_capabilities,
            runtime_session,
            core,
        })
    }

    pub fn prepared_project(&self) -> &ProjectManager {
        &self.prepared_project
    }

    /// Transfers bootstrap ownership into the editor's production retained-host automation path.
    pub fn run_retained_host_automation(
        self,
        bindings: &[zircon_editor::ui::binding::EditorUiBinding],
    ) -> Result<RetainedHostAutomationResult, Box<dyn Error>> {
        let Self {
            startup_request,
            prepared_project,
            editor_plugin_registrations,
            runtime_capabilities,
            runtime_session,
            core,
        } = self;
        let runtime_teardown_failure = runtime_session.teardown_failure_state();
        let result = (|| {
            let runtime_gateway = runtime_session.editor_gateway(runtime_capabilities)?;
            let config = EditorHostRunConfig::new()
                .with_startup_request(startup_request)
                .with_prepared_project(Some(prepared_project))
                .with_editor_plugin_registrations(editor_plugin_registrations);
            run_retained_host_automation(core.clone(), runtime_gateway, config, bindings)
        })();
        drop(core);
        drop(runtime_session);
        match (result, runtime_teardown_failure.take()) {
            (Ok(result), None) => Ok(result),
            (Ok(_), Some(teardown_failure)) => Err(editor_startup_diagnostic_error(
                "runtime_session",
                "editor_application_composition",
                format!("runtime session teardown failed: {teardown_failure}"),
                "close the retained editor host before releasing the linked runtime session, then verify its lifecycle",
            )
            .into()),
            (Err(error), None) => Err(error),
            (Err(error), Some(teardown_failure)) => Err(format!(
                "retained-host automation failed: {error}; runtime session teardown also failed: {teardown_failure}"
            )
            .into()),
        }
    }

    /// Releases every gateway owner and reports a runtime session teardown failure.
    pub fn close(self) -> Result<(), Box<dyn Error>> {
        let Self {
            startup_request: _,
            prepared_project: _,
            editor_plugin_registrations: _,
            runtime_capabilities: _,
            runtime_session,
            core,
        } = self;
        let runtime_teardown_failure = runtime_session.teardown_failure_state();
        drop(core);
        drop(runtime_session);
        if let Some(error) = runtime_teardown_failure.take() {
            return Err(editor_startup_diagnostic_error(
                "runtime_session",
                "editor_application_composition",
                format!("runtime session teardown failed: {error}"),
                "close the editor composition after releasing all borrowed editor state, then verify the linked runtime session lifecycle",
            )
            .into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn project_composition_transfers_the_gateway_to_the_retained_host_runner() {
        let source = include_str!("composition.rs");

        assert!(
            source.contains(
                "let runtime_gateway = runtime_session.editor_gateway(runtime_capabilities)?;"
            ),
            "composition must create the runtime gateway before entering the retained host"
        );
        assert!(
            source.contains(
                "run_retained_host_automation(core.clone(), runtime_gateway, config, bindings)"
            ),
            "composition must transfer automation to zircon_editor's retained host"
        );
    }

    #[test]
    fn project_composition_close_releases_gateway_owners_before_checking_teardown() {
        let source = include_str!("composition.rs");
        let close = source
            .split("pub fn close(self)")
            .nth(1)
            .expect("project composition should expose explicit close");
        let mut offset = 0;
        for needle in [
            "let runtime_teardown_failure = runtime_session.teardown_failure_state();",
            "drop(core);",
            "drop(runtime_session);",
            "if let Some(error) = runtime_teardown_failure.take()",
            "Ok(())",
        ] {
            let index = close[offset..]
                .find(needle)
                .unwrap_or_else(|| panic!("composition close path is missing `{needle}`"));
            offset += index + needle.len();
        }
    }
}
