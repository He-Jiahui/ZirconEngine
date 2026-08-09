use zircon_runtime::rhi::UiSurfacePresenter;

use super::super::super::diagnostics::HostRefreshDiagnostics;
use super::geometry::clamp_size;
use super::GpuChromePresenter;
use crate::ui::retained_host::host_contract::presenter::error::HostPresenterResult;

impl<P: UiSurfacePresenter> GpuChromePresenter<P> {
    pub(in crate::ui::retained_host::host_contract) fn new(surface: P, size: (u32, u32)) -> Self {
        Self {
            surface,
            size: clamp_size(size),
            diagnostics: HostRefreshDiagnostics::default(),
            last_upload_bytes: 0,
            last_draw_calls: 0,
            surface_cache_initialized: false,
            native_resize_projection_size: clamp_size(size),
            native_resize_draw_list: None,
            #[cfg(test)]
            native_resize_snapshot_build_count: 0,
            #[cfg(test)]
            native_resize_snapshot_reuse_count: 0,
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn resize(
        &mut self,
        size: (u32, u32),
    ) -> HostPresenterResult<()> {
        let size = clamp_size(size);
        if self.native_resize_draw_list.is_none() {
            self.native_resize_projection_size = (
                self.native_resize_projection_size
                    .0
                    .max(self.size.0)
                    .max(size.0),
                self.native_resize_projection_size
                    .1
                    .max(self.size.1)
                    .max(size.1),
            );
        }
        self.surface.resize(size.0, size.1)?;
        self.size = size;
        self.surface_cache_initialized = false;
        Ok(())
    }

    pub(in crate::ui::retained_host::host_contract) fn diagnostics_snapshot(
        &self,
    ) -> HostRefreshDiagnostics {
        self.diagnostics.clone()
    }

    pub(in crate::ui::retained_host::host_contract) fn last_upload_bytes(&self) -> u64 {
        self.last_upload_bytes
    }

    pub(in crate::ui::retained_host::host_contract) fn last_draw_calls(&self) -> u64 {
        self.last_draw_calls
    }
}
