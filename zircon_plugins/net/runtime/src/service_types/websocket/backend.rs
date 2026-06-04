use std::sync::Arc;

use zircon_runtime::core::framework::net::NetError;

use crate::websocket::WebSocketRuntimeBackend;

use super::super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn websocket_backend(
        &self,
    ) -> Result<Arc<dyn WebSocketRuntimeBackend>, NetError> {
        self.state
            .websocket_backend
            .lock()
            .expect("net WebSocket backend mutex poisoned")
            .clone()
            .ok_or_else(|| NetError::ProtocolUnavailable {
                capability: "runtime.feature.net.websocket".to_string(),
            })
    }
}
