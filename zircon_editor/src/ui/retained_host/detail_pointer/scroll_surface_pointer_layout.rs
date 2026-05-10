use zircon_runtime_interface::ui::layout::UiSize;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScrollSurfacePointerLayout {
    pub(super) pane_size: UiSize,
    pub(super) viewport_origin_y: f32,
    pub(super) content_extent: f32,
}
