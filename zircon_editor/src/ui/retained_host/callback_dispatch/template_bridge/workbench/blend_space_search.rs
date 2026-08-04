use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const SEARCH_CONTROL: &str = "WorkbenchExtensionBlendSpaceSearch";
const EMPTY_SEARCH_CONTROL: &str = "WorkbenchExtensionBlendSpaceSearchEmpty";
const ASSET_ROWS: &[(&str, &str)] = &[
    ("WorkbenchExtensionBlendSpaceIdleRunRow", "BS_Idle_Run"),
    ("WorkbenchExtensionBlendSpaceStrafeRow", "BS_Strafe_Grid"),
    ("WorkbenchExtensionBlendSpaceSprintRow", "BS_Sprint_Lean"),
];
const ASSET_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionBlendSpaceIdleRunRow",
    "WorkbenchExtensionBlendSpaceStrafeRow",
    "WorkbenchExtensionBlendSpaceSprintRow",
];

pub(super) fn is_blend_space_search_action(action_id: &str) -> bool {
    matches!(
        action_id,
        "workbench.extension.blend_space.search.edit"
            | "workbench.extension.blend_space.search.commit"
    )
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn apply_blend_space_search_action(
        &mut self,
        action_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if !is_blend_space_search_action(action_id) {
            return Ok(());
        }

        let query = self
            .control_string(SEARCH_CONTROL, "query")
            .unwrap_or_default();
        let mut first_match = None;
        let mut selected_match = false;

        for (control_id, label) in ASSET_ROWS {
            let matches = contains_ascii_case_insensitive(label, query.trim());
            self.set_visible(control_id, matches)?;
            if matches {
                first_match.get_or_insert(*control_id);
                selected_match |= self.control_bool(control_id, "selected");
            }
        }

        self.set_visible(EMPTY_SEARCH_CONTROL, first_match.is_none())?;
        if let Some(control_id) = first_match.filter(|_| !selected_match) {
            self.select_exclusive_selected(ASSET_ROW_CONTROLS, control_id)?;
        }
        Ok(())
    }
}

fn contains_ascii_case_insensitive(haystack: &str, needle: &str) -> bool {
    needle.is_empty()
        || haystack
            .as_bytes()
            .windows(needle.len())
            .any(|candidate| candidate.eq_ignore_ascii_case(needle.as_bytes()))
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

    use super::*;

    #[test]
    fn search_filters_blend_assets_and_preserves_a_visible_selection() {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
                .expect("workbench bridge should build");
        bridge
            .dispatch_control_state("WorkbenchAbilityBlendSpaceButton", UiEventKind::Click)
            .expect("Blend Space opener should dispatch")
            .expect("Blend Space opener should bind");

        bridge
            .mutate_control_property(
                SEARCH_CONTROL,
                "query",
                UiValue::String("strafe".to_string()),
            )
            .expect("search query should update");
        bridge
            .dispatch_control_state(SEARCH_CONTROL, UiEventKind::Change)
            .expect("search edit should dispatch")
            .expect("search edit should bind");

        assert!(
            bridge
                .control_frame("WorkbenchExtensionBlendSpaceIdleRunRow")
                .is_none()
        );
        assert!(
            bridge
                .control_frame("WorkbenchExtensionBlendSpaceStrafeRow")
                .is_some()
        );
        assert!(
            bridge
                .control_frame("WorkbenchExtensionBlendSpaceSprintRow")
                .is_none()
        );
        assert!(bridge.control_bool("WorkbenchExtensionBlendSpaceStrafeRow", "selected"));
        assert!(bridge.control_frame(EMPTY_SEARCH_CONTROL).is_none());

        bridge
            .mutate_control_property(
                SEARCH_CONTROL,
                "query",
                UiValue::String("missing".to_string()),
            )
            .expect("search query should update");
        bridge
            .dispatch_control_state(SEARCH_CONTROL, UiEventKind::Submit)
            .expect("search commit should dispatch")
            .expect("search commit should bind");

        assert!(
            bridge
                .control_frame("WorkbenchExtensionBlendSpaceStrafeRow")
                .is_none()
        );
        assert!(bridge.control_frame(EMPTY_SEARCH_CONTROL).is_some());

        bridge
            .mutate_control_property(SEARCH_CONTROL, "query", UiValue::String(String::new()))
            .expect("search query should clear");
        bridge
            .dispatch_control_state(SEARCH_CONTROL, UiEventKind::Change)
            .expect("cleared search should dispatch")
            .expect("cleared search should bind");

        for control_id in ASSET_ROW_CONTROLS {
            assert!(bridge.control_frame(control_id).is_some());
        }
        assert!(bridge.control_bool("WorkbenchExtensionBlendSpaceStrafeRow", "selected"));
        assert!(bridge.control_frame(EMPTY_SEARCH_CONTROL).is_none());
    }

    #[test]
    fn asset_search_is_ascii_case_insensitive_without_allocating_per_row() {
        assert!(contains_ascii_case_insensitive("BS_Idle_Run", "idle"));
        assert!(contains_ascii_case_insensitive("BS_Sprint_Lean", "SPRINT"));
        assert!(!contains_ascii_case_insensitive("BS_Strafe_Grid", "run"));
    }
}
