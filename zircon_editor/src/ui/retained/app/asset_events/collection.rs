use super::*;
use crate::ui::host::EditorError;
use crate::ui::workbench::view::ViewInstanceId;

#[path = "collection/binding.rs"]
mod binding;
#[path = "collection/editor.rs"]
mod editor;

type CollectionEventDispatch = Option<Result<(), EditorError>>;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_ui_asset_collection_event(
        &mut self,
        instance_id: &str,
        collection_id: &str,
        event_kind: &str,
        item_index: i32,
        palette_target: Option<
            &crate::ui::retained_host::host_contract::data::UiAssetPaletteInputTarget,
        >,
        value: &str,
    ) {
        if collection_id == "palette" {
            self.dispatch_ui_asset_palette_search_event(
                instance_id,
                event_kind,
                item_index,
                palette_target,
                value,
            );
            return;
        }
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let item_index = item_index.max(0) as usize;
        let result = match self.editor_manager.enter().map_err(EditorError::from) {
            Ok(manager) => editor::dispatch_editor_collection_event(
                &manager,
                &instance_id,
                collection_id,
                event_kind,
                item_index,
            )
            .or_else(|| {
                binding::dispatch_binding_collection_event(
                    &manager,
                    &instance_id,
                    collection_id,
                    event_kind,
                    item_index,
                )
            }),
            Err(error) => Some(Err(error)),
        };

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

    fn dispatch_ui_asset_palette_search_event(
        &mut self,
        instance_id: &str,
        event_kind: &str,
        item_index: i32,
        target: Option<&crate::ui::retained_host::host_contract::data::UiAssetPaletteInputTarget>,
        value: &str,
    ) {
        let Some(target) = target.filter(|target| target.instance_id() == instance_id) else {
            return;
        };
        if !matches!(event_kind, "query" | "page" | "selected") {
            return;
        }
        let index = if event_kind == "query" {
            0
        } else {
            let Ok(index) = usize::try_from(item_index) else {
                return;
            };
            index
        };
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match self.editor_manager.enter().map_err(EditorError::from) {
            Ok(manager) => match event_kind {
                "query" => manager.set_ui_asset_editor_palette_search_query(
                    &instance_id,
                    target.snapshot(),
                    value,
                ),
                "page" => manager.set_ui_asset_editor_palette_search_page(
                    &instance_id,
                    target.snapshot(),
                    index,
                ),
                _ => manager.set_ui_asset_editor_palette_search_row(
                    &instance_id,
                    target.snapshot(),
                    index,
                ),
            },
            Err(error) => Err(error),
        };
        match result {
            Ok(true) => self.mark_presentation_dirty_for_view(&instance_id),
            Ok(false) => {}
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}
