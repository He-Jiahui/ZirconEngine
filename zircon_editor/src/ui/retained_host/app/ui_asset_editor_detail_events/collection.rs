use super::*;
use crate::ui::host::EditorError;
use crate::ui::workbench::view::ViewInstanceId;

mod binding;
mod editor;

type CollectionEventDispatch = Option<Result<(), EditorError>>;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_ui_asset_collection_event(
        &mut self,
        instance_id: &str,
        collection_id: &str,
        event_kind: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let item_index = item_index.max(0) as usize;
        let result = editor::dispatch_editor_collection_event(
            &self.editor_manager,
            &instance_id,
            collection_id,
            event_kind,
            item_index,
        )
        .or_else(|| {
            binding::dispatch_binding_collection_event(
                &self.editor_manager,
                &instance_id,
                collection_id,
                event_kind,
                item_index,
            )
        });

        let Some(result) = result else {
            self.set_status_line(format!(
                "Unknown UI asset collection event {collection_id}:{event_kind}"
            ));
            return;
        };

        match result {
            Ok(()) => self.mark_presentation_dirty_for_view(&instance_id),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}
