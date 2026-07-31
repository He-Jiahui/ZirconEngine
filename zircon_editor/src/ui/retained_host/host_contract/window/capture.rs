use std::fs;
use std::path::PathBuf;

use zircon_runtime::diagnostic_log::write_log;

use super::UiHostWindow;
use crate::ui::retained_host::primitives::PlatformError;

impl UiHostWindow {
    /// Saves the host presentation only after a native presenter reports success.
    pub(in crate::ui::retained_host::host_contract) fn capture_first_presented_frame(
        &self,
    ) -> Result<Option<PathBuf>, PlatformError> {
        let Some(path) = self
            .state
            .borrow_mut()
            .first_presented_frame_capture_path
            .take()
        else {
            return Ok(None);
        };
        let snapshot = self.window().take_snapshot()?;
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|error| {
                PlatformError::Other(format!(
                    "failed to create editor first-frame capture directory '{}': {error}",
                    parent.display()
                ))
            })?;
        }
        image::save_buffer_with_format(
            &path,
            snapshot.as_bytes(),
            snapshot.width(),
            snapshot.height(),
            image::ColorType::Rgba8,
            image::ImageFormat::Png,
        )
        .map_err(|error| {
            PlatformError::Other(format!(
                "failed to write editor first-frame capture '{}': {error}",
                path.display()
            ))
        })?;
        write_log(
            "editor_host_window",
            format!(
                "editor_product_frame_capture_written path={}",
                path.display()
            ),
        );
        Ok(Some(path))
    }
}
