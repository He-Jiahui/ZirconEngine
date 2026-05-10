use zircon_runtime_interface::ui::layout::UiSize;

use super::build_viewport_toolbar_pointer_layout_with_size::build_viewport_toolbar_pointer_layout_with_size;
use super::constants::{SURFACE_HEIGHT, SURFACE_WIDTH};
use super::viewport_toolbar_pointer_layout::ViewportToolbarPointerLayout;

#[cfg(test)]
pub(crate) fn build_viewport_toolbar_pointer_layout<I, S>(
    surface_keys: I,
) -> ViewportToolbarPointerLayout
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    build_viewport_toolbar_pointer_layout_with_size(
        surface_keys,
        UiSize::new(SURFACE_WIDTH, SURFACE_HEIGHT),
    )
}
