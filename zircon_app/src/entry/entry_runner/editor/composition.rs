use std::error::Error;
use std::fmt::Display;
use std::path::Path;
use std::sync::Arc;

use zircon_editor::{
    ui::host::{EditorHostEventController, EditorHostStartupSession, EditorManager},
    EditorGuiStartupRequest, EDITOR_MANAGER_NAME,
};
use zircon_runtime::core::math::UVec2;
use zircon_runtime::core::CoreHandle;

use super::super::super::runtime_library::{LoadedRuntime, RuntimeSession};
use super::{
    editor_startup_diagnostic_error, prepare_editor_gui_startup, EditorStartupPreparation,
    EntryRunner,
};

/// Complete non-windowed editor composition for product authoring and integration hosts.
pub struct EditorApplicationComposition {
    host: EditorHostStartupSession,
    _runtime_session: Arc<RuntimeSession>,
    _core: CoreHandle,
}

impl EditorApplicationComposition {
    pub fn open_project(project_root: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let startup_request = EditorGuiStartupRequest::open_project(project_root.as_ref());
        let EditorStartupPreparation {
            entry_config,
            startup_request,
            prepared_project,
            editor_plugin_registrations,
            runtime_plugin_registrations,
            runtime_capabilities,
        } = prepare_editor_gui_startup(Some(startup_request))?;
        let prepared_project = prepared_project.ok_or_else(|| {
            std::io::Error::other("project composition preparation did not open its project")
        })?;
        let core = EntryRunner::bootstrap_with_runtime_plugin_registrations(
            entry_config,
            runtime_plugin_registrations.clone(),
        )?;
        let manager = core.resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)?;
        let runtime_library = LoadedRuntime::linked()?;
        let runtime_session = Arc::new(RuntimeSession::create_linked_with_profile_and_project(
            runtime_library,
            b"editor",
            None,
            runtime_plugin_registrations,
        )?);
        let runtime_teardown_failure = runtime_session.teardown_failure_state();
        let host_result: Result<_, Box<dyn Error>> = (|| {
            let runtime_gateway = runtime_session.editor_gateway(runtime_capabilities)?;
            let host = EditorHostStartupSession::open_with_prepared_project(
                manager,
                startup_request,
                Some(prepared_project),
                UVec2::new(1280, 720),
            )?;
            host.controller().attach_play_gateway(runtime_gateway)?;
            for registration in editor_plugin_registrations {
                host.controller()
                    .register_editor_plugin_registration(registration)?;
            }
            Ok(host)
        })();
        let host = match host_result {
            Ok(host) => host,
            Err(error) => {
                drop(runtime_session);
                return Err(editor_composition_open_error(
                    error,
                    runtime_teardown_failure.take(),
                ));
            }
        };

        Ok(Self {
            host,
            _runtime_session: runtime_session,
            _core: core,
        })
    }

    pub fn editor_host(&self) -> &EditorHostEventController {
        self.host.controller()
    }

    pub fn startup_session(
        &self,
    ) -> &zircon_editor::ui::workbench::startup::EditorStartupSessionDocument {
        self.host.startup_session()
    }

    /// Returns the runtime inspection generation activated while opening this project.
    pub fn opened_project_inspection_generation(&self) -> Option<u64> {
        self.host
            .startup_session()
            .project
            .as_ref()
            .map(|project| project.world.inspection_artifact().generation())
    }

    /// Releases every gateway owner and reports a runtime session teardown failure.
    pub fn close(self) -> Result<(), Box<dyn Error>> {
        let Self {
            host,
            _runtime_session: runtime_session,
            _core: core,
        } = self;
        let runtime_teardown_failure = runtime_session.teardown_failure_state();
        drop(host);
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

fn editor_composition_open_error<E>(
    open_error: Box<dyn Error>,
    teardown_failure: Option<E>,
) -> Box<dyn Error>
where
    E: Display,
{
    match teardown_failure {
        Some(teardown_failure) => format!(
            "editor composition initialization failed: {open_error}; runtime session teardown also failed: {teardown_failure}"
        )
        .into(),
        None => open_error,
    }
}

#[cfg(test)]
mod tests {
    use super::editor_composition_open_error;

    #[test]
    fn project_composition_propagates_gateway_attachment_failure() {
        let source = include_str!("composition.rs");

        assert!(
            source.contains("host.controller().attach_play_gateway(runtime_gateway)?;"),
            "project composition must not continue after a runtime gateway replacement failure"
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
            "drop(host);",
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

    #[test]
    fn project_composition_open_combines_initialization_and_teardown_failures() {
        let error = editor_composition_open_error(
            "runtime gateway attachment failed".into(),
            Some("runtime session destroy failed"),
        );

        assert_eq!(
            error.to_string(),
            "editor composition initialization failed: runtime gateway attachment failed; runtime session teardown also failed: runtime session destroy failed"
        );
    }

    #[test]
    fn project_composition_open_releases_partial_runtime_before_returning_error() {
        let source = include_str!("composition.rs");
        let open = source
            .split("pub fn open_project")
            .nth(1)
            .expect("project composition should expose project open");
        let mut offset = 0;
        for needle in [
            "let runtime_teardown_failure = runtime_session.teardown_failure_state();",
            "let host_result: Result<_, Box<dyn Error>> = (|| {",
            "runtime_session.editor_gateway(runtime_capabilities)?",
            "host.controller().attach_play_gateway(runtime_gateway)?;",
            "for registration in editor_plugin_registrations",
            "register_editor_plugin_registration(registration)?;",
            "Ok(host)",
            "let host = match host_result",
            "drop(runtime_session);",
            "editor_composition_open_error(error, runtime_teardown_failure.take())",
            "Ok(Self",
        ] {
            let index = open[offset..]
                .find(needle)
                .unwrap_or_else(|| panic!("composition open path is missing `{needle}`"));
            offset += index + needle.len();
        }
    }
}
