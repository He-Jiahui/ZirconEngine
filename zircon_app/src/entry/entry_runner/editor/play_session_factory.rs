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
    capabilities: RuntimeCapabilities,
}

impl AppPlaySessionFactory {
    pub(super) fn new(
        runtime_preflight: RuntimeLibraryPreflight,
        capabilities: RuntimeCapabilities,
    ) -> Self {
        let capabilities = RuntimeCapabilities::new(
            SessionProfileKind::Runtime,
            capabilities.core_capabilities().iter().cloned(),
            capabilities.plugin_summary().iter().cloned(),
        );
        Self {
            runtime_preflight,
            capabilities,
        }
    }
}

impl PlaySessionFactory for AppPlaySessionFactory {
    fn create(
        &self,
        request: &PlaySessionLaunchRequest,
    ) -> Result<Box<dyn PlaySessionLease>, String> {
        let runtime = self
            .runtime_preflight
            .load_after_preflight()
            .map_err(|error| error.to_string())?;
        let session = Arc::new(
            RuntimeSession::create_with_profile_and_project(
                runtime,
                b"runtime",
                Some(request.project_root()),
                Some(request.scene()),
                None,
                None,
            )
            .map_err(|error| error.to_string())?,
        );
        let gateway = session
            .editor_gateway(self.capabilities.clone())
            .map_err(|error| error.to_string())?;
        Ok(Box::new(AppPlaySessionLease {
            gateway: Some(gateway),
            session: Some(session),
        }))
    }
}

struct AppPlaySessionLease {
    gateway: Option<SharedEditorRuntimeGateway>,
    session: Option<Arc<RuntimeSession>>,
}

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
