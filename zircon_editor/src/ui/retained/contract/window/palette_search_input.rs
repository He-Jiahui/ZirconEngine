use super::super::template_component_family::{is_component_family, TemplateComponentFamily};
use super::{retained_pane_input::find_retained_input_pane, UiHostWindow};
use crate::ui::retained_host::host_contract::data::{
    HostTextInputFocusData, HostWindowPresentationData, PaneData, UiAssetPaletteInputTarget,
};
use std::{rc::Rc, sync::Arc};

fn palette_search_pane<'a>(
    presentation: &'a HostWindowPresentationData,
    target: &UiAssetPaletteInputTarget,
) -> Option<&'a PaneData> {
    let (pane, content) = find_retained_input_pane(presentation, target.instance_id())?;
    if pane.kind.as_str() != "UiAssetEditor"
        || !content.width.is_finite()
        || !content.height.is_finite()
        || content.width <= 0.0
        || content.height <= 0.0
    {
        return None;
    }
    let current = pane.ui_asset.collections.palette_search.as_ref()?;
    current.same_source(target.snapshot()).then_some(pane)
}

pub(crate) fn palette_search_focus_current(
    presentation: &HostWindowPresentationData,
    focus: &HostTextInputFocusData,
) -> bool {
    let Some(target) = focus.palette_search_target.as_ref() else {
        return false;
    };
    let Some(pane) = palette_search_pane(presentation, target) else {
        return false;
    };
    if focus.dispatch_kind.as_str() != "ui_asset_palette_search"
        || focus.control_id.is_empty()
        || focus.edit_action_id.is_empty()
    {
        return false;
    }
    // The pane/source can survive removal or replacement of its actual field.
    // Follow the same model chosen by paint/hit testing, including template overrides.
    pane.template_nodes().is_some_and(|nodes| {
        nodes.iter().any(|node| {
            node.control_id == focus.control_id
                && node.dispatch_kind == focus.dispatch_kind
                && node.edit_action_id == focus.edit_action_id
                && node.commit_action_id == focus.commit_action_id
                // A hot-replaced node can keep its ID/routes while no longer
                // taking text input. Reuse the native hit/paint family authority.
                && is_component_family(node, TemplateComponentFamily::TextInput)
                && !node.disabled
                && node.frame.x.is_finite()
                && node.frame.y.is_finite()
                && node.frame.width.is_finite()
                && node.frame.height.is_finite()
                && node.frame.width > 0.0
                && node.frame.height > 0.0
        })
    })
}

impl UiHostWindow {
    /// Retire only the Palette field, without invoking app callbacks or publishing layout.
    pub(crate) fn retire_palette_search_input(&self) -> bool {
        let mut state = self.state.borrow_mut();
        if state.text_input_focus.palette_search_target.is_none() {
            return false;
        }
        state.replace_text_input_focus(Default::default())
    }

    pub(crate) fn same_host_instance(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }

