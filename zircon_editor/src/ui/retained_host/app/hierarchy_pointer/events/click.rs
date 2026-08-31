use super::super::super::{callback_dispatch, RetainedEditorHost, UiPoint};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn hierarchy_pointer_clicked(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        let scene_entries = self.prepare_hierarchy_pointer_target(width, height, true);
        match callback_dispatch::dispatch_shared_hierarchy_pointer_click(
            &self.runtime,
            &mut self.hierarchy_pointer_bridge,
            scene_entries.as_ref(),
            UiPoint::new(x, y),
        ) {
            Ok(dispatch) => {
                let rename_entry = dispatch
                    .selected_entity
                    .and_then(|entity| self.runtime.scene_inspection_hierarchy_row(entity));
                self.hierarchy_pointer_state = dispatch.pointer.state;
                self.apply_hierarchy_pointer_state_to_ui();
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
                self.track_hierarchy_click_for_rename(rename_entry);
            }
            Err(error) => self.set_status_line(error),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn rename_reads_the_exact_runtime_row_after_sparse_name_patches() {
        let source = include_str!("click.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);

        assert!(production.contains("self.runtime.scene_inspection_hierarchy_row(entity)"));
        assert!(!production.contains("get(*item_index).cloned()"));
    }
}
