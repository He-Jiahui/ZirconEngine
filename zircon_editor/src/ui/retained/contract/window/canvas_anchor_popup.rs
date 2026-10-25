//! Native Canvas popup lifecycle. The original Session ticket lives in the app consumer.
use super::super::data::{FrameRect, HostWindowPresentationData, TemplatePaneNodeData};
use super::{retained_pane_input::find_retained_input_pane, UiHostWindow};
use crate::ui::retained_host::primitives::SharedString;
use std::{rc::Rc, sync::Arc};
use zircon_runtime_interface::ui::surface::UiSurfaceFrame;

#[path = "canvas_anchor_popup/input.rs"]
mod input;
#[path = "canvas_anchor_popup/visual.rs"]
mod visual;
pub(crate) use visual::CanvasAnchorPopupVisual;

#[derive(Clone)]
pub(crate) struct CanvasAnchorPopupId(Rc<()>);
impl CanvasAnchorPopupId {
    pub(crate) fn new() -> Self {
        Self(Rc::new(()))
    }
    pub(crate) fn same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone, Copy)]
pub(crate) enum CanvasAnchorPopupKind {
    Submit {
        index: usize,
        shift: bool,
        control: bool,
    },
    Escape,
    ClickAway,
}

#[derive(Clone)]
pub(crate) struct CanvasAnchorPopupEvent {
    pub(crate) id: CanvasAnchorPopupId,
    pub(crate) kind: CanvasAnchorPopupKind,
}

#[derive(Clone)]
pub(crate) struct CanvasAnchorPopupAddress {
    pub(crate) instance_id: SharedString,
    pub(crate) source_frame: Arc<UiSurfaceFrame>,
    pub(crate) opener: TemplatePaneNodeData,
}

impl CanvasAnchorPopupAddress {
    pub(crate) fn matches(&self, next: &HostWindowPresentationData) -> bool {
        let Some(current) = address_in(next, self.instance_id.as_str()) else {
            return false;
        };
        Arc::ptr_eq(&self.source_frame, &current.source_frame)
            && self.opener.node_id == current.opener.node_id
            && self.opener.source_path == current.opener.source_path
            && self.opener.source_node_id == current.opener.source_node_id
            && self.opener.instance_path == current.opener.instance_path
            && self.opener.frame.x == current.opener.frame.x
            && self.opener.frame.y == current.opener.frame.y
            && self.opener.frame.width == current.opener.frame.width
            && self.opener.frame.height == current.opener.frame.height
    }
}

pub(crate) struct CanvasAnchorPopupOpen {
    address: CanvasAnchorPopupAddress,
}
impl CanvasAnchorPopupOpen {
    pub(in crate::ui::retained_host::host_contract) fn from_hit(
        presentation: &HostWindowPresentationData,
        hit: &super::super::surface_hit_test::TemplateNodePointerHit,
    ) -> Option<Self> {
        let address = address_in(presentation, hit.pane_id.as_str())?;
        let opener = &address.opener;
        if opener.control_id != hit.control_id
            || opener.action_id != hit.action_id
            || opener.frame.x != hit.frame.x
            || opener.frame.y != hit.frame.y
            || opener.frame.width != hit.frame.width
            || opener.frame.height != hit.frame.height
        {
            return None;
        }
        Some(Self { address })
    }
    pub(crate) fn into_current_address(
        self,
        source_ui: &UiHostWindow,
    ) -> Option<CanvasAnchorPopupAddress> {
        // Recheck active pane, native input ownership and the full captured
        // address before the app consumer closes any popup or begins a ticket.
        source_ui.canvas_anchor_popup_address(self.address.instance_id.as_str())?;
        let generation = source_ui.get_host_presentation_generation();
        self.address
            .matches(generation.structure())
            .then_some(self.address)
    }
}

