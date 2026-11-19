//! Opt-in test package; never linked into the WebEngine product.
#![cfg(all(
    target_arch = "wasm32",
    not(feature = "native-host"),
    not(feature = "target-web")
))]

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
use serde_json::{json, Map, Value};
use std::sync::Arc;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Element, HtmlCanvasElement};

const QA_CANVAS: &str = "zircon-key-fixture";

#[wasm_bindgen]
pub struct WebListenerFixture {
    listeners: Option<listeners::Listeners>,
    ingress: Arc<BrowserInput>,
    frame: BrowserInputFrame,
    manager: DefaultInputManager,
    steps: u8,
}

#[wasm_bindgen]
impl WebListenerFixture {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WebListenerFixture, JsValue> {
        if listeners::live_listeners() != 0 {
            return Err(error("a listener fixture is already mounted"));
        }
        let doc = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| error("QA document unavailable"))?;
        let canvas = doc
            .get_element_by_id(QA_CANVAS)
            .and_then(|element| element.dyn_into::<HtmlCanvasElement>().ok())
            .ok_or_else(|| error("QA canvas unavailable"))?;
        canvas.set_tab_index(0);
        canvas.focus()?;
        if doc.hidden() || doc.active_element().as_ref().map(Element::id).as_deref() != Some(QA_CANVAS)
        {
            return Err(error("QA canvas must be visible and focused"));
        }
        let ingress = Arc::new(BrowserInput::default());
        let listeners = listeners::Listeners::new(canvas, Arc::clone(&ingress)).map_err(error)?;
        Ok(Self {
            listeners: Some(listeners),
            ingress,
            frame: BrowserInputFrame::default(),
            manager: DefaultInputManager::default(),
            steps: 0,
        })
    }

    pub fn step(&mut self) -> Result<String, JsValue> {
        if self.listeners.is_none() || self.steps >= 8 || !self.ingress.is_visible() {
            return Err(error("fixture is closed, hidden, or over its frame budget"));
        }
        self.steps += 1;
        let mut raw = Vec::new();
        // Same pre-manager stage as the product UI, without rendering or key rewriting.
        self.ingress
            .submit_frame(&mut self.frame, &self.manager, |event| {
                raw.push(event.clone());
                Ok(())
            })
            .map_err(error)?;
        Ok(json!({
            "listener": {
                "source": "rust-browser-input-pre-manager",
                "events": raw,
                "omitted_events": self.ingress.omitted_events(),
            },
            "manager": query(&self.manager),
        })
        .to_string())
    }

    pub fn close(&mut self) -> bool {
        self.listeners.take();
        self.ingress.close();
        self.frame.clear();
        listeners::live_listeners() == 0
    }
}

fn query(manager: &DefaultInputManager) -> Value {
    let frame = manager.frame_snapshot();
    let buttons: Map<String, Value> = [
        ("A", InputButton::Key("A".into())),
        ("a", InputButton::Key("a".into())),
        ("Shift", InputButton::Key("Shift".into())),
        ("1", InputButton::Key("1".into())),
        ("16", InputButton::KeyCode(16)),
        ("49", InputButton::KeyCode(49)),
        ("65", InputButton::KeyCode(65)),
        ("97", InputButton::KeyCode(97)),
    ]
    .into_iter()
    .map(|(label, button)| {
        (
            label.into(),
            json!([
                manager.button_pressed(&button),
                frame.buttons.just_pressed(&button),
                frame.buttons.just_released(&button),
            ]),
        )
    })
    .collect();
    json!({"source": "rust-default-input-manager", "buttons": buttons})
}

fn error(message: impl AsRef<str>) -> JsValue {
    JsValue::from_str(message.as_ref())
}
