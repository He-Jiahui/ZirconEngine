mod attempts;
mod bitmap;
mod hash;
mod http_fetch;
mod manifest;
mod progress;
mod resume;
mod state;

use std::sync::Arc;

use zircon_runtime::core::framework::net::NetManager;

pub use state::NetContentDownloadRuntimeManager;

const HTTP_PARTIAL_CONTENT_STATUS: u16 = 206;
const HTTP_SUCCESS_STATUS: u16 = 200;
const CONTENT_DOWNLOAD_HTTP_TIMEOUT_MS: u64 = 30_000;
const CONTENT_DOWNLOAD_HTTP_RETRY_ATTEMPTS: u8 = 1;

#[derive(Debug)]
pub(in crate::manager) struct FetchAttemptResponse {
    pub(in crate::manager) status_code: u16,
    pub(in crate::manager) body: Vec<u8>,
}

impl FetchAttemptResponse {
    pub(in crate::manager) fn status_code_is_successful(&self) -> bool {
        matches!(
            self.status_code,
            HTTP_SUCCESS_STATUS | HTTP_PARTIAL_CONTENT_STATUS
        )
    }
}

impl NetContentDownloadRuntimeManager {
    pub(in crate::manager) fn state(
        &self,
    ) -> std::sync::MutexGuard<'_, state::NetContentDownloadRuntimeState> {
        self.state
            .lock()
            .expect("net content download state mutex poisoned")
    }

    pub(in crate::manager) fn net(&self) -> Option<&Arc<dyn NetManager>> {
        self.net.as_ref()
    }
}

pub fn net_content_download_runtime_manager() -> NetContentDownloadRuntimeManager {
    NetContentDownloadRuntimeManager::new()
}