pub(crate) struct CanvasAnchorPopupState {
    pub(crate) id: CanvasAnchorPopupId,
    pub(crate) address: CanvasAnchorPopupAddress,
    pub(crate) visual: Rc<CanvasAnchorPopupVisual>,
    pub(crate) focused: usize,
    pub(crate) hovered: Option<usize>,
    pub(crate) pressed: Option<(usize, zircon_runtime_interface::ui::dispatch::UiPointerId)>,
    pub(crate) swallow_release: Option<zircon_runtime_interface::ui::dispatch::UiPointerId>,
    // Calls the immutable Manager's exact-ticket cancel only. Never borrows RetainedEditorHost.
    pub(crate) retire: Rc<dyn Fn() -> Result<(), String>>,
}

fn address_in(
    presentation: &HostWindowPresentationData,
    instance_id: &str,
) -> Option<CanvasAnchorPopupAddress> {
    if presentation.close_prompt.visible
        || presentation.asset_deletion_blocker.visible
        || presentation.menu_state.open_menu_index >= 0
        || presentation.host_page_overflow_menu_state.open
        || presentation.host_dock_overflow_menu_state.open
    {
        return None;
    }
    let (pane, content) = find_retained_input_pane(presentation, instance_id)?;
    if pane.kind.as_str() != "UiAssetEditor" {
        return None;
    }
    let frame = pane.ui_asset.preview.canvas.source_surface_frame.as_ref()?;
    pane.ui_asset
        .preview
        .canvas
        .transform(&pane.ui_asset.designer_canvas_panel_node, &content)?;
    let opener_control_id =
        super::super::data::ui_asset_canvas_anchor_opener_control_id(instance_id);
    let mut opener = pane
        .ui_asset
        .nodes
        .iter()
        .find(|node| {
            node.control_id == opener_control_id
                && node.dispatch_kind.as_str() == "ui_asset_anchor_popup_open"
                && !node.disabled
        })?
        .clone();
    opener.frame.x += content.x;
    opener.frame.y += content.y;
    if opener.frame.width <= 0.0 || opener.frame.height <= 0.0 {
        return None;
    }
    // A clipped/nonvisible opener must not authorize a popup.
    if opener.frame.x < content.x
        || opener.frame.y < content.y
        || opener.frame.x + opener.frame.width > content.x + content.width
        || opener.frame.y + opener.frame.height > content.y + content.height
    {
        return None;
    }
    Some(CanvasAnchorPopupAddress {
        instance_id: instance_id.into(),
        source_frame: Arc::clone(frame),
        opener,
    })
}

pub(in crate::ui::retained_host::host_contract) fn can_open_now(
    state: &super::super::globals::HostContractState,
) -> bool {
    state.window_visible
        && !state.exit_requested
        && state.menu_state.open_menu_index < 0
        && !state.host_page_overflow_menu_state.open
        && !state.host_dock_overflow_menu_state.open
        && !state.resize_state.resize_active
        && state.drag_state.drag_tab_id.is_empty()
        && super::super::native_popup_dismiss::unreal_viewport_overflow_dismiss_target(
            &state.host_presentation.workbench_window_nodes,
        )
        .is_none()
}

impl UiHostWindow {
    pub(crate) fn canvas_anchor_popup_owns_native_input(&self) -> bool {
        let state = self.state.borrow();
        state.window_visible && state.canvas_anchor_popup.is_some()
    }

    pub(crate) fn same_native_window(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }

    pub(crate) fn canvas_anchor_popup_address(
        &self,
        instance_id: &str,
    ) -> Option<CanvasAnchorPopupAddress> {
        if !can_open_now(&self.state.borrow())
            || self.text_input_focus_active()
            || self.chord_capture_focus_active()
        {
            return None;
        }
        let generation = self.get_host_presentation_generation();
        address_in(generation.structure(), instance_id)
    }

    pub(crate) fn canvas_anchor_popup_is_open(&self, id: &CanvasAnchorPopupId) -> bool {
        self.state
            .borrow()
            .canvas_anchor_popup
            .as_ref()
            .is_some_and(|popup| popup.id.same(id))
    }

    pub(crate) fn canvas_anchor_popup_has_current_address(&self, id: &CanvasAnchorPopupId) -> bool {
        let state = self.state.borrow();
        state.window_visible
            && state.canvas_anchor_popup.as_ref().is_some_and(|popup| {
                popup.id.same(id) && popup.address.matches(state.host_presentation.as_ref())
            })
    }

