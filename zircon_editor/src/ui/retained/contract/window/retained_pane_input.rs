use crate::ui::retained_host::primitives::SharedString;
use std::sync::Arc;

use zircon_runtime_interface::ui::{
    dispatch::UiInputEvent,
    surface::{UiPointerEventKind, UiSurfaceFrame},
    window::{UiWindowEvent, UiWindowEventKind, UiWindowEventMetadata, UiWindowInputPumpEvent},
};

use super::super::{
    data::{FrameRect, HostWindowPresentationData, PaneData, UiAssetPreviewCanvasTransform},
    globals::PaneSurfaceHostContext,
    native_pointer::routing::{route_pointer_to_pane, PanePointerTarget},
    redraw::NativePointerDispatchResult,
};
use super::UiHostWindow;

struct PreviewTarget<'a> {
    pane_id: &'a str,
    source: &'a Arc<UiSurfaceFrame>,
    transform: UiAssetPreviewCanvasTransform,
    accepts_text: bool,
}

impl UiHostWindow {
    /// Normalized window input enters the same session Surface represented by the visible canvas.
    pub(crate) fn dispatch_retained_pane_platform_input(
        &self,
        mut event: UiInputEvent,
    ) -> Option<NativePointerDispatchResult> {
        if !self.window().is_visible() {
            return None;
        }
        let generation = self.get_host_presentation_generation();
        let presentation = generation.structure();
        if presentation.menu_state.open_menu_index >= 0
            || super::super::native_popup_dismiss::unreal_viewport_overflow_dismiss_target(
                &presentation.workbench_window_nodes,
            )
            .is_some()
            || self.chord_capture_focus_active()
        {
            return None;
        }
        let pane_id = self.state.borrow().retained_pane_input_pane.clone();
        let target = match &mut event {
            UiInputEvent::Pointer(pointer) => {
                let continuation = matches!(
                    pointer.event.kind,
                    UiPointerEventKind::Move | UiPointerEventKind::Up | UiPointerEventKind::Cancel
                );
                let captured = continuation
                    .then(|| {
                        pane_id
                            .as_ref()
                            .and_then(|id| find_preview_target(presentation, id.as_str()))
                    })
                    .flatten()
                    .filter(|target| {
                        target.source.focus_state.captured.is_some()
                            || target.source.focus_state.pressed.is_some()
                    });
                let target = captured.or_else(|| {
                    let route = route_pointer_to_pane(
                        presentation,
                        generation.pane_interaction_state(),
                        pointer.event.point.x,
                        pointer.event.point.y,
                    )?;
                    let (pane_id, source_frame, transform) = match route.target {
                        PanePointerTarget::UiAssetPreview {
                            pane_id,
                            source_frame,
                            transform,
                        }
                        | PanePointerTarget::GeneratedBottom {
                            pane_id,
                            source_frame,
                            transform,
                        } => (pane_id, source_frame, transform),
                        _ => return None,
                    };
                    Some(PreviewTarget {
                        pane_id,
                        source: source_frame,
                        transform,
                        accepts_text: false,
                    })
                });
                let Some(target) = target else {
                    if pointer.event.kind == UiPointerEventKind::Down {
                        self.leave_retained_pane_input(presentation, &event);
                    }
                    return None;
                };
                if matches!(
                    pointer.event.kind,
                    UiPointerEventKind::Up | UiPointerEventKind::Cancel
                ) && target.source.focus_state.captured.is_none()
                    && target.source.focus_state.pressed.is_none()
                {
                    return Some(NativePointerDispatchResult::idle());
                }
                // Captured continuations can leave the canvas; fresh hits require its current clip.
                pointer.event.point.x =
                    (pointer.event.point.x - target.transform.origin_x) / target.transform.scale;
                pointer.event.point.y =
                    (pointer.event.point.y - target.transform.origin_y) / target.transform.scale;
                target
            }
            UiInputEvent::DragDrop(drag) => {
                let route = route_pointer_to_pane(
                    presentation,
                    generation.pane_interaction_state(),
                    drag.point.x,
                    drag.point.y,
                )?;
                let (pane_id, source_frame, transform) = match route.target {
                    PanePointerTarget::UiAssetPreview {
                        pane_id,
                        source_frame,
                        transform,
                    }
                    | PanePointerTarget::GeneratedBottom {
                        pane_id,
                        source_frame,
                        transform,
                    } => (pane_id, source_frame, transform),
                    _ => return None,
                };
                drag.point.x = (drag.point.x - transform.origin_x) / transform.scale;
                drag.point.y = (drag.point.y - transform.origin_y) / transform.scale;
                PreviewTarget {
                    pane_id,
                    source: source_frame,
                    transform,
                    accepts_text: false,
                }
            }
            UiInputEvent::Keyboard(keyboard) => {
                let target = pane_id
                    .as_ref()
                    .and_then(|id| find_preview_target(presentation, id.as_str()))?;
                if self.text_input_focus_active() {
                    self.finish_retained_pane_input(UiWindowEventKind::Focused { focused: false });
                    return None;
                }
                let modifiers = &keyboard.metadata.modifiers;
                // Preserve editor command chords, including the current Content Drawer shortcut.
                if modifiers.alt
                    || modifiers.super_key
                    || (modifiers.control
                        && !(target.accepts_text && keyboard.logical_key.eq_ignore_ascii_case("a")))
                {
                    return None;
                }
                target
            }
            UiInputEvent::Text(_) | UiInputEvent::Ime(_) => {
                let target = pane_id
                    .as_ref()
                    .and_then(|id| find_preview_target(presentation, id.as_str()))?;
                if !target.accepts_text || self.text_input_focus_active() {
                    return None;
                }
                target
            }
            _ => return None,
        };
        let input_pane = target.pane_id.to_string();
        let clip = target.transform.clip.clone();
        let source = Arc::clone(target.source);
        let is_down = matches!(&event, UiInputEvent::Pointer(pointer) if pointer.event.kind == UiPointerEventKind::Down);
        drop(generation);
        if is_down {
            let switching = self
                .state
                .borrow()
                .retained_pane_input_pane
                .as_ref()
                .is_some_and(|previous| previous.as_str() != input_pane.as_str());
            if switching {
                // Retire the previous visible Session's input owner before replacing its address.
                self.finish_retained_pane_input(UiWindowEventKind::Focused { focused: false });
            }
            self.cancel_focused_text_input();
            self.state.borrow_mut().retained_pane_input_pane = Some(input_pane.as_str().into());
        }
        self.global::<PaneSurfaceHostContext>()
            .invoke_retained_pane_input(
                input_pane.into(),
                source,
                UiWindowInputPumpEvent::Input(event),
            );
        Some(NativePointerDispatchResult::region_with_frame_update(clip))
    }

