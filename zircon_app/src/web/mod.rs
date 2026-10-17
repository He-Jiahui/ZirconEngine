use std::sync::atomic::{AtomicU64, Ordering};
use wasm_bindgen::prelude::*;
use zircon_runtime::browser::BrowserEngine;

mod listeners;
static NEXT_CANVAS: AtomicU64 = AtomicU64::new(1);
#[wasm_bindgen]
pub struct WebEngine {
    engine: Option<BrowserEngine>,
    listeners: Option<listeners::Listeners>,
}
fn js_error(error: impl std::fmt::Display) -> JsValue {
    js_sys::Error::new(&error.to_string()).into()
}
#[wasm_bindgen]
impl WebEngine {
    /// Loads an immutable desktop-preprocessed bundle on the browser Window thread.
    pub async fn load(
        canvas: web_sys::HtmlCanvasElement,
        asset_root: String,
    ) -> Result<WebEngine, JsValue> {
        let identity = NEXT_CANVAS
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| js_error("Canvas identity space is exhausted"))?;
        let plan =
            zircon_plugin_animation_runtime::portable_runtime_extension_plan().map_err(js_error)?;
        let engine = BrowserEngine::open(
            canvas.clone(),
            identity,
            asset_root,
            zircon_plugin_animation_runtime::module_descriptor(),
            plan,
        )
        .await
        .map_err(js_error)?;
        let listeners = listeners::Listeners::new(canvas, engine.input()).map_err(js_error)?;
        Ok(Self {
            engine: Some(engine),
            listeners: Some(listeners),
        })
    }
    /// Await each frame before requesting another one.
    pub async fn render_frame(&mut self, timestamp_ms: f64) -> Result<String, JsValue> {
        let engine = self
            .engine
            .as_mut()
            .ok_or_else(|| js_error("browser engine is closed"))?;
        let receipt = engine.frame(timestamp_ms).await.map_err(js_error)?;
        serde_json::to_string(&receipt).map_err(js_error)
    }
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), JsValue> {
        self.engine
            .as_mut()
            .ok_or_else(|| js_error("browser engine is closed"))?
            .resize(width, height)
            .map_err(js_error)
    }
    /// Incomplete drains are errors; the retained owner permits a close retry.
    pub async fn close(&mut self) -> Result<String, JsValue> {
        self.listeners.take();
        let Some(engine) = self.engine.as_mut() else {
            return Ok(
                "{\"modules_closed\":true,\"task_graph_joined\":true,\"pending_assets\":0}".into(),
            );
        };
        let receipt = engine.close().await.map_err(js_error)?;
        self.engine.take();
        serde_json::to_string(&receipt).map_err(js_error)
    }
}
/// The optional heap counters measure requested Rust bytes, not Wasm linear capacity.
#[wasm_bindgen]
pub fn browser_memory_snapshot() -> String {
    let mut snapshot = serde_json::json!({"live_engines":zircon_runtime::browser::live_browser_engines(),
        "event_listeners":listeners::live_listeners(), "registered_canvases":zircon_runtime::browser::registered_browser_canvases()});
    #[cfg(feature = "memory-validation")]
    {
        snapshot["rust_heap"] =
            serde_json::to_value(zircon_runtime::browser::browser_heap_snapshot())
                .expect("heap counters serialize");
    }
    snapshot.to_string()
}
impl Drop for WebEngine {
    fn drop(&mut self) {
        self.listeners.take();
        self.engine.take();
    }
}