    pub(crate) fn show_canvas_anchor_popup(
        &self,
        id: CanvasAnchorPopupId,
        address: CanvasAnchorPopupAddress,
        retire: Rc<dyn Fn() -> Result<(), String>>,
    ) -> bool {
        let generation = self.get_host_presentation_generation();
        if !can_open_now(&self.state.borrow()) || !address.matches(generation.structure()) {
            return false;
        }
        let Some(visual) = CanvasAnchorPopupVisual::new(
            &address,
            self.window().size(),
            self.window().scale_factor(),
            0,
            None,
            None,
        ) else {
            return false;
        };
        drop(generation);
        self.retire_current_canvas_anchor_popup();
        let damage = visual.bounds.clone();
        let visual = Rc::new(visual);
        let mut state = self.state.borrow_mut();
        // Retire only this actual opener's generic press before popup input owns its Up.
        if state.pane_interaction_state.pressed_template_control_id == address.opener.control_id
            && state.pane_interaction_state.pressed_template_pane_id == address.instance_id
        {
            state.clear_template_button_press();
        }
        // Only the outer host projection changes. No Session/preset/preview resize occurs.
        state.canvas_anchor_popup = Some(CanvasAnchorPopupState {
            id,
            address,
            visual: Rc::clone(&visual),
            focused: 0,
            hovered: None,
            pressed: None,
            swallow_release: None,
            retire,
        });
        state.update_host_presentation(|presentation| {
            presentation.canvas_anchor_popup = Some(visual)
        });
        drop(state);
        self.request_redraw_region(damage);
        true
    }

    pub(crate) fn close_canvas_anchor_popup(
        &self,
        id: &CanvasAnchorPopupId,
        restore_focus: bool,
    ) -> bool {
        let popup = {
            let mut state = self.state.borrow_mut();
            if !state
                .canvas_anchor_popup
                .as_ref()
                .is_some_and(|popup| popup.id.same(id))
            {
                return false;
            }
            let popup = state.canvas_anchor_popup.take().expect("identity checked");
            state.canvas_anchor_popup_pointer_tail = popup
                .swallow_release
                .or_else(|| popup.pressed.map(|(_, pointer)| pointer));
            let return_focus = restore_focus
                && state.window_visible
                && popup.address.matches(state.host_presentation.as_ref());
            state.update_host_presentation(|presentation| presentation.canvas_anchor_popup = None);
            if return_focus {
                state.update_pane_interaction(true, |interaction| {
                    interaction.focused_template_control_id =
                        popup.address.opener.control_id.clone();
                    interaction.template_button_frame = FrameRect {
                        x: popup.address.opener.frame.x,
                        y: popup.address.opener.frame.y,
                        width: popup.address.opener.frame.width,
                        height: popup.address.opener.frame.height,
                    };
                    interaction.template_focus_visible = true;
                    interaction.template_focus_observed = true;
                });
            }
            popup
        };
        // Release the native state borrow before document/session guards or service calls.
        if let Err(error) = (popup.retire)() {
            zircon_runtime::diagnostic_log::write_diagnostic_log(
                "canvas_anchor_popup_retire",
                error,
            );
        }
        let damage = if restore_focus {
            super::super::frame_geometry::union_optional_frames(
                Some(popup.visual.bounds.clone()),
                Some(FrameRect {
                    x: popup.address.opener.frame.x,
                    y: popup.address.opener.frame.y,
                    width: popup.address.opener.frame.width,
                    height: popup.address.opener.frame.height,
                }),
            )
            .unwrap_or_else(|| popup.visual.bounds.clone())
        } else {
            popup.visual.bounds.clone()
        };
        self.request_redraw_region(damage);
        true
    }

    pub(crate) fn retire_current_canvas_anchor_popup(&self) {
        let id = self
            .state
            .borrow()
            .canvas_anchor_popup
            .as_ref()
            .map(|popup| popup.id.clone());
        if let Some(id) = id {
            self.close_canvas_anchor_popup(&id, false);
        }
    }

