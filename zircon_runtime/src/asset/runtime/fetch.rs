//! Bounded streaming Fetch. Known-sized blocks go directly into their final Arc allocation.
use crate::core::runtime::TaskCancellationToken;
use std::mem::MaybeUninit;
use std::sync::Arc;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
struct FetchGuard {
    controller: web_sys::AbortController,
    interval: i32,
    _callback: Closure<dyn FnMut()>,
}
impl Drop for FetchGuard {
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            window.clear_interval_with_handle(self.interval);
        }
        self.controller.abort();
    }
}
fn js_error(value: JsValue) -> String {
    value.as_string().unwrap_or_else(|| format!("{value:?}"))
}
pub(crate) async fn fetch_bytes(
    url: &str,
    expected_bytes: Option<u64>,
    max_bytes: u64,
    cancellation: &TaskCancellationToken,
) -> Result<Arc<[u8]>, String> {
    if max_bytes == 0 || expected_bytes.is_some_and(|n| n > max_bytes) {
        return Err("Fetch byte limit exceeded".into());
    }
    if cancellation.is_cancellation_requested() {
        return Err("Fetch cancelled".into());
    }
    let window = web_sys::window().ok_or("browser Window unavailable")?;
    let controller = web_sys::AbortController::new().map_err(js_error)?;
    let token = cancellation.clone();
    let abort = controller.clone();
    let started = web_time::Instant::now();
    let callback = Closure::new(move || {
        if token.is_cancellation_requested() || started.elapsed().as_secs() >= 60 {
            abort.abort();
        }
    });
    let interval = window
        .set_interval_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            16,
        )
        .map_err(js_error)?;
    let guard = FetchGuard {
        controller,
        interval,
        _callback: callback,
    };
    let options = web_sys::RequestInit::new();
    options.set_signal(Some(&guard.controller.signal()));
    let request = web_sys::Request::new_with_str_and_init(url, &options).map_err(js_error)?;
    let response: web_sys::Response = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(js_error)?
        .dyn_into()
        .map_err(|value| js_error(value.into()))?;
    if !response.ok() {
        return Err(format!("Fetch HTTP {} for {url}", response.status()));
    }
    // Fetch exposes the decoded HTTP body. Content-Length may describe gzip/brotli
    // bytes, so enforce the descriptor's bound on streamed decoded bytes below.
    let expected = expected_bytes
        .map(usize::try_from)
        .transpose()
        .map_err(|_| "Fetch size exceeds address space")?;
    let limit = usize::try_from(max_bytes).map_err(|_| "Fetch limit exceeds address space")?;
    let mut exact: Option<Arc<[MaybeUninit<u8>]>> = expected.map(Arc::<[u8]>::new_uninit_slice);
    let mut variable: Vec<u8> = Vec::new();
    let mut written = 0_usize;
    let body = response
        .body()
        .ok_or("Fetch response has no streaming body")?;
    let reader: web_sys::ReadableStreamDefaultReader = body
        .get_reader()
        .dyn_into()
        .map_err(|value| js_error(value.into()))?;
    loop {
        if cancellation.is_cancellation_requested() {
            return Err("Fetch cancelled".into());
        }
        let part = JsFuture::from(reader.read()).await.map_err(js_error)?;
        if js_sys::Reflect::get(&part, &JsValue::from_str("done"))
            .map_err(js_error)?
            .as_bool()
            == Some(true)
        {
            break;
        }
        let value = js_sys::Reflect::get(&part, &JsValue::from_str("value")).map_err(js_error)?;
        let chunk = js_sys::Uint8Array::new(&value);
        let count = chunk.length() as usize;
        let next = written.checked_add(count).ok_or("Fetch length overflow")?;
        if next > expected.unwrap_or(limit) {
            return Err("Fetch body exceeds declared limit".into());
        }
        if let Some(exact) = exact.as_mut() {
            chunk.copy_to_uninit(
                &mut Arc::get_mut(exact).expect("unpublished Fetch buffer")[written..next],
            );
        } else {
            if next > variable.capacity() {
                let target = variable.capacity().saturating_mul(2).max(next).min(limit);
                variable
                    .try_reserve_exact(target - variable.len())
                    .map_err(|_| "Fetch allocation refused")?;
            }
            chunk.copy_to_uninit(&mut variable.spare_capacity_mut()[..count]);
            // SAFETY: copy_to_uninit initialized exactly count new bytes, after checked bounds.
            unsafe {
                variable.set_len(next);
            }
        }
        written = next;
    }
    reader.release_lock();
    if cancellation.is_cancellation_requested() {
        return Err("Fetch cancelled".into());
    }
    if let Some(exact) = exact {
        if written != expected.unwrap() {
            return Err("Fetch body size does not match descriptor".into());
        }
        // SAFETY: no clone escaped and every byte in the exact-length slice was initialized.
        Ok(unsafe { exact.assume_init() })
    } else {
        Ok(variable.into())
    }
}
