use std::time::{Duration, Instant};

use zircon_runtime::scene::{NodeId, WorldInspectionHierarchyRow};

use crate::ui::retained_host::{HostTextInputFocusData, UiHostContext};

use super::{FrameRect, HostInvalidationMask, RetainedEditorHost, SceneEntries, callback_dispatch};

pub(in crate::ui::retained_host) const HIERARCHY_INLINE_RENAME_CONTROL_ID: &str =
    "HierarchyInlineRename";
pub(in crate::ui::retained_host::app) const HIERARCHY_INLINE_RENAME_EDIT_ACTION_ID: &str =
    "HierarchyInlineRenameEdit";
pub(in crate::ui::retained_host::app) const HIERARCHY_INLINE_RENAME_COMMIT_ACTION_ID: &str =
    "HierarchyInlineRenameCommit";
pub(in crate::ui::retained_host) const HIERARCHY_INLINE_RENAME_DISPATCH_KIND_PREFIX: &str =
    "hierarchy_inline_rename:";

const HIERARCHY_RENAME_DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(500);

pub(in crate::ui::retained_host::app) struct HierarchyRenameClick {
    node_id: NodeId,
    at: Instant,
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn try_begin_hierarchy_rename_from_keyboard(
        &mut self,
        keyboard: &zircon_runtime_interface::ui::dispatch::UiKeyboardInputEvent,
    ) -> bool {
        use zircon_runtime_interface::ui::dispatch::UiKeyboardInputState;

        if keyboard.state != UiKeyboardInputState::Pressed
            || !keyboard.logical_key.eq_ignore_ascii_case("F2")
            || !can_begin_hierarchy_rename_from_focus(
                &self.ui.global::<UiHostContext>().get_text_input_focus(),
            )
        {
            return false;
        }

        let authoritative_scene_entries = self.runtime.editor_snapshot().scene_entries;
        let Some((node_id, name)) =
            single_selected_hierarchy_rename_target(&authoritative_scene_entries)
        else {
            return false;
        };

        self.begin_hierarchy_rename(node_id, &name);
        true
    }

    pub(in crate::ui::retained_host::app) fn track_hierarchy_click_for_rename(
        &mut self,
        entry: Option<WorldInspectionHierarchyRow>,
    ) {
        self.end_hierarchy_rename();
        let Some(entry) = entry else {
            self.last_hierarchy_rename_click = None;
            return;
        };

        let now = Instant::now();
        if is_hierarchy_rename_double_click(
            self.last_hierarchy_rename_click.as_ref(),
            entry.entity,
            now,
        ) {
            self.last_hierarchy_rename_click = None;
            self.begin_hierarchy_rename(entry.entity, &entry.display_name);
        } else {
            self.last_hierarchy_rename_click = Some(HierarchyRenameClick {
                node_id: entry.entity,
                at: now,
            });
        }
    }

    pub(in crate::ui::retained_host::app) fn dispatch_hierarchy_rename_edit(
        &mut self,
        binding_id: &str,
        value: &str,
    ) {
        match binding_id {
            HIERARCHY_INLINE_RENAME_EDIT_ACTION_ID => {
                self.invalidate_host(HostInvalidationMask::RENDER);
            }
            HIERARCHY_INLINE_RENAME_COMMIT_ACTION_ID => self.commit_hierarchy_rename(value),
            _ => self.set_status_line(format!("Unknown hierarchy rename action {binding_id}")),
        }
    }

    fn begin_hierarchy_rename(&mut self, node_id: NodeId, name: &str) {
        self.ui
            .global::<UiHostContext>()
            .set_text_input_focus(HostTextInputFocusData {
                control_id: HIERARCHY_INLINE_RENAME_CONTROL_ID.into(),
                dispatch_kind: format!("{HIERARCHY_INLINE_RENAME_DISPATCH_KIND_PREFIX}{node_id}")
                    .into(),
                edit_action_id: HIERARCHY_INLINE_RENAME_EDIT_ACTION_ID.into(),
                commit_action_id: HIERARCHY_INLINE_RENAME_COMMIT_ACTION_ID.into(),
                value_text: name.into(),
                edit_frame: FrameRect::default(),
                ..HostTextInputFocusData::default()
            });
        self.invalidate_host(HostInvalidationMask::RENDER);
    }

    fn commit_hierarchy_rename(&mut self, value: &str) {
        let text_focus = self.ui.global::<UiHostContext>().get_text_input_focus();
        let Some(node_id) = hierarchy_rename_target_from_focus(&text_focus) else {
            return;
        };
        let name = value.trim();
        if name.is_empty() {
            self.set_status_line("Hierarchy node name cannot be empty");
            return;
        }

        self.end_hierarchy_rename();
        match callback_dispatch::dispatch_hierarchy_rename(&self.runtime, node_id, name.into()) {
            Ok(effects) => self.apply_dispatch_effects(effects),
            Err(error) => self.set_status_line(error),
        }
    }

    fn end_hierarchy_rename(&mut self) {
        let text_focus = self.ui.global::<UiHostContext>().get_text_input_focus();
        if text_focus.control_id.as_str() == HIERARCHY_INLINE_RENAME_CONTROL_ID {
            self.ui.global::<UiHostContext>().clear_text_input_focus();
        }
        self.invalidate_host(HostInvalidationMask::RENDER);
    }
}

