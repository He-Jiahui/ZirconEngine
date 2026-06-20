use super::super::super::{close_prompt, RetainedEditorHost};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::native_window_close) fn save_dirty_prompt_views(
        &self,
        views: &[close_prompt::DirtyCloseView],
    ) -> Result<(), String> {
        for view in views {
            match view.descriptor_id.0.as_str() {
                "editor.ui_asset" => {
                    self.editor_manager
                        .save_ui_asset_editor(&view.instance_id)
                        .map_err(|error| error.to_string())?;
                }
                "editor.animation_sequence" | "editor.animation_graph" => {
                    self.editor_manager
                        .save_animation_editor(&view.instance_id)
                        .map_err(|error| error.to_string())?;
                }
                _ => {
                    return Err(format!(
                        "Cannot save {} automatically; use Discard or Cancel",
                        view.title
                    ));
                }
            }
        }
        Ok(())
    }
}
