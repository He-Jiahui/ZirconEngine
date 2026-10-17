//! Native validation resolves inline. Browser scopes are retained in a bounded device batch:
//! asynchronous startup and frame entry points await that batch before reporting success.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn finish(device: &wgpu::Device, scope: wgpu::ErrorScopeGuard) -> Option<String> {
    let _ = device;
    pollster::block_on(scope.pop()).map(|error| error.to_string())
}

#[cfg(target_arch = "wasm32")]
mod web {
    use std::{cell::RefCell, future::Future, pin::Pin};
    const CAPACITY: usize = 256;
    struct Pending {
        device: wgpu::Device,
        result: Pin<Box<dyn Future<Output = Option<wgpu::Error>>>>,
    }
    thread_local! { static PENDING: RefCell<Vec<Pending>> = const { RefCell::new(Vec::new()) }; }
    pub(crate) fn finish(device: &wgpu::Device, scope: wgpu::ErrorScopeGuard) -> Option<String> {
        let result = Box::pin(scope.pop());
        PENDING.with(|pending| {
            let mut pending = pending.borrow_mut();
            if pending.len() >= CAPACITY {
                return Some("browser GPU validation batch capacity reached".into());
            }
            pending.push(Pending {
                device: device.clone(),
                result,
            });
            None
        })
    }
    pub(crate) async fn resolve(
        device: &wgpu::Device,
    ) -> Result<(), crate::graphics::GraphicsError> {
        let batch = PENDING.with(|pending| {
            let mut pending = pending.borrow_mut();
            let mut batch = Vec::new();
            let mut index = 0;
            while index < pending.len() {
                if &pending[index].device == device {
                    batch.push(pending.swap_remove(index));
                } else {
                    index += 1;
                }
            }
            batch
        });
        let mut first_error = None;
        // Await all receipts, including after an error, so no stale device validation survives.
        for pending in batch {
            if let Some(error) = pending.result.await {
                if first_error.is_none() {
                    first_error = Some(error.to_string());
                }
            }
        }
        match first_error {
            Some(error) => Err(crate::graphics::GraphicsError::WgpuValidation(error)),
            None => Ok(()),
        }
    }
    pub(crate) fn cancel(device: &wgpu::Device) {
        PENDING.with(|pending| pending.borrow_mut().retain(|entry| &entry.device != device));
    }
}
#[cfg(target_arch = "wasm32")]
pub(crate) use web::{cancel, finish, resolve};
