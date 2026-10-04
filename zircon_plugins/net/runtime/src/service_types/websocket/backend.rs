//! 从根 manager 共享状态取出可选 WebSocket 后端，并把未安装后端转换为稳定能力错误。
//! 后端回调必须在 mutex 之外运行，以允许重入 manager。

use std::sync::Arc;

use zircon_runtime::core::framework::net::NetError;

use crate::poison_recovery::{lock_or_error, NetSharedState};
use crate::websocket::WebSocketRuntimeBackend;

use super::super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn websocket_backend(
        &self,
    ) -> Result<Arc<dyn WebSocketRuntimeBackend>, NetError> {
        lock_or_error(
            &self.state.websocket_backend,
            NetSharedState::WebSocketBackend,
        )?
        .clone()
        .ok_or_else(|| NetError::ProtocolUnavailable {
            capability: "runtime.feature.net.websocket".to_string(),
        })
    }
}
