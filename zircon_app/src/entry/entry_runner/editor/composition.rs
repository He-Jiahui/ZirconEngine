use std::error::Error;
use std::path::Path;
use std::sync::Arc;

use zircon_editor::{
    ui::host::{EditorHostEventController, EditorHostStartupSession, EditorManager},
    EditorGuiStartupRequest, EDITOR_MANAGER_NAME,
};
use zircon_runtime::core::math::UVec2;
use zircon_runtime::core::CoreHandle;

use super::super::super::runtime_library::{LoadedRuntime, RuntimeSession};
use super::{prepare_editor_gui_startup, EditorStartupPreparation, EntryRunner};

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
        let runtime_gateway = runtime_session.editor_gateway(runtime_capabilities)?;
        let host = EditorHostStartupSession::open_with_prepared_project(
            manager,
            startup_request,
            Some(prepared_project),
            UVec2::new(1280, 720),
        )?;
        host.controller().set_runtime_gateway(runtime_gateway)?;
        for registration in editor_plugin_registrations {
            host.controller()
                .register_editor_plugin_registration(registration)?;
        }

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
}

#[cfg(test)]
mod tests {
    #[test]
    fn project_composition_propagates_gateway_attachment_failure() {
        let source = include_str!("composition.rs");

        assert!(
            source.contains("host.controller().set_runtime_gateway(runtime_gateway)?;"),
            "project composition must not continue after a runtime gateway replacement failure"
        );
    }
}