    pub(crate) fn canvas_anchor_popup_instance(&self) -> Option<SharedString> {
        self.state
            .borrow()
            .canvas_anchor_popup
            .as_ref()
            .map(|popup| popup.address.instance_id.clone())
    }

    pub(crate) fn prepare_canvas_anchor_popup_publication(
        &self,
        next: &mut HostWindowPresentationData,
    ) {
        let valid = {
            let state = self.state.borrow();
            state
                .canvas_anchor_popup
                .as_ref()
                .map(|popup| (popup.id.clone(), popup.address.matches(next)))
        };
        if let Some((id, false)) = valid {
            self.close_canvas_anchor_popup(&id, false);
        }
        next.canvas_anchor_popup = self
            .state
            .borrow()
            .canvas_anchor_popup
            .as_ref()
            .map(|popup| Rc::clone(&popup.visual));
    }
}

#[cfg(test)]
mod open_consumer_tests {
    use super::*;
    use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession};
    use crate::ui::retained_host::host_contract::data::TemplateNodeFrameData;
    use std::cell::{Cell, RefCell};
    use zircon_runtime_interface::ui::{layout::UiSize, template::UiAssetKind};

    pub(super) fn published_canvas() -> (
        UiHostWindow,
        Rc<RefCell<UiAssetEditorSession>>,
        HostWindowPresentationData,
    ) {
        let mut session = UiAssetEditorSession::from_v2_source(
            UiAssetEditorRoute::new(
                "asset://ui/canvas-open-consumer.zui",
                UiAssetKind::Layout,
                UiAssetEditorMode::Design,
            ),
            r#"
[asset]
kind = "view"
id = "ui.test.canvas_open_consumer"
version = 2
[root]
node = "canvas"
[nodes.canvas]
component = "Panel"
layout = { container = { kind = "Canvas" } }
children = [{ node = "button", slot = { layout = { offset = { right = 80, bottom = 40 } } } }]
[nodes.button]
component = "Button"
props = { text = "Canvas" }
"#,
            UiSize::new(640.0, 480.0),
        )
        .expect("real v2 preview session");
        session.select_hierarchy_index(1).unwrap();
        let frame = session.preview_host().surface().surface_frame();
        let mut presentation = HostWindowPresentationData::default();
        let dock = &mut presentation.host_scene_data.document_dock;
        dock.region_frame = FrameRect {
            width: 1000.0,
            height: 700.0,
            ..Default::default()
        };
        dock.content_frame = dock.region_frame.clone();
        let pane = &mut dock.pane;
        pane.id = "canvas-view".into();
        pane.kind = "UiAssetEditor".into();
        pane.ui_asset.preview.canvas.source_surface_frame = Some(frame);
        pane.ui_asset.preview.canvas.width = 640.0;
        pane.ui_asset.preview.canvas.height = 480.0;
        pane.ui_asset.designer_canvas_panel_node.frame = TemplateNodeFrameData {
            x: 20.0,
            y: 80.0,
            width: 600.0,
            height: 400.0,
        };
        pane.ui_asset.nodes = crate::ui::layouts::common::model_rc(vec![TemplatePaneNodeData {
            node_id: "canvas-view/slot/anchors".into(),
            source_path: "asset://ui/canvas-open-consumer.zui".into(),
            source_node_id: "button".into(),
            control_id: super::super::super::data::ui_asset_canvas_anchor_opener_control_id(
                "canvas-view",
            ),
            dispatch_kind: "ui_asset_anchor_popup_open".into(),
            role: "Button".into(),
            action_id: "canvas.anchor.open".into(),
            frame: TemplateNodeFrameData {
                x: 40.0,
                y: 10.0,
                width: 100.0,
                height: 24.0,
            },
            ..Default::default()
        }]);
        let ui = UiHostWindow::new().expect("unshown native host contract");
        ui.state.borrow_mut().window_visible = true;
        ui.set_host_presentation(presentation.clone());
        (ui, Rc::new(RefCell::new(session)), presentation)
    }

    fn queued_open(ui: &UiHostWindow) -> CanvasAnchorPopupOpen {
        CanvasAnchorPopupOpen {
            address: ui
                .canvas_anchor_popup_address("canvas-view")
                .expect("visible opener"),
        }
    }

    #[test]
    fn canvas_anchor_stale_open_cannot_retire_current_popup_or_session_ticket() {
        let mutations: [fn(&mut TemplatePaneNodeData); 8] = [
            |node| node.frame.x += 1.0,
            |node| node.frame.y += 1.0,
            |node| node.frame.width += 1.0,
            |node| node.frame.height += 1.0,
            |node| node.node_id = "canvas-view/new-slot/anchors".into(),
            |node| node.source_path = "asset://ui/other.zui".into(),
            |node| node.source_node_id = "other-button".into(),
            |node| node.instance_path = "other-instance".into(),
        ];
        for mutate in mutations {
            let (ui, session, mut presentation) = published_canvas();
            let old = queued_open(&ui);
            let source_frame = old.address.source_frame.clone();
            let mut opener = presentation
                .host_scene_data
                .document_dock
                .pane
                .ui_asset
                .nodes
                .get(0)
                .unwrap();
            mutate(&mut opener);
            presentation
                .host_scene_data
                .document_dock
                .pane
                .ui_asset
                .nodes = crate::ui::layouts::common::model_rc(vec![opener]);
            ui.set_host_presentation(presentation);
            let ticket = session
                .borrow_mut()
                .begin_canvas_anchor_preset()
                .unwrap()
                .unwrap();
            let id = CanvasAnchorPopupId::new();
            let retired = Rc::new(Cell::new(0));
            let (retire_count, retire_session, retire_ticket) =
                (retired.clone(), session.clone(), ticket.clone());
            assert!(ui.show_canvas_anchor_popup(
                id.clone(),
                ui.canvas_anchor_popup_address("canvas-view").unwrap(),
                Rc::new(move || {
                    retire_count.set(retire_count.get() + 1);
                    assert!(retire_session
                        .borrow_mut()
                        .cancel_canvas_anchor_preset(&retire_ticket));
                    Ok(())
                }),
            ));
            let generation = ui.get_host_presentation_generation().structure_generation();
            let source = session.borrow().source_buffer().text().to_string();
            assert!(old.into_current_address(&ui).is_none());
            assert!(ui.canvas_anchor_popup_is_open(&id));
            assert!(ticket.is_current(&session.borrow()));
            assert_eq!(retired.get(), 0);
            assert_eq!(session.borrow().source_buffer().text(), source);
            assert_eq!(
                ui.get_host_presentation_generation().structure_generation(),
                generation
            );
            let accepted = queued_open(&ui).into_current_address(&ui).unwrap();
            assert!(Arc::ptr_eq(&accepted.source_frame, &source_frame));
            assert!(ticket.is_current(&session.borrow()));
            assert!(ui.close_canvas_anchor_popup(&id, false));
            assert_eq!(retired.get(), 1);
        }
    }

    #[test]
    fn canvas_anchor_queued_open_rejects_equal_value_frame_aba_and_inactive_pane() {
        let (ui, _session, presentation) = published_canvas();
        let old = queued_open(&ui);
        let original = old.address.source_frame.clone();
        let replacement = Arc::new(original.as_ref().clone());
        assert_eq!(original.generation, replacement.generation);
        assert!(!Arc::ptr_eq(&original, &replacement));
        let mut next = presentation.clone();
        next.host_scene_data
            .document_dock
            .pane
            .ui_asset
            .preview
            .canvas
            .source_surface_frame = Some(replacement);
        ui.set_host_presentation(next);
        assert!(old.into_current_address(&ui).is_none());
        ui.set_host_presentation(presentation.clone());
        let old = queued_open(&ui);
        let mut next = presentation;
        next.host_scene_data.document_dock.pane.id = "other-active-view".into();
        ui.set_host_presentation(next);
        assert!(old.into_current_address(&ui).is_none());
    }
}
