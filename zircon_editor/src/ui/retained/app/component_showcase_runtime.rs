use super::*;

const SHOWCASE_CONTENT_KIND: ViewContentKind = ViewContentKind::UiComponentShowcase;

impl RetainedEditorHost {
    pub(super) fn ensure_component_showcase_runtime_loaded(&mut self) -> Result<(), String> {
        if self.component_showcase_runtime_loaded {
            return Ok(());
        }

        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "load_component_showcase_templates"
        );
        self.component_showcase_runtime
            .load_builtin_host_templates()
            .map_err(|error| format!("Component showcase templates failed: {error}"))?;
        self.component_showcase_runtime_loaded = true;
        Ok(())
    }

    pub(super) fn prepare_component_showcase_runtime_for_presentation(
        &mut self,
        model: &WorkbenchViewModel,
    ) -> bool {
        if !should_prepare_component_showcase_runtime(model) {
            return false;
        }

        match self.ensure_component_showcase_runtime_loaded() {
            Ok(()) => true,
            Err(error) => {
                self.set_status_line(error);
                false
            }
        }
    }
}

pub(super) fn should_prepare_component_showcase_runtime(model: &WorkbenchViewModel) -> bool {
    model.document_tabs.iter().any(active_showcase_document_tab)
        || model.tool_windows.values().any(visible_showcase_tool_stack)
        || model
            .floating_windows
            .iter()
            .any(|window| window.tabs.iter().any(active_showcase_document_tab))
}

fn active_showcase_document_tab(tab: &crate::ui::workbench::model::DocumentTabModel) -> bool {
    tab.active && tab.content_kind == SHOWCASE_CONTENT_KIND
}

fn visible_showcase_tool_stack(stack: &crate::ui::workbench::model::ToolWindowStackModel) -> bool {
    stack.visible
        && stack.tabs.iter().any(|tab| {
            (tab.active || stack.active_tab.as_ref() == Some(&tab.instance_id))
                && tab.content_kind == SHOWCASE_CONTENT_KIND
        })
}
