use std::error::Error;
use std::path::Path;
use std::sync::Arc;

use zircon_editor::{
    core::play::{EmbeddedPlayBackend, SharedPlayBackend},
    run_retained_host_automation, EditorGuiStartupRequest, EditorHostRunConfig,
    RetainedHostAutomationResult,
};
use zircon_runtime::asset::project::ResolvedProjectPath;
use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;

use crate::entry::ProductComposition;

use super::super::super::runtime_library::{
    LoadedRuntime, RuntimeLibraryPreflight, RuntimeSession,
};
use super::play_session_factory::AppPlaySessionFactory;
use super::{
    application_open_project_intent, finish_editor_host, prepare_editor_gui_startup,
    prepare_editor_gui_startup_with_resolved_project, record_editor_host_failure,
    EditorStartupPreparation, EntryRunner,
};

/// Complete non-windowed editor composition for product authoring and integration hosts.
#[must_use = "call close or run_retained_host_automation to observe teardown failures"]
pub struct EditorApplicationComposition {
    startup_request: Option<EditorGuiStartupRequest>,
    editor_plugin_registrations: Vec<zircon_editor::EditorPluginRegistrationReport>,
    runtime_capabilities: zircon_editor::RuntimeCapabilities,
    project_runtime_build_set: ZrRuntimeBuildSetId,
    product_composition: ProductComposition,
    runtime_session: Arc<RuntimeSession>,
    play_backend: SharedPlayBackend,
}

impl EditorApplicationComposition {
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
        let product_composition = EntryRunner::compose_resolved_with_runtime_plugin_registrations(
            entry_config,
            runtime_plugin_registrations,
        )?;
        let project_runtime_build_set = runtime_preflight.build_set_id();
        let play_backend = Arc::new(EmbeddedPlayBackend::new(Arc::new(
            AppPlaySessionFactory::new(runtime_preflight.clone(), runtime_capabilities.clone()),
        ))) as SharedPlayBackend;
        let runtime_library = runtime_preflight.load_after_preflight()?;
        let runtime_session = Arc::new(RuntimeSession::create_with_profile(
            runtime_library,
            b"editor",
        )?);
        Ok(Self {
            startup_request,
            editor_plugin_registrations,
            runtime_capabilities,
            project_runtime_build_set,
            product_composition,
            runtime_session,
            play_backend,
        })
    }

    /// Transfers bootstrap ownership into the editor's production retained-host automation path.
    pub fn run_retained_host_automation(
        self,
        bindings: &[zircon_editor::ui::binding::EditorUiBinding],
    ) -> Result<RetainedHostAutomationResult, Box<dyn Error>> {
        let Self {
            startup_request,
            editor_plugin_registrations,
            runtime_capabilities,
            project_runtime_build_set,
            runtime_session,
            play_backend,
            product_composition,
        } = self;
        let core = product_composition.core().clone();
        let runtime_teardown_failure = runtime_session.teardown_failure_state();
        let product_failure_ledger = runtime_teardown_failure.failure_ledger();
        let result = (|| {
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
        drop(product_composition);
        drop(runtime_session);
        finish_editor_host(
            "editor_application_composition",
            result,
            product_failure_ledger.snapshot(),
        )
    }

    /// Releases every gateway owner and reports a runtime session teardown failure.
    pub fn close(self) -> Result<(), Box<dyn Error>> {
        let Self {
            startup_request: _,
            editor_plugin_registrations: _,
            runtime_capabilities: _,
            project_runtime_build_set: _,
            runtime_session,
            play_backend: _,
            product_composition,
        } = self;
        let runtime_teardown_failure = runtime_session.teardown_failure_state();
        let product_failure_ledger = runtime_teardown_failure.failure_ledger();
        drop(product_composition);
        drop(runtime_session);
        finish_editor_host(
            "editor_application_composition",
            Ok(()),
            product_failure_ledger.snapshot(),
        )
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
    fn project_composition_preflights_the_dynamic_runtime_before_project_materialization() {
        let source = include_str!("composition.rs");
        let product_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("production composition source must precede its tests");
        let runtime_preflight = product_source
            .find("LoadedRuntime::preflight_default()")
            .expect("composition must preflight the staged runtime BuildSet");
        let project_prepare = product_source
            .find("prepare_editor_gui_startup(Some(startup_request))?")
            .expect("composition must prepare its project after runtime preflight");

        assert!(runtime_preflight < project_prepare);
        assert!(product_source.contains("runtime_preflight.load_after_preflight()?"));
        assert!(!product_source.contains("LoadedRuntime::linked()?"));
        assert!(!product_source.contains("create_linked_with_profile_and_project("));
        assert!(product_source.contains("RuntimeSession::create_with_profile("));
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
            "let product_failure_ledger = runtime_teardown_failure.failure_ledger();",
            "drop(product_composition);",
            "drop(runtime_session);",
            "finish_editor_host(",
            "product_failure_ledger.snapshot()",
        ] {
            let index = close[offset..]
                .find(needle)
                .unwrap_or_else(|| panic!("composition close path is missing `{needle}`"));
            offset += index + needle.len();
        }
    }

    #[test]
    fn default_drop_releases_product_composition_before_runtime_session() {
        let source = include_str!("composition.rs");
        let fields = source
            .split("pub struct EditorApplicationComposition")
            .nth(1)
            .and_then(|body| body.split("impl EditorApplicationComposition").next())
            .expect("editor application composition fields must precede the impl");

        let product_composition = fields
            .find("product_composition:")
            .expect("composition must own the App product composition");
        let runtime_session = fields
            .find("runtime_session:")
            .expect("composition must own the dynamic runtime session");
        assert!(
            product_composition < runtime_session,
            "default field drop must release Core/plugin owners before the dynamic runtime session"
        );
        assert!(source.contains("#[must_use = \"call close or run_retained_host_automation"));
    }
}
