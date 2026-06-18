use super::*;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn handle_ui_asset_palette_drag_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        primary: &str,
        secondary: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match action_id {
            "palette.drag.hover" => {
                let surface_x = match primary.parse::<f32>() {
                    Ok(value) => value,
                    Err(error) => {
                        self.set_status_line(format!(
                            "Invalid UI asset palette drag hover x `{primary}`: {error}"
                        ));
                        return;
                    }
                };
                let surface_y = match secondary.parse::<f32>() {
                    Ok(value) => value,
                    Err(error) => {
                        self.set_status_line(format!(
                            "Invalid UI asset palette drag hover y `{secondary}`: {error}"
                        ));
                        return;
                    }
                };
                match self
                    .editor_manager
                    .update_ui_asset_editor_palette_drag_target(&instance_id, surface_x, surface_y)
                {
                    Ok(_) => self.mark_presentation_dirty(),
                    Err(error) => self.set_status_line(error.to_string()),
                }
            }
            other => {
                self.set_status_line(format!("Unknown UI asset palette drag action {other}"));
            }
        }
    }
}