    pub(crate) fn palette_search_input_is_current(
        &self,
        target: &UiAssetPaletteInputTarget,
        event: &str,
    ) -> bool {
        if !self.window().is_visible() {
            return false;
        }
        let generation = self.get_host_presentation_generation();
        let Some(pane) = palette_search_pane(generation.structure(), target) else {
            return false;
        };
        match event {
            // Query/page allocations change on every keystroke; keep the one focused
            // field alive only while its actual source allocation and pane remain valid.
            "query" => {
                generation.text_input_focus().palette_search_target.as_ref() == Some(target)
                    && palette_search_focus_current(
                        generation.structure(),
                        generation.text_input_focus(),
                    )
            }
            "selected" | "page" => pane
                .body_surface_frame
                .as_ref()
                .is_some_and(|frame| Arc::ptr_eq(frame, target.shell_frame())),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::asset_editor::{
        ui_asset_editor_node_projection_with_palette_search, UiAssetEditorMode, UiAssetEditorRoute,
        UiAssetEditorSession,
    };
    use crate::ui::layouts::common::model_rc;
    use crate::ui::retained_host::host_contract::{
        data::{FrameRect, HostTextInputFocusData, TemplateNodeFrameData, TemplatePaneNodeData},
        globals::HostContractState,
    };
    use crate::ui::retained_host::primitives::PhysicalSize;
    use zircon_runtime_interface::ui::{
        layout::UiSize, surface::UiSurfaceFrame, template::UiAssetKind,
    };

    fn session() -> UiAssetEditorSession {
        UiAssetEditorSession::from_v2_source(
            UiAssetEditorRoute::new(
                "editor.test.palette_context",
                UiAssetKind::Layout,
                UiAssetEditorMode::Design,
            ),
            r#"[asset]
kind = "view"
id = "editor.test.palette_context"
version = 2
[root]
node = "root"
[nodes.root]
component = "VerticalGroup"
"#,
            UiSize::new(640.0, 520.0),
        )
        .unwrap()
    }

    fn presentation(session: &UiAssetEditorSession) -> HostWindowPresentationData {
        let mut value = HostWindowPresentationData::default();
        let dock = &mut value.host_scene_data.document_dock;
        dock.pane.id = "ui-asset-editor#palette".into();
        dock.pane.kind = "UiAssetEditor".into();
        dock.content_frame = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 640.0,
            height: 520.0,
        };
        let snapshot = session.palette_search_snapshot();
        let field = ui_asset_editor_node_projection_with_palette_search(
            UiSize::new(640.0, 520.0),
            &snapshot,
        )
        .nodes
        .into_iter()
        .find(|node| node.dispatch_kind.as_str() == "ui_asset_palette_search")
        .expect("actual product ZUI must project its Palette search field");
        // Only eligibility metadata is copied from the production v2 projection.
        // This unit fixture is not a Surface/renderer producer or native GUI evidence.
        dock.pane.ui_asset.nodes = model_rc(vec![TemplatePaneNodeData {
            control_id: field.control_id,
            role: field.role,
            component_role: field.component_role,
            dispatch_kind: field.dispatch_kind,
            edit_action_id: field.edit_action_id,
            commit_action_id: field.commit_action_id,
            disabled: field.disabled,
            frame: TemplateNodeFrameData {
                x: field.frame.x,
                y: field.frame.y,
                width: field.frame.width,
                height: field.frame.height,
            },
            ..TemplatePaneNodeData::default()
        }]);
        dock.pane.ui_asset.collections.palette_search = Some(snapshot);
        value
    }

    fn target(session: &UiAssetEditorSession) -> UiAssetPaletteInputTarget {
        // Synthetic frame is only a unit identity fixture, never native evidence.
        UiAssetPaletteInputTarget::new(
            "ui-asset-editor#palette".into(),
            session.palette_search_snapshot(),
            Arc::new(UiSurfaceFrame::default()),
        )
    }

    fn input_focus(input: &UiAssetPaletteInputTarget) -> HostTextInputFocusData {
        HostTextInputFocusData {
            control_id: "PaletteSearchInput".into(),
            dispatch_kind: "ui_asset_palette_search".into(),
            edit_action_id: "palette.search.query".into(),
            palette_search_target: Some(input.clone()),
            ..HostTextInputFocusData::default()
        }
    }

    #[test]
    fn palette_context_requires_visible_active_instance_and_same_source_allocation() {
        let first = session();
        let input = target(&first);
        let original = presentation(&first);
        let focus = input_focus(&input);
        assert!(palette_search_focus_current(&original, &focus));
        let mut hidden = original.clone();
        hidden.host_scene_data.document_dock.pane.id = "different-active-document".into();
        assert!(!palette_search_focus_current(&hidden, &focus));
        assert!(!palette_search_focus_current(
            &presentation(&session()),
            &focus
        ));
        let mut no_area = original;
        no_area.host_scene_data.document_dock.content_frame.width = 0.0;
        assert!(!palette_search_focus_current(&no_area, &focus));
    }

    #[test]
    fn query_publication_keeps_caret_but_hide_and_reopen_retire_field_without_resurrection() {
        let mut first = session();
        let input = target(&first);
        let original = presentation(&first);
        let mut state = HostContractState::new(PhysicalSize::new(640, 520));
        state.window_visible = true;
        state.replace_host_presentation(original.clone());
        state.replace_text_input_focus(HostTextInputFocusData {
            control_id: "PaletteSearchInput".into(),
            dispatch_kind: "ui_asset_palette_search".into(),
            edit_action_id: "palette.search.query".into(),
            palette_search_target: Some(input.clone()),
            caret_scalar_offset: Some(2),
            ..HostTextInputFocusData::default()
        });
        first.set_palette_search_query("中文");
        state.replace_host_presentation(presentation(&first));
        assert_eq!(state.text_input_focus.caret_scalar_offset, Some(2));
        assert!(state.text_input_focus.palette_search_target.as_ref() == Some(&input));

        let mut hidden = presentation(&first);
        hidden.host_scene_data.document_dock.pane.id = "different-active-document".into();
        state.replace_host_presentation(hidden);
        assert!(!state.text_input_focus.is_active());
        state.replace_host_presentation(original);
        assert!(
            !state.text_input_focus.is_active(),
            "publishing an old pane cannot restore a retired field"
        );

        state.replace_text_input_focus(HostTextInputFocusData {
            control_id: "PaletteSearchInput".into(),
            dispatch_kind: "ui_asset_palette_search".into(),
            edit_action_id: "palette.search.query".into(),
            palette_search_target: Some(input),
            ..HostTextInputFocusData::default()
        });
        state.replace_host_presentation(presentation(&session()));
        assert!(!state.text_input_focus.is_active());
    }

    fn focused_window() -> (UiHostWindow, UiAssetPaletteInputTarget) {
        let source = session();
        let input = target(&source);
        let host = UiHostWindow::new().expect("unshown host contract with real font assets");
        host.show().unwrap();
        host.set_host_presentation(presentation(&source));
        host.state
            .borrow_mut()
            .replace_text_input_focus(HostTextInputFocusData {
                control_id: "PaletteSearchInput".into(),
                dispatch_kind: "ui_asset_palette_search".into(),
                edit_action_id: "palette.search.query".into(),
                value_text: "中文 query".into(),
                palette_search_target: Some(input.clone()),
                caret_scalar_offset: Some(2),
                ..HostTextInputFocusData::default()
            });
        assert!(host.palette_search_input_is_current(&input, "query"));
        (host, input)
    }

    #[test]
    fn palette_search_programmatic_hide_cannot_revive_a_field_without_new_publication() {
        let (host, input) = focused_window();
        let before = host.get_host_presentation_generation();
        host.hide().unwrap();
        assert!(!host.text_input_focus_active());
        assert!(!host.window().is_visible());
        host.show().unwrap();
        let after = host.get_host_presentation_generation();
        assert!(std::ptr::eq(before.structure(), after.structure()));
        assert!(!host.palette_search_input_is_current(&input, "query"));
        assert!(!host.text_input_focus_accepts_text());
    }

    #[test]
    fn palette_search_native_blur_retires_field_and_requests_frame_without_layout_change() {
        use crate::ui::retained_host::host_contract::globals::UiHostContext;
        use std::cell::Cell;

        let (host, input) = focused_window();
        let before = host.get_host_presentation_generation();
        let requests = Rc::new(Cell::new(0));
        let observed = Rc::clone(&requests);
        host.global::<UiHostContext>()
            .on_frame_requested(move || observed.set(observed.get() + 1));
        host.dispatch_native_focus_lost_event_for_test();
        let after = host.get_host_presentation_generation();
        assert!(std::ptr::eq(before.structure(), after.structure()));
        assert!(host.window().is_visible());
        assert!(!host.palette_search_input_is_current(&input, "query"));
        assert!(!host.text_input_focus_accepts_text());
        assert!(requests.get() > 0);
    }

    #[test]
    fn palette_search_native_close_retires_only_after_hide_is_accepted() {
        use crate::ui::retained_host::primitives::CloseRequestResponse;

        let (host, input) = focused_window();
        host.window()
            .on_close_requested(|| CloseRequestResponse::KeepWindowShown);
        assert_eq!(
            host.dispatch_native_close_request_event_for_test(),
            CloseRequestResponse::KeepWindowShown
        );
        assert!(host.window().is_visible());
        assert!(host.palette_search_input_is_current(&input, "query"));
        host.window()
            .on_close_requested(|| CloseRequestResponse::HideWindow);
        assert_eq!(
            host.dispatch_native_close_request_event_for_test(),
            CloseRequestResponse::HideWindow
        );
        assert!(!host.text_input_focus_active());
        host.show().unwrap();
        assert!(!host.palette_search_input_is_current(&input, "query"));
    }

    #[test]
    fn palette_search_exit_retires_field_without_waiting_for_structure_publication() {
        let (host, input) = focused_window();
        let before = host.get_host_presentation_generation();
        host.request_exit();
        let after = host.get_host_presentation_generation();
        assert!(std::ptr::eq(before.structure(), after.structure()));
        assert!(!host.window().is_visible());
        assert!(!host.text_input_focus_active());
        assert!(!host.palette_search_input_is_current(&input, "query"));
    }

    #[test]
    fn palette_search_retirement_preserves_other_field_and_chord_owners() {
        let (host, _) = focused_window();
        for kind in ["inspector", "chord_capture"] {
            let original = HostTextInputFocusData {
                control_id: "another-control".into(),
                dispatch_kind: kind.into(),
                value_text: "original".into(),
                caret_scalar_offset: Some(3),
                ..HostTextInputFocusData::default()
            };
            host.state
                .borrow_mut()
                .replace_text_input_focus(original.clone());
            host.hide().unwrap();
            host.show().unwrap();
            assert!(host.state.borrow().text_input_focus.as_ref() == &original);
        }
    }

    #[test]
    fn palette_search_missing_or_paint_overridden_field_retires_old_query_without_reviving_it() {
        for paint_override in [false, true] {
            let (host, input) = focused_window();
            let original = host.get_host_presentation_generation().structure().clone();
            let mut hidden = original.clone();
            let pane = &mut hidden.host_scene_data.document_dock.pane;
            if paint_override {
                pane.template_v2.nodes = model_rc(vec![TemplatePaneNodeData {
                    control_id: "different-painted-body".into(),
                    ..TemplatePaneNodeData::default()
                }]);
            } else {
                pane.ui_asset.nodes = model_rc(Vec::new());
            }
            assert!(pane
                .ui_asset
                .collections
                .palette_search
                .as_ref()
                .unwrap()
                .same_source(input.snapshot()));
            host.set_host_presentation(hidden);
            assert!(!host.text_input_focus_active());
            assert!(!host.palette_search_input_is_current(&input, "query"));
            host.set_host_presentation(original);
            assert!(!host.text_input_focus_active());
            assert!(!host.palette_search_input_is_current(&input, "query"));
        }
    }

    #[test]
    fn palette_search_focus_rejects_changed_field_identity_disabled_or_invalid_layout() {
        let (host, input) = focused_window();
        let original = host.get_host_presentation_generation().structure().clone();
        let focus = host.state.borrow().text_input_focus.as_ref().clone();
        for changed in 0..9 {
            let mut candidate = original.clone();
            let pane = &mut candidate.host_scene_data.document_dock.pane;
            let mut field = pane.ui_asset.nodes.row_data(0).unwrap();
            match changed {
                0 => field.control_id = "different-control".into(),
                1 => field.edit_action_id = "different-edit-route".into(),
                2 => field.commit_action_id = "different-commit-route".into(),
                3 => field.dispatch_kind = "inspector".into(),
                4 => field.disabled = true,
                5 => field.frame.width = 0.0,
                6 => field.frame.height = -1.0,
                7 => field.frame.x = f32::INFINITY,
                _ => field.frame.height = f32::NAN,
            }
            pane.ui_asset.nodes = model_rc(vec![field]);
            assert!(pane
                .ui_asset
                .collections
                .palette_search
                .as_ref()
                .unwrap()
                .same_source(input.snapshot()));
            assert!(!palette_search_focus_current(&candidate, &focus));
        }
        assert!(palette_search_focus_current(&original, &focus));
        assert!(host.palette_search_input_is_current(&input, "query"));
    }

    #[test]
    fn palette_search_same_id_non_text_replacement_retires_query_and_does_not_revive_it() {
        for (role, component_role) in [
            ("Button", "button"),
            ("Dropdown", "dropdown"),
            ("KeySelector", "key-selector"),
            ("Label", ""),
        ] {
            let (host, input) = focused_window();
            let original = host.get_host_presentation_generation().structure().clone();
            let focus = host.state.borrow().text_input_focus.as_ref().clone();
            host.set_host_presentation(original.clone());
            assert!(host.palette_search_input_is_current(&input, "query"));
            assert!(host.state.borrow().text_input_focus.as_ref() == &focus);

            let mut replacement = original.clone();
            let pane = &mut replacement.host_scene_data.document_dock.pane;
            let mut field = pane.ui_asset.nodes.row_data(0).unwrap();
            field.role = role.into();
            field.component_role = component_role.into();
            assert_eq!(field.control_id, focus.control_id);
            assert_eq!(field.dispatch_kind, focus.dispatch_kind);
            assert_eq!(field.edit_action_id, focus.edit_action_id);
            assert_eq!(field.commit_action_id, focus.commit_action_id);
            assert!(!is_component_family(
                &field,
                TemplateComponentFamily::TextInput
            ));
            pane.ui_asset.nodes = model_rc(vec![field]);
            assert!(pane
                .ui_asset
                .collections
                .palette_search
                .as_ref()
                .unwrap()
                .same_source(input.snapshot()));
            assert!(!palette_search_focus_current(&replacement, &focus));

            host.set_host_presentation(replacement);
            assert!(!host.text_input_focus_active());
            assert!(!host.palette_search_input_is_current(&input, "query"));
            assert!(!host
                .dispatch_focused_text_insert("late query")
                .request_redraw());
            host.set_host_presentation(original);
            assert!(!host.text_input_focus_active());
            assert!(!host.palette_search_input_is_current(&input, "query"));
        }
    }
}
