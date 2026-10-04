use super::HubRuntimeSession;
use crate::error::HubError;
use crate::state::{HubMessage, HubMessageId, ShellMessageId, TaskOperationKind, TaskStatus};
use crate::tauri_app::view_model::HubViewModel;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::tauri_app) struct NormalWindowGeometry {
    pub position_x: i32,
    pub position_y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::tauri_app) struct WindowGeometrySample {
    pub normal: Option<NormalWindowGeometry>,
    pub maximized: Option<bool>,
}

impl HubRuntimeSession {
    pub(in crate::tauri_app) fn persist_window_geometry(
        &mut self,
        sample: WindowGeometrySample,
    ) -> Result<bool, HubError> {
        let had_close_save_error = self.window_close_save_error.is_some();
        let previous = self.config.window.clone();
        if let Some(normal) = sample.normal {
            self.config.window.position_x = Some(normal.position_x);
            self.config.window.position_y = Some(normal.position_y);
            self.config.window.width = Some(normal.width);
            self.config.window.height = Some(normal.height);
        }
        if let Some(maximized) = sample.maximized {
            self.config.window.maximized = maximized;
        }
        if self.config.window == previous {
            self.window_close_save_error = None;
            return Ok(had_close_save_error);
        }
        if let Err(error) = self.persist_config() {
            // An unchanged follow-up event must still be able to retry the failed write.
            self.config.window = previous;
            return Err(error);
        }
        self.window_close_save_error = None;
        Ok(had_close_save_error)
    }

    pub(in crate::tauri_app) fn report_window_geometry_save_failure(
        &mut self,
        error: &HubError,
    ) -> HubViewModel {
        self.window_close_save_error = Some(
            TaskStatus::error(
                "Save Hub state failed",
                HubMessage::raw_text(error.to_string()),
                HubMessage::new(HubMessageId::Shell(ShellMessageId::CheckConfigPath)),
            )
            .with_operation(TaskOperationKind::Hub, "Hub state"),
        );
        self.publish_view_model()
    }
}

#[cfg(test)]
#[path = "tests/window_geometry.rs"]
mod tests;
