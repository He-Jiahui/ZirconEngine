use super::super::viewport_overlay_renderer::ViewportOverlayRenderer;

impl ViewportOverlayRenderer {
    #[cfg(test)]
    pub(crate) fn pass_order() -> &'static [&'static str] {
        super::super::super::PASS_ORDER
    }
}
