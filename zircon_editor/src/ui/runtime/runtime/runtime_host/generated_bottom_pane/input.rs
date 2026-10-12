use std::sync::{Arc, Weak};

use zircon_runtime_interface::ui::{
    dispatch::{UiInputEvent, UiInputEventMetadata},
    event_ui::UiNodeId,
    surface::{UiPointerEventKind, UiSurfaceFrame},
    window::{UiWindowEvent, UiWindowEventKind, UiWindowEventMetadata, UiWindowInputPumpEvent},
};

use super::{
    unavailable, EditorUiHostRuntime, EditorUiHostRuntimeError, Feedback, GeneratedBottomPane,
    GeneratedBottomPaneDispatch, GeneratedBottomPaneInput,
};
use crate::ui::binding::EditorUiBinding;

pub(super) struct GeneratedPointerSource {
    initial: Weak<UiSurfaceFrame>,
    latest: Weak<UiSurfaceFrame>,
    metadata: UiInputEventMetadata,
    node: UiNodeId,
    incarnation: u64,
    root: UiNodeId,
    root_incarnation: u64,
}

impl GeneratedBottomPane {
    fn accepts_captured_source(&self, source: &Arc<UiSurfaceFrame>, event: &UiInputEvent) -> bool {
        let UiInputEvent::Pointer(pointer) = event else {
            return false;
        };
        if !matches!(
            pointer.event.kind,
            UiPointerEventKind::Move | UiPointerEventKind::Up | UiPointerEventKind::Cancel
        ) {
            return false;
        }
        let Some(proof) = &self.pointer_source else {
            return false;
        };
        let current = self.surface.surface_frame();
        [proof.initial.upgrade(), proof.latest.upgrade()]
            .into_iter()
            .flatten()
            .any(|known| Arc::ptr_eq(&known, source))
            && source.tree_id == current.tree_id
            && proof.metadata.window_id == pointer.metadata.window_id
            && proof.metadata.pointer_id == pointer.metadata.pointer_id
            && self.surface.tree.roots.contains(&proof.root)
            && self.surface.tree.node_incarnation(proof.root) == Some(proof.root_incarnation)
            && self.surface.tree.node_incarnation(proof.node) == Some(proof.incarnation)
            && (current.focus_state.captured == Some(proof.node)
                || current.focus_state.pressed == Some(proof.node))
            && self.accepts_node_input(proof.node)
    }

    fn update_pointer_source(&mut self, source: &Arc<UiSurfaceFrame>, event: &UiInputEvent) {
        let UiInputEvent::Pointer(pointer) = event else {
            return;
        };
        match pointer.event.kind {
            UiPointerEventKind::Down => {
                let current = self.surface.surface_frame();
                self.pointer_source = current
                    .focus_state
                    .captured
                    .or(current.focus_state.pressed)
                    .and_then(|node| {
                        let root = *self.surface.tree.roots.first()?;
                        Some(GeneratedPointerSource {
                            initial: Arc::downgrade(source),
                            latest: Arc::downgrade(source),
                            metadata: pointer.metadata.clone(),
                            node,
                            incarnation: self.surface.tree.node_incarnation(node)?,
                            root,
                            root_incarnation: self.surface.tree.node_incarnation(root)?,
                        })
                    });
            }
            UiPointerEventKind::Move => {
                if let Some(proof) = &mut self.pointer_source {
                    if proof.metadata.pointer_id == pointer.metadata.pointer_id
                        && proof.metadata.window_id == pointer.metadata.window_id
                    {
                        proof.latest = Arc::downgrade(source);
                    }
                }
            }
            UiPointerEventKind::Up | UiPointerEventKind::Cancel => {
                let current = self.surface.surface_frame();
                let retained = self.pointer_source.as_ref().is_some_and(|proof| {
                    current.focus_state.captured == Some(proof.node)
                        || current.focus_state.pressed == Some(proof.node)
                });
                if retained {
                    if let Some(proof) = &mut self.pointer_source {
                        proof.latest = Arc::downgrade(source);
                    }
                } else {
                    self.pointer_source = None;
                }
            }
            _ => {}
        }
    }

