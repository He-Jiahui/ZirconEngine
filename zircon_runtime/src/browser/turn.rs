use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_futures::JsFuture;

/// One RAII timer per awaiting close operation; cancellation removes the callback.
pub(super) async fn next_turn() -> Result<(), String> {
    let window = web_sys::window().ok_or_else(|| "browser Window is unavailable".to_string())?;
    let mut callback: Option<Closure<dyn FnMut()>> = None;
    let mut timeout = None;
    let mut schedule_error = None;
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let function = Closure::wrap(Box::new(move || {
            let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
        }) as Box<dyn FnMut()>);
        match window.set_timeout_with_callback_and_timeout_and_arguments_0(
            function.as_ref().unchecked_ref(),
            8,
        ) {
            Ok(id) => timeout = Some(id),
            Err(error) => schedule_error = Some(format!("{error:?}")),
        }
        callback = Some(function);
    });
    struct Timer {
        window: web_sys::Window,
        timeout: Option<i32>,
        _callback: Option<Closure<dyn FnMut()>>,
    }
    impl Drop for Timer {
        fn drop(&mut self) {
            if let Some(id) = self.timeout {
                self.window.clear_timeout_with_handle(id);
            }
        }
    }
    let _timer = Timer {
        window,
        timeout,
        _callback: callback,
    };
    if let Some(error) = schedule_error {
        return Err(error);
    }
    JsFuture::from(promise)
        .await
        .map(|_| ())
        .map_err(|error| format!("{error:?}"))
}