    pub(crate) fn has_retained_pane_input_address(&self) -> bool {
        self.state.borrow().retained_pane_input_pane.is_some()
    }

    pub(crate) fn retained_pane_input_source_visible(&self, id: &str) -> bool {
        if !self.window().is_visible() {
            return false;
        }
        let generation = self.get_host_presentation_generation();
        find_preview_target(generation.structure(), id).is_some()
    }

    pub(crate) fn hidden_retained_pane_input(
        &self,
    ) -> Option<(SharedString, UiWindowEventMetadata)> {
        let id = self.state.borrow().retained_pane_input_pane.clone()?;
        if self.retained_pane_input_source_visible(id.as_str()) {
            return None;
        }
        let metadata = super::metadata::native_input_metadata(self, 0);
        Some((
            id,
            UiWindowEventMetadata::for_window(
                metadata.window_id.unwrap_or_default(),
                metadata.timestamp,
                metadata.sequence,
            )
            .synthetic(metadata.synthetic),
        ))
    }

    pub(crate) fn clear_retained_pane_input_address(&self, id: &str) {
        let mut state = self.state.borrow_mut();
        if state
            .retained_pane_input_pane
            .as_ref()
            .is_some_and(|current| current.as_str() == id)
        {
            state.retained_pane_input_pane = None;
        }
    }