    fn dispatch_input(
        &mut self,
        pane_id: &str,
        source: &Arc<UiSurfaceFrame>,
        event: UiWindowInputPumpEvent,
    ) -> Result<GeneratedBottomPaneInput, EditorUiHostRuntimeError> {
        if !self.visible || self.surface.window_state.closed || self.surface.window_state.destroyed
        {
            return Err(unavailable(
                pane_id,
                "Generated pane input owner is retired",
            ));
        }
        let current = self.surface.surface_frame();
        let current_source = self.published.as_ref().is_some_and(|published| {
            Arc::ptr_eq(published, source) && Arc::ptr_eq(published, &current)
        });
        let captured = match &event {
            UiWindowInputPumpEvent::Input(input) => self.accepts_captured_source(source, input),
            UiWindowInputPumpEvent::Window(_) => false,
        };
        if !current_source && !captured {
            return Err(unavailable(
                pane_id,
                "Generated pane source/frame proof is stale",
            ));
        }
        let is_down = matches!(&event, UiWindowInputPumpEvent::Input(UiInputEvent::Pointer(pointer))
            if pointer.event.kind == UiPointerEventKind::Down);
        if is_down && self.surface.window_state.focused == Some(false) {
            let metadata = match &event {
                UiWindowInputPumpEvent::Input(UiInputEvent::Pointer(pointer)) => &pointer.metadata,
                _ => unreachable!(),
            };
            let metadata = UiWindowEventMetadata::for_window(
                metadata.window_id.clone().unwrap_or_default(),
                metadata.timestamp,
                metadata.sequence,
            )
            .synthetic(metadata.synthetic);
            self.surface.dispatch_window_input_pump_event(
                &mut self.input_manager,
                UiWindowInputPumpEvent::Window(UiWindowEvent::window_focused(metadata, true)),
            )?;
        }
        let terminal = matches!(&event, UiWindowInputPumpEvent::Window(window)
            if matches!(&window.kind, UiWindowEventKind::Focused { focused: false }
                | UiWindowEventKind::Closed | UiWindowEventKind::Destroyed));
        if terminal {
            self.surface.clear_focus();
            self.pointer_source = None;
        }
        // The Surface belongs to the pane, which can move between native windows.
        // A native terminal event cancels this input lease; actual CloseView removes the holder.
        let event = match event {
            UiWindowInputPumpEvent::Window(window)
                if matches!(
                    &window.kind,
                    UiWindowEventKind::Closed | UiWindowEventKind::Destroyed
                ) =>
            {
                UiWindowInputPumpEvent::Window(UiWindowEvent::window_focused(
                    window.metadata,
                    false,
                ))
            }
            event => event,
        };
        let result = self
            .surface
            .dispatch_window_input_pump_event(&mut self.input_manager, event)?;
        self.surface.rebuild_dirty(self.size)?;
        if !terminal {
            self.update_pointer_source(source, &result.event);
        }
        // Only the actual producer's explicit instance association resolves the canonical Editor kind/path.
        let bindings = result
            .component_events
            .iter()
            .filter_map(|report| self.binding_for_report(report))
            .collect();
        let result = Arc::new(result);
        self.last_input_result = Some(Arc::clone(&result));
        Ok(GeneratedBottomPaneInput { bindings, result })
    }
}

impl EditorUiHostRuntime {
    /// Keep the source, action epoch, and holder stable until its canonical reports and feedback finish.
    /// The callback may only observe/record through the Editor controller and return owned effects.
    pub(crate) fn dispatch_generated_bottom_pane_input<T>(
        &self,
        pane_id: &str,
        source: &Arc<UiSurfaceFrame>,
        event: UiWindowInputPumpEvent,
        mut dispatch: impl FnMut(&EditorUiBinding) -> Result<(T, Feedback), String>,
    ) -> Result<GeneratedBottomPaneDispatch<T>, EditorUiHostRuntimeError> {
        let mut registry = self
            .template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned");
        let holder = registry
            .generated_bottom_panes
            .get_mut(pane_id)
            .ok_or_else(|| unavailable(pane_id, "Generated pane input owner is absent"))?;
        let input = holder.dispatch_input(pane_id, source, event)?;
        let mut dispatched = Vec::new();
        for binding in &input.bindings {
            let (effect, updates) =
                dispatch(binding).map_err(|error| unavailable(pane_id, error))?;
            holder.apply_feedback(&updates)?;
            dispatched.push(effect);
        }
        Ok(GeneratedBottomPaneDispatch {
            dispatched,
            result: input.result,
        })
    }
}
