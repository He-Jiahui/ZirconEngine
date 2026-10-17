use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use wasm_bindgen::{closure::Closure, JsCast};
use zircon_runtime::{
    browser::BrowserInput,
    core::framework::input::{InputButton, InputEvent, TouchPhase},
};
static LIVE_LISTENERS: AtomicU64 = AtomicU64::new(0);
pub(super) fn live_listeners() -> u64 {
    LIVE_LISTENERS.load(Ordering::Acquire)
}
struct Listener {
    target: web_sys::EventTarget,
    name: &'static str,
    callback: Closure<dyn FnMut(web_sys::Event)>,
}
impl Drop for Listener {
    fn drop(&mut self) {
        let _ = self
            .target
            .remove_event_listener_with_callback(self.name, self.callback.as_ref().unchecked_ref());
        LIVE_LISTENERS.fetch_sub(1, Ordering::AcqRel);
    }
}
pub(super) struct Listeners {
    _listeners: Vec<Listener>,
}
impl Listeners {
    pub(super) fn new(
        canvas: web_sys::HtmlCanvasElement,
        input: Arc<BrowserInput>,
    ) -> Result<Self, String> {
        let window = web_sys::window().ok_or_else(|| "browser Window unavailable".to_string())?;
        let document = window
            .document()
            .ok_or_else(|| "browser Document unavailable".to_string())?;
        canvas.set_tab_index(0);
        let target: web_sys::EventTarget = canvas.clone().unchecked_into();
        let mut listeners = Self {
            _listeners: Vec::with_capacity(12),
        };
        for name in ["pointerdown", "pointerup", "pointermove", "pointercancel"] {
            let canvas = canvas.clone();
            let input = Arc::clone(&input);
            listeners.add(target.clone(), name, move |event| {
                let Some(event) = event.dyn_ref::<web_sys::PointerEvent>() else {
                    return;
                };
                let bounds = canvas.get_bounding_client_rect();
                let x = ((event.client_x() as f64 - bounds.left()) * canvas.width() as f64
                    / bounds.width().max(1.0)) as f32;
                let y = ((event.client_y() as f64 - bounds.top()) * canvas.height() as f64
                    / bounds.height().max(1.0)) as f32;
                if name == "pointerdown" {
                    let _ = canvas.focus();
                    let _ = canvas.set_pointer_capture(event.pointer_id());
                }
                if event.pointer_type() == "touch" {
                    let phase = match name {
                        "pointerdown" => TouchPhase::Started,
                        "pointerup" => TouchPhase::Ended,
                        "pointercancel" => TouchPhase::Cancelled,
                        _ => TouchPhase::Moved,
                    };
                    input.push(InputEvent::Touch {
                        id: event.pointer_id().unsigned_abs() as u64,
                        phase,
                        x,
                        y,
                    });
                } else {
                    input.push(InputEvent::CursorMoved { x, y });
                    let button = match event.button() {
                        0 => InputButton::MouseLeft,
                        1 => InputButton::MouseMiddle,
                        2 => InputButton::MouseRight,
                        3 => InputButton::MouseBack,
                        4 => InputButton::MouseForward,
                        other => InputButton::MouseOther(other.max(0) as u16),
                    };
                    match name {
                        "pointerdown" => input.push(InputEvent::ButtonPressed(button)),
                        "pointerup" => input.push(InputEvent::ButtonReleased(button)),
                        "pointercancel" => input.push(InputEvent::FocusLost),
                        _ => input.push(InputEvent::MouseMotion {
                            delta_x: event.movement_x() as f32,
                            delta_y: event.movement_y() as f32,
                        }),
                    }
                }
                if name == "pointerup" || name == "pointercancel" {
                    let _ = canvas.release_pointer_capture(event.pointer_id());
                }
                event.prevent_default();
            })?;
        }
        for name in ["keydown", "keyup"] {
            let input = Arc::clone(&input);
            listeners.add(target.clone(), name, move |event| {
                let Some(event) = event.dyn_ref::<web_sys::KeyboardEvent>() else {
                    return;
                };
                let key = event.key();
                let text = if name == "keydown" && !event.is_composing() && key.chars().count() == 1
                {
                    Some(key.clone())
                } else {
                    None
                };
                input.push(InputEvent::KeyboardInput {
                    key_code: event.key_code(),
                    logical_key: Some(key),
                    text,
                    pressed: name == "keydown",
                    repeat: event.repeat(),
                });
                event.prevent_default();
            })?;
        }
        let wheel = Arc::clone(&input);
        listeners.add(target.clone(), "wheel", move |event| {
            if let Some(event) = event.dyn_ref::<web_sys::WheelEvent>() {
                wheel.push(InputEvent::WheelScrolled {
                    delta: -(event.delta_y() as f32) / 100.0,
                });
                event.prevent_default();
            }
        })?;
        for (name, event) in [
            ("pointerenter", InputEvent::CursorEntered),
            ("pointerleave", InputEvent::CursorLeft),
        ] {
            let input = Arc::clone(&input);
            listeners.add(target.clone(), name, move |_| input.push(event.clone()))?;
        }
        let blur = Arc::clone(&input);
        listeners.add(window.clone().unchecked_into(), "blur", move |_| {
            blur.push(InputEvent::FocusLost)
        })?;
        let visibility = Arc::clone(&input);
        let visibility_document = document.clone();
        listeners.add(
            document.clone().unchecked_into(),
            "visibilitychange",
            move |_| visibility.set_visible(!visibility_document.hidden()),
        )?;
        input.set_visible(!document.hidden());
        Ok(listeners)
    }
    fn add(
        &mut self,
        target: web_sys::EventTarget,
        name: &'static str,
        callback: impl FnMut(web_sys::Event) + 'static,
    ) -> Result<(), String> {
        let callback = Closure::wrap(Box::new(callback) as Box<dyn FnMut(web_sys::Event)>);
        let options = web_sys::AddEventListenerOptions::new();
        options.set_passive(false);
        target
            .add_event_listener_with_callback_and_add_event_listener_options(
                name,
                callback.as_ref().unchecked_ref(),
                &options,
            )
            .map_err(|error| format!("{error:?}"))?;
        LIVE_LISTENERS.fetch_add(1, Ordering::AcqRel);
        self._listeners.push(Listener {
            target,
            name,
            callback,
        });
        Ok(())
    }
}