    pub(crate) fn retained_pane_accepts_text_input(&self) -> bool {
        if !self.window().is_visible() {
            return false;
        }
        let Some(id) = self.state.borrow().retained_pane_input_pane.clone() else {
            return false;
        };
        let generation = self.get_host_presentation_generation();
        find_preview_target(generation.structure(), id.as_str())
            .is_some_and(|target| target.accepts_text)
            && !self.text_input_focus_active()
    }

    fn leave_retained_pane_input(
        &self,
        _presentation: &HostWindowPresentationData,
        event: &UiInputEvent,
    ) {
        let UiInputEvent::Pointer(pointer) = event else {
            return;
        };
        let metadata = UiWindowEventMetadata::for_window(
            pointer.metadata.window_id.clone().unwrap_or_default(),
            pointer.metadata.timestamp,
            pointer.metadata.sequence,
        )
        .synthetic(pointer.metadata.synthetic);
        self.dispatch_retained_pane_window_event(UiWindowEvent::window_focused(metadata, false));
    }

    pub(crate) fn dispatch_retained_pane_window_event(&self, event: UiWindowEvent) -> bool {
        let id = self.state.borrow().retained_pane_input_pane.clone();
        let generation = self.get_host_presentation_generation();
        let target = id
            .as_ref()
            .and_then(|id| find_preview_target(generation.structure(), id.as_str()));
        let Some(target) = target else {
            self.state.borrow_mut().retained_pane_input_pane = None;
            return false;
        };
        let input_pane = target.pane_id.to_string();
        let source = Arc::clone(target.source);
        if matches!(
            &event.kind,
            UiWindowEventKind::Focused { focused: false }
                | UiWindowEventKind::Closed
                | UiWindowEventKind::Destroyed
        ) {
            self.state.borrow_mut().retained_pane_input_pane = None;
        }
        drop(generation);
        self.global::<PaneSurfaceHostContext>()
            .invoke_retained_pane_input(
                input_pane.into(),
                source,
                UiWindowInputPumpEvent::Window(event),
            );
        true
    }

    pub(crate) fn finish_retained_pane_input(&self, kind: UiWindowEventKind) -> bool {
        let metadata = super::metadata::native_input_metadata(self, 0);
        self.dispatch_retained_pane_window_event(UiWindowEvent::new(
            UiWindowEventMetadata::for_window(
                metadata.window_id.unwrap_or_default(),
                metadata.timestamp,
                metadata.sequence,
            )
            .synthetic(metadata.synthetic),
            kind,
        ))
    }
}

fn preview_target<'a>(
    pane: &'a PaneData,
    content: &FrameRect,
    pane_id: &str,
) -> Option<PreviewTarget<'a>> {
    if pane.id.as_str() != pane_id {
        return None;
    }
    if pane.kind.as_str() == "GeneratedBottom" {
        return Some(PreviewTarget {
            pane_id: pane.id.as_str(),
            source: pane.generated_bottom.source_surface_frame.as_ref()?,
            transform: pane.generated_bottom.input_transform(content)?,
            accepts_text: pane.generated_bottom.accepts_text_input,
        });
    }
    if pane.kind.as_str() != "UiAssetEditor" {
        return None;
    }
    let canvas = &pane.ui_asset.preview.canvas;
    if !canvas.input_enabled {
        return None;
    }
    Some(PreviewTarget {
        pane_id: pane.id.as_str(),
        source: canvas.source_surface_frame.as_ref()?,
        transform: canvas.transform(&pane.ui_asset.designer_canvas_panel_node, content)?,
        accepts_text: canvas.accepts_text_input,
    })
}

