use crate::ui::workbench::snapshot::ViewContentKind;
use zircon_runtime_interface::ui::layout::UiSize;

use super::super::super::RetainedEditorHost;

mod asset_surface;
mod frame;
mod host_frames;
mod workbench_regions;

use asset_surface::asset_surface_kind;
use frame::is_valid_size;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn resolve_callback_surface_size_for_kind(
        &self,
        width: f32,
        height: f32,
        cached_size: UiSize,
        kind: ViewContentKind,
    ) -> UiSize {
        let callback_size = UiSize::new(width.max(0.0), height.max(0.0));
        if is_valid_size(callback_size) {
            return callback_size;
        }
        if is_valid_size(cached_size) {
            return cached_size;
        }

        self.resolve_host_frame_backed_size_for_kind(kind)
            .unwrap_or(UiSize::new(0.0, 0.0))
    }

    pub(in crate::ui::retained_host::app) fn resolve_callback_surface_size_for_asset_surface(
        &self,
        surface_mode: &str,
        width: f32,
        height: f32,
        cached_size: UiSize,
    ) -> Option<UiSize> {
        asset_surface_kind(surface_mode).map(|kind| {
            self.resolve_callback_surface_size_for_kind(width, height, cached_size, kind)
        })
    }
}
