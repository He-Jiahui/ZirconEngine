use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

use super::constants::SURFACE_VERTICAL_STRIDE;
use super::viewport_toolbar_pointer_layout::ViewportToolbarPointerLayout;
use super::viewport_toolbar_pointer_surface::ViewportToolbarPointerSurface;

pub(crate) fn build_viewport_toolbar_pointer_layout_with_size<I, S>(
    surface_keys: I,
    surface_size: UiSize,
) -> ViewportToolbarPointerLayout
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    ViewportToolbarPointerLayout {
        surfaces: surface_keys
            .into_iter()
            .enumerate()
            .map(|(index, key)| ViewportToolbarPointerSurface {
                key: key.as_ref().to_string(),
                frame: UiFrame::new(
                    0.0,
                    index as f32 * SURFACE_VERTICAL_STRIDE,
                    surface_size.width.max(1.0),
                    surface_size.height.max(1.0),
                ),
            })
            .collect(),
    }
}
