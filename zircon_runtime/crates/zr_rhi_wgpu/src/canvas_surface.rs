//! Explicit browser ownership for retained UI surfaces.
use std::cell::RefCell;
use std::collections::HashMap;
use web_sys::HtmlCanvasElement;
use zr_rhi::{RenderNativeSurfaceTarget, RhiError};
thread_local! { static CANVASES: RefCell<HashMap<u64, HtmlCanvasElement>> = RefCell::new(HashMap::new()); }
/// Keep this lease beside the surface owner; dropping it releases the registry's DOM reference.
/// WGPU independently retains the Canvas while its surface remains alive.
pub struct CanvasSurfaceRegistration {
    identity: u64,
}
impl CanvasSurfaceRegistration {
    pub const fn identity(&self) -> u64 {
        self.identity
    }
}
pub fn register_canvas_surface(
    target: RenderNativeSurfaceTarget,
    canvas: HtmlCanvasElement,
) -> Result<CanvasSurfaceRegistration, RhiError> {
    let RenderNativeSurfaceTarget::Canvas { identity } = target else {
        return Err(RhiError::SurfaceUnavailable(
            "expected Canvas surface target".to_owned(),
        ));
    };
    CANVASES.with(|entries| {
        let mut entries = entries.borrow_mut();
        if identity == 0 || entries.len() >= 64 || entries.contains_key(&identity) {
            return Err(RhiError::SurfaceUnavailable(
                "Canvas surface capacity reached or identity already registered".to_owned(),
            ));
        }
        entries.insert(identity, canvas);
        Ok(CanvasSurfaceRegistration { identity })
    })
}
pub(crate) fn canvas_for_surface(identity: u64) -> Result<HtmlCanvasElement, RhiError> {
    CANVASES
        .with(|entries| entries.borrow().get(&identity).cloned())
        .ok_or_else(|| {
            RhiError::SurfaceUnavailable("Canvas surface registration has been released".to_owned())
        })
}
pub fn registered_canvas_surface_count() -> usize {
    CANVASES.with(|entries| entries.borrow().len())
}
impl Drop for CanvasSurfaceRegistration {
    fn drop(&mut self) {
        CANVASES.with(|entries| {
            entries.borrow_mut().remove(&self.identity);
        });
    }
}
