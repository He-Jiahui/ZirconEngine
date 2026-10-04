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
pub(super) fn is_blend_space_search_action(action_id: &str) -> bool {
    matches!(
        action_id,
        "workbench.extension.blend_space.search.edit"
            | "workbench.extension.blend_space.search.commit"
    )
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    // 搜索仅过滤内建 Blend Space 样例行并保留可见选择，不修改资产文件。
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
            self.select_blend_space_asset_control(control_id)?;
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
#[path = "tests/blend_space_search.rs"]
mod tests;