fn single_selected_hierarchy_rename_target(entries: &SceneEntries) -> Option<(NodeId, String)> {
    let mut selected_entries = entries
        .iter()
        .filter(|entry| entries.is_selected(entry.entity));
    let entry = selected_entries.next()?;
    selected_entries
        .next()
        .is_none()
        .then(|| (entry.entity, entry.display_name.clone()))
}

pub(in crate::ui::retained_host) fn hierarchy_inline_rename_target_id(
    dispatch_kind: &str,
) -> Option<&str> {
    dispatch_kind.strip_prefix(HIERARCHY_INLINE_RENAME_DISPATCH_KIND_PREFIX)
}

fn hierarchy_rename_target_from_focus(text_focus: &HostTextInputFocusData) -> Option<NodeId> {
    (text_focus.control_id.as_str() == HIERARCHY_INLINE_RENAME_CONTROL_ID)
        .then(|| hierarchy_inline_rename_target_id(text_focus.dispatch_kind.as_str()))
        .flatten()
        .and_then(|node_id| node_id.parse().ok())
}

fn can_begin_hierarchy_rename_from_focus(text_focus: &HostTextInputFocusData) -> bool {
    !text_focus.is_active()
}

fn is_hierarchy_rename_double_click(
    previous: Option<&HierarchyRenameClick>,
    node_id: NodeId,
    now: Instant,
) -> bool {
    previous.is_some_and(|previous| {
        previous.node_id == node_id
            && now.saturating_duration_since(previous.at) <= HIERARCHY_RENAME_DOUBLE_CLICK_WINDOW
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scene_entries(values: &[(NodeId, &str, bool)]) -> SceneEntries {
        let selected = values
            .iter()
            .filter(|(_, _, selected)| *selected)
            .map(|(id, _, _)| *id)
            .collect::<Vec<_>>();
        SceneEntries::from_entries(
            values
                .iter()
                .map(|(id, name, _)| SceneEntry {
                    id: *id,
                    name: (*name).into(),
                    depth: 0,
                })
                .collect::<Vec<_>>(),
            selected,
        )
    }

    #[test]
    fn hierarchy_rename_target_requires_exactly_one_selected_entry() {
        let entries = scene_entries(&[(4, "Camera", true), (7, "Light", false)]);

        assert_eq!(
            single_selected_hierarchy_rename_target(&entries),
            Some((4, "Camera".into()))
        );
        assert_eq!(
            single_selected_hierarchy_rename_target(&scene_entries(&[(4, "Camera", false)])),
            None
        );
        assert_eq!(
            single_selected_hierarchy_rename_target(&scene_entries(&[
                (4, "Camera", true),
                (7, "Light", true),
            ])),
            None
        );
    }

    #[test]
    fn keyboard_rename_uses_the_authoritative_scene_snapshot() {
        let source = include_str!("hierarchy_rename.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);

        assert!(production.contains("self.runtime.editor_snapshot().scene_entries"));
        assert!(
            !production
                .contains("single_selected_hierarchy_rename_target(&self.hierarchy_scene_entries)")
        );
    }

    #[test]
    fn hierarchy_rename_dispatch_kind_carries_the_exact_node_id() {
        assert_eq!(
            hierarchy_inline_rename_target_id("hierarchy_inline_rename:42"),
            Some("42")
        );
        assert_eq!(hierarchy_inline_rename_target_id("hierarchy"), None);
    }

    #[test]
    fn hierarchy_rename_target_comes_only_from_the_active_inline_focus() {
        let focus = HostTextInputFocusData {
            control_id: HIERARCHY_INLINE_RENAME_CONTROL_ID.into(),
            dispatch_kind: "hierarchy_inline_rename:42".into(),
            ..HostTextInputFocusData::default()
        };

        assert_eq!(hierarchy_rename_target_from_focus(&focus), Some(42));
        assert_eq!(
            hierarchy_rename_target_from_focus(&HostTextInputFocusData {
                control_id: "OtherControl".into(),
                dispatch_kind: "hierarchy_inline_rename:42".into(),
                ..HostTextInputFocusData::default()
            }),
            None
        );
        assert_eq!(
            hierarchy_rename_target_from_focus(&HostTextInputFocusData {
                control_id: HIERARCHY_INLINE_RENAME_CONTROL_ID.into(),
                dispatch_kind: "hierarchy_inline_rename:not-a-node".into(),
                ..HostTextInputFocusData::default()
            }),
            None
        );
    }

    #[test]
    fn hierarchy_rename_does_not_replace_an_existing_text_input_focus() {
        assert!(can_begin_hierarchy_rename_from_focus(
            &HostTextInputFocusData::default()
        ));
        assert!(!can_begin_hierarchy_rename_from_focus(
            &HostTextInputFocusData {
                control_id: "InspectorField".into(),
                ..HostTextInputFocusData::default()
            }
        ));
    }

    #[test]
    fn hierarchy_rename_double_click_requires_the_same_node_within_the_window() {
        let now = Instant::now();
        let previous = HierarchyRenameClick {
            node_id: 4,
            at: now,
        };

        assert!(is_hierarchy_rename_double_click(Some(&previous), 4, now));
        assert!(!is_hierarchy_rename_double_click(Some(&previous), 5, now));
        assert!(!is_hierarchy_rename_double_click(
            Some(&previous),
            4,
            now + HIERARCHY_RENAME_DOUBLE_CLICK_WINDOW + Duration::from_millis(1),
        ));
    }
}
