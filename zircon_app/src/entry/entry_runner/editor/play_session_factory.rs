use std::sync::Arc;

use super::super::super::runtime_library::{RuntimeLibraryPreflight, RuntimeSession};
use zircon_editor::core::gateway::{
    RuntimeCapabilities, SessionProfileKind, SharedEditorRuntimeGateway,
};
use zircon_editor::core::play::{
    PlaySessionFactory, PlaySessionLaunchRequest, PlaySessionLease, PlaySessionRetireReport,
};

pub(super) struct AppPlaySessionFactory {
    runtime_preflight: RuntimeLibraryPreflight,
    #[cfg(not(feature = "editor-scripting"))]
    capabilities: RuntimeCapabilities,
}

impl AppPlaySessionFactory {
    pub(super) fn new(
        runtime_preflight: RuntimeLibraryPreflight,
        capabilities: RuntimeCapabilities,
    ) -> Self {
        #[cfg(not(feature = "editor-scripting"))]
        let capabilities = RuntimeCapabilities::new(
            SessionProfileKind::Runtime,
            capabilities.core_capabilities().iter().cloned(),
            capabilities.plugin_summary().iter().cloned(),
        );
        #[cfg(feature = "editor-scripting")]
        let _ = capabilities;
        Self {
            runtime_preflight,
            #[cfg(not(feature = "editor-scripting"))]
            capabilities,
        }
    }
}

impl PlaySessionFactory for AppPlaySessionFactory {
    fn create(
        &self,
        request: &PlaySessionLaunchRequest,
    ) -> Result<Box<dyn PlaySessionLease>, String> {
        eprintln!("mvp_play_boundary app_factory_entered");
        #[cfg(feature = "editor-scripting")]
        let (session, capabilities) = self.create_linked(request)?;
        #[cfg(not(feature = "editor-scripting"))]
        let (session, capabilities) = {
            let runtime = self
                .runtime_preflight
                .load_after_preflight()
                .map_err(|error| error.to_string())?;
            eprintln!("mvp_play_boundary app_factory_runtime_loaded");
            let session = Arc::new(
                RuntimeSession::create_with_profile_and_project(
                    runtime,
                    b"runtime",
                    Some(request.project_root()),
                    Some(request.scene()),
                    None,
                    None,
                )
                .map_err(|error| error.diagnostic_with_recovery())?,
            );
            (session, self.capabilities.clone())
        };
        eprintln!("mvp_play_boundary app_factory_session_created");
        let gateway = session
            .editor_gateway(capabilities)
            .map_err(|error| error.to_string())?;
        eprintln!("mvp_play_boundary app_factory_gateway_created");
        Ok(Box::new(AppPlaySessionLease {
            gateway: Some(gateway),
            session: Some(session),
        }))
    }
}

#[cfg(feature = "editor-scripting")]
impl AppPlaySessionFactory {
    fn create_linked(
        &self,
        request: &PlaySessionLaunchRequest,
    ) -> Result<(Arc<RuntimeSession>, RuntimeCapabilities), String> {
        use zircon_runtime::asset::project::ProjectManifest;
        use zircon_runtime::builtin::manifest_with_mode_baseline;
        use zircon_runtime::core::framework::platform::RuntimeTargetMode;

        // Resolve anew for this project and the runtime target. EditorHost-only
        // registrations must not silently become active gameplay providers.
        let project = ProjectManifest::load(request.project_root().join("zircon-project.toml"))
            .map_err(|error| error.to_string())?;
        let manifest =
            manifest_with_mode_baseline(RuntimeTargetMode::ClientRuntime, Some(&project.plugins));
        let mut registrations =
            crate::entry::first_party_runtime_plugin_registrations_for_manifest(
                RuntimeTargetMode::ClientRuntime,
                &manifest,
            )
            .into_registrations_if_required_resolved_where(|selection| {
                !selection.is_runtime_builtin_domain()
            })
            .map_err(|error| error.to_string())?;
        // Resolution returns only enabled, target-matching project requests, but
        // each provider report still carries its descriptor's enablement default.
        // Admit those resolved providers without widening their supported targets.
        for registration in registrations.iter_mut() {
            registration.project_selection.enabled = true;
        }
        let capabilities = RuntimeCapabilities::from_runtime_plugin_registrations(
            SessionProfileKind::Runtime,
            &registrations,
        );
        let session = RuntimeSession::create_linked_play(
            &self.runtime_preflight,
            request.project_root(),
            request.scene(),
            registrations.into_iter().collect(),
        )
        .map_err(|error| error.diagnostic_with_recovery())?;
        eprintln!("mvp_play_boundary app_factory_linked_session_created");
        Ok((Arc::new(session), capabilities))
    }
}

struct AppPlaySessionLease {
    gateway: Option<SharedEditorRuntimeGateway>,
    session: Option<Arc<RuntimeSession>>,
}

#[cfg(test)]
#[path = "tests/play_session_factory.rs"]
mod tests;

impl PlaySessionLease for AppPlaySessionLease {
    fn gateway(&self) -> SharedEditorRuntimeGateway {
        self.gateway
            .as_ref()
            .expect("a live App Play lease must retain its gateway")
            .clone()
    }

    fn retire(&mut self) -> Result<PlaySessionRetireReport, String> {
        drop(self.gateway.take());
        let session = self
            .session
            .take()
            .ok_or_else(|| "App Play session lease is already retired".to_string())?;
        let mut session = match Arc::try_unwrap(session) {
            Ok(session) => session,
            Err(session) => {
                let owners = Arc::strong_count(&session);
                self.session = Some(session);
                return Err(format!(
                    "App Play session still has {owners} owners after gateway detach"
                ));
            }
        };
        if let Err(error) = session.try_destroy() {
            self.session = Some(Arc::new(session));
            return Err(error.to_string());
        }
        Ok(PlaySessionRetireReport {
            diagnostics: vec!["embedded.session=retired".to_string()],
        })
    }
}