fn find_retained_input_pane<'a>(
    presentation: &'a HostWindowPresentationData,
    id: &str,
) -> Option<(&'a PaneData, FrameRect)> {
    use super::super::native_pointer::routing::{
        floating_window_content_frame, side_dock_content_frame, translated,
    };
    let matches = |pane: &PaneData| {
        pane.id.as_str() == id && matches!(pane.kind.as_str(), "UiAssetEditor" | "GeneratedBottom")
    };
    if presentation.host_shell.native_floating_window_mode {
        for window in presentation
            .native_floating_surface_data
            .floating_windows
            .iter()
        {
            if window.window_id.as_str() == presentation.host_shell.native_floating_window_id
                && matches(&window.active_pane)
            {
                return Some((
                    &window.active_pane,
                    floating_window_content_frame(&window.frame, &window.header_frame),
                ));
            }
        }
        return None;
    }
    let scene = &presentation.host_scene_data;
    for window in scene.floating_layer.floating_windows.iter().rev() {
        if matches(&window.active_pane) {
            return Some((
                &window.active_pane,
                floating_window_content_frame(&window.frame, &window.header_frame),
            ));
        }
    }
    for dock in scene.document_surfaces() {
        if matches(&dock.pane) {
            return Some((
                &dock.pane,
                translated(
                    &dock.content_frame,
                    dock.region_frame.x,
                    dock.region_frame.y,
                ),
            ));
        }
    }
    if !super::super::componentized_workbench_regions::owns_ordinary_panes(presentation) {
        for dock in [&scene.left_dock, &scene.right_dock] {
            if matches(&dock.pane) {
                return Some((&dock.pane, side_dock_content_frame(dock)));
            }
        }
    }
    let dock = &scene.bottom_dock;
    matches(&dock.pane).then(|| {
        (
            &dock.pane,
            translated(
                &dock.content_frame,
                dock.region_frame.x,
                dock.region_frame.y,
            ),
        )
    })
}

fn find_preview_target<'a>(
    presentation: &'a HostWindowPresentationData,
    id: &str,
) -> Option<PreviewTarget<'a>> {
    let (pane, content) = find_retained_input_pane(presentation, id)?;
    preview_target(pane, &content, id)
}

#[cfg(test)]
impl UiHostWindow {
    pub(crate) fn ui_asset_palette_control_point_for_test(
        &self,
        id: &str,
        control: &str,
    ) -> zircon_runtime_interface::ui::layout::UiPoint {
        let generation = self.get_host_presentation_generation();
        let (pane, content) =
            find_retained_input_pane(generation.structure(), id).expect("visible UI asset pane");
        let node = pane
            .ui_asset
            .nodes
            .iter()
            .find(|node| node.control_id.as_str() == control)
            .expect("actual palette control in published shell");
        assert!(!node.disabled);
        zircon_runtime_interface::ui::layout::UiPoint::new(
            content.x + node.frame.x + node.frame.width * 0.5,
            content.y + node.frame.y + node.frame.height * 0.5,
        )
    }

    pub(crate) fn ui_asset_preview_source_for_test(&self, id: &str) -> Arc<UiSurfaceFrame> {
        let generation = self.get_host_presentation_generation();
        let (pane, _) =
            find_retained_input_pane(generation.structure(), id).expect("published UI asset pane");
        pane.ui_asset
            .preview
            .canvas
            .source_surface_frame
            .as_ref()
            .expect("published PreviewHost source")
            .clone()
    }

