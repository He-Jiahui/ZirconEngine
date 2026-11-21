//! Standalone DOM focus regression; not part of the WebEngine product.
#![cfg(all(
    target_arch = "wasm32",
    not(feature = "native-host"),
    not(feature = "target-web")
))]

// Compile the actual listener against the production queue source without WebGPU.
extern crate self as zircon_runtime;
extern crate zircon_runtime as engine;

pub use engine::core;
#[path = "../../zircon_runtime/src/browser/input.rs"]
mod input_owner;
mod browser {
    pub use crate::input_owner::BrowserInput;
}
#[path = "../src/web/listeners.rs"]
mod listeners;

use core::framework::input::{InputButton, InputManager};
use engine::input::DefaultInputManager;
use input_owner::{BrowserInput, BrowserInputFrame};
use serde_json::json;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

#[wasm_bindgen]
pub struct WebFocusFixture {
    listeners: Option<listeners::Listeners>,
    ingress: Arc<BrowserInput>,
    events: BrowserInputFrame,
    manager: DefaultInputManager,
}

#[wasm_bindgen]
impl WebFocusFixture {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> Result<WebFocusFixture, JsValue> {
        let ingress = Arc::new(BrowserInput::default());
        let listeners = listeners::Listeners::new(canvas, Arc::clone(&ingress))
            .map_err(|error| JsValue::from_str(&error))?;
        Ok(Self {
            listeners: Some(listeners),
            ingress,
            events: BrowserInputFrame::default(),
            manager: DefaultInputManager::default(),
        })
    }

    pub fn step(&mut self) -> Result<(), JsValue> {
        if self.listeners.is_none() || !self.ingress.is_visible() {
            return Err(JsValue::from_str("fixture is closed or document is hidden"));
        }
        // Same ingress and manager submission as BrowserEngine, without UI/rendering.
        self.ingress
            .submit_frame(&mut self.events, &self.manager, |_| Ok(()))
            .map_err(|error| JsValue::from_str(&error))
    }

    pub fn snapshot(&self, key_code: u32, logical_key: &str) -> String {
        let frame = self.manager.frame_snapshot();
        let button = |id| {
            [
                frame.buttons.pressed(&id),
                frame.buttons.just_pressed(&id),
                frame.buttons.just_released(&id),
            ]
        };
        json!({
            "physical": button(InputButton::KeyCode(key_code)),
            "logical": button(InputButton::Key(logical_key.into())),
            "mouse_left": button(InputButton::MouseLeft),
            "cursor_requests": frame.cursor_host_requests.len(),
            "ime_requests": frame.ime_host_requests.len(),
            "omitted_events": self.ingress.omitted_events(),
        })
        .to_string()
    }

    pub fn close(&mut self) -> u32 {
        self.listeners.take();
        self.ingress.close();
        self.events.clear();
        listeners::live_listeners() as u32
    }
}
