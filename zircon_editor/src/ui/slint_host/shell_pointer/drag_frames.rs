use zircon_runtime::ui::layout::UiFrame;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct DragTargetFrames {
    pub(super) left: UiFrame,
    pub(super) right: UiFrame,
    pub(super) bottom: UiFrame,
    pub(super) document: UiFrame,
}