    /// Records the actual published window through the production pane painter.
    pub(crate) fn ui_asset_preview_paint_sources_for_test(
        &self,
    ) -> Vec<(
        Arc<UiSurfaceFrame>,
        zircon_runtime_interface::ui::surface::UiRenderFrameCommandRef,
        FrameRect,
    )> {
        let generation = self.get_host_presentation_generation();
        let presentation = generation.structure();
        let size = self.window().size();
        let mut frame =
            super::super::paint_frame::HostRgbaFrame::recording_only(size.width, size.height);
        super::super::paint_workbench::draw_workbench_presentation_commands(
            &mut frame,
            presentation,
        );
        let recorded = frame.into_recorded_frame();
        recorded
            .commands
            .iter()
            .filter_map(|command| {
                if command.frame.width <= 0.0 || command.frame.height <= 0.0 {
                    return None;
                }
                if command.clip_frame.as_ref().is_some_and(|clip| {
                    super::super::paint_geometry::intersect(&command.frame, clip).is_none()
                }) {
                    return None;
                }
                let source = command.source?;
                let frame = recorded.render_sources.resolve(source.surface_key)?;
                Some((Arc::clone(frame), source.command_ref, command.frame.clone()))
            })
            .collect()
    }

    pub(crate) fn ui_asset_preview_point_for_test(
        &self,
        id: &str,
        point: zircon_runtime_interface::ui::layout::UiPoint,
    ) -> zircon_runtime_interface::ui::layout::UiPoint {
        let generation = self.get_host_presentation_generation();
        let target = find_preview_target(generation.structure(), id)
            .expect("visible UiAsset preview canvas");
        zircon_runtime_interface::ui::layout::UiPoint::new(
            target.transform.origin_x + point.x * target.transform.scale,
            target.transform.origin_y + point.y * target.transform.scale,
        )
    }
}

#[cfg(test)]
impl UiHostWindow {
    pub(crate) fn generated_bottom_pane_for_test(
        &self,
        id: &str,
    ) -> super::super::data::GeneratedBottomPaneData {
        let generation = self.get_host_presentation_generation();
        let (pane, content) = find_retained_input_pane(generation.structure(), id)
            .expect("actual visible Generated native pane");
        assert_eq!(pane.kind.as_str(), "GeneratedBottom");
        eprintln!(
            "generated-bottom-input: id={id} window=({},{}) content=({},{},{},{}) source_size=({},{}) source_present={} source_ptr={:?} node_count={} status={}",
            self.window().size().width, self.window().size().height,
            content.x, content.y, content.width, content.height,
            pane.generated_bottom.source_width, pane.generated_bottom.source_height,
            pane.generated_bottom.source_surface_frame.is_some(),
            pane.generated_bottom.source_surface_frame.as_ref().map(std::sync::Arc::as_ptr),
            pane.generated_bottom.nodes.row_count(),
            pane.generated_bottom.status.as_str(),
        );
        assert!(pane.generated_bottom.input_transform(&content).is_some());
        pane.generated_bottom.clone()
    }

    pub(crate) fn generated_bottom_control_point_for_test(
        &self,
        id: &str,
        control: &str,
    ) -> Option<zircon_runtime_interface::ui::layout::UiPoint> {
        let generation = self.get_host_presentation_generation();
        let (pane, content) = find_retained_input_pane(generation.structure(), id)?;
        if pane.kind.as_str() != "GeneratedBottom" {
            return None;
        }
        let transform = pane.generated_bottom.input_transform(&content)?;
        let source = pane.generated_bottom.source_surface_frame.as_ref()?;
        let node = source
            .arranged_tree
            .nodes
            .iter()
            .find(|node| node.control_id.as_deref() == Some(control))?;
        let mut x = node.frame.x.max(0.0);
        let mut y = node.frame.y.max(0.0);
        let mut right = (node.frame.x + node.frame.width).min(pane.generated_bottom.source_width);
        let mut bottom =
            (node.frame.y + node.frame.height).min(pane.generated_bottom.source_height);
        let clip = &node.clip_frame;
        x = x.max(clip.x);
        y = y.max(clip.y);
        right = right.min(clip.x + clip.width);
        bottom = bottom.min(clip.y + clip.height);
        if right <= x || bottom <= y {
            return None;
        }
        let point = zircon_runtime_interface::ui::layout::UiPoint::new(
            transform.origin_x + (x + right) * 0.5,
            transform.origin_y + (y + bottom) * 0.5,
        );
        transform.source_point(point.x, point.y)?;
        Some(point)
    }
}
