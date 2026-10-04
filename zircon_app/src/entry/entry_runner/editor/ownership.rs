use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant};

use zircon_editor::{core::play::SharedPlayBackend, EditorGuiStartupRequest};
use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;

use crate::entry::product_shutdown::retained_owner::{ProductCompositionFailure, RetainedPacket};
use crate::entry::product_shutdown::{
    ProductFailureLedger, ProductFailureSeverity, ProductHostPhase,
};
use crate::entry::runtime_library::RuntimeSession;
use crate::entry::ProductComposition;

pub(super) struct EditorApplicationOwnership {
    pub(super) startup_request: Option<EditorGuiStartupRequest>,
    pub(super) editor_plugin_registrations: Vec<zircon_editor::EditorPluginRegistrationReport>,
    pub(super) runtime_capabilities: zircon_editor::RuntimeCapabilities,
    pub(super) project_runtime_build_set: ZrRuntimeBuildSetId,
    pub(super) product_composition: ProductComposition,
    pub(super) runtime_session: Arc<RuntimeSession>,
    pub(super) play_backend: SharedPlayBackend,
}

pub(super) fn close_deadline() -> Instant {
    Instant::now() + Duration::from_secs(5)
}

impl EditorApplicationOwnership {
    pub(super) fn retain_unclosed(self) {
        let packet = self
            .product_composition
            .into_packet()
            .with_runtime(self.runtime_session)
            .with_host_pin(self.play_backend);
        let _ = ProductCompositionFailure::retained(
            Arc::new(std::io::Error::other(
                "Editor composition dropped without an explicit close receipt",
            )),
            packet,
        );
    }
}

/// The same packet survives an ordinary host failure and every incomplete close stage.
/// Temporary managers and gateways must have left the host call before this boundary.
pub(super) fn finish_owned_editor_host<T>(
    requested: &str,
    host_result: Result<T, Box<dyn Error + Send + Sync>>,
    product: ProductComposition,
    session: Arc<RuntimeSession>,
    play_backend: SharedPlayBackend,
    failures: &ProductFailureLedger,
    deadline: Instant,
) -> Result<T, Box<dyn Error>> {
    let mut packet: RetainedPacket = product
        .into_packet()
        .with_runtime(session)
        .with_host_pin(play_backend);
    match host_result {
        Err(primary) => {
            let failure = ProductCompositionFailure::owned(Arc::from(primary), packet, deadline);
            if let Ok((Some(secondary), _)) = failure.cleanup_observation() {
                failures.record(
                    ProductHostPhase::DestroyingRuntime,
                    ProductFailureSeverity::Terminal,
                    "product_composition",
                    secondary,
                );
            }
            Err(Box::new(failure))
        }
        Ok(value) => match packet.close_until(deadline) {
            Err(primary) => {
                failures.record(
                    ProductHostPhase::DestroyingRuntime,
                    ProductFailureSeverity::Terminal,
                    "product_composition",
                    &primary,
                );
                Err(Box::new(ProductCompositionFailure::retained(
                    Arc::new(primary),
                    packet,
                )))
            }
            Ok(_) => super::finish_editor_host(requested, Ok(value), failures.snapshot()),
        },
    }
}
