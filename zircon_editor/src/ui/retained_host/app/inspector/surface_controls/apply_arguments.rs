use super::super::super::RetainedEditorHost;
use zircon_runtime_interface::ui::binding::UiBindingValue;

impl RetainedEditorHost {
    pub(super) fn inspector_apply_arguments(&self) -> Result<Vec<UiBindingValue>, String> {
        let selected = self
            .runtime
            .shell()
            .lock()
            .state
            .viewport_controller
            .selection()
            .active_primary();
        inspector_apply_arguments_for_active_selection(selected)
    }
}

fn inspector_apply_arguments_for_active_selection(
    selected: Option<zircon_runtime::scene::NodeId>,
) -> Result<Vec<UiBindingValue>, String> {
    if selected.is_none() {
        return Err("Nothing selected".to_string());
    }
    Ok(vec![
        UiBindingValue::string("entity://selected"),
        UiBindingValue::array(Vec::new()),
    ])
}

#[cfg(test)]
#[path = "apply_arguments/tests/owned_snapshot_tests.rs"]
mod owned_snapshot_tests;
