use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::ui::retained_host::primitives::{
    CloseRequestResponse, ModelRc, PhysicalPosition, PhysicalSize, PlatformError, SharedString,
    VecModel,
};
use winit::application::ApplicationHandler;
use winit::event::{
    ButtonSource, ElementState, Ime, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent,
};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};
use zircon_runtime::diagnostic_log::{
    diagnostic_log_allows, write_diagnostic_log, write_error, DiagnosticLogLevel,
};
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::data::{
    FrameRect, HostClosePromptData, HostMenuStateData, HostPaneInteractionStateData,
    HostTextInputFocusData, HostWindowBootstrapData, HostWindowPresentationData, PaneData,
    TemplatePaneNodeData,
};
use super::diagnostics::{HostInvalidationDiagnostics, HostRefreshDiagnostics};
use super::globals::{
    HostContractGlobal, HostContractState, PaneSurfaceHostContext, UiHostContext,
};
use super::native_pointer::{
    dispatch_native_pointer_button, dispatch_native_pointer_move, dispatch_native_pointer_scroll,
    NativePointerButtonState,
};
use super::painter::{paint_host_frame, HostRgbaFrame};
use super::presenter::{create_host_chrome_presenter, HostChromePresenter, HostPresenterBackend};
use super::redraw::{HostRedrawRequest, NativePointerDispatchResult};
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, record_current_ui_perf_counter, UiPerfCounter, UiPerfScenario,
};

const DEFAULT_HOST_WINDOW_WIDTH: u32 = 1280;
const DEFAULT_HOST_WINDOW_HEIGHT: u32 = 720;

#[derive(Clone)]
pub(crate) struct UiHostWindow {
    state: Rc<RefCell<HostContractState>>,
}

impl UiHostWindow {
    pub(crate) fn new() -> Result<Self, PlatformError> {
        Ok(Self {
            state: Rc::new(RefCell::new(HostContractState::new(PhysicalSize::new(
                DEFAULT_HOST_WINDOW_WIDTH,
                DEFAULT_HOST_WINDOW_HEIGHT,
            )))),
        })
    }

    pub(crate) fn clone_strong(&self) -> Self {
        self.clone()
    }

    pub(crate) fn show(&self) -> Result<(), PlatformError> {
        self.state.borrow_mut().window_visible = true;
        Ok(())
    }

    pub(crate) fn hide(&self) -> Result<(), PlatformError> {
        self.state.borrow_mut().window_visible = false;
        Ok(())
    }

    pub(crate) fn run(&self) -> Result<(), PlatformError> {
        let event_loop = EventLoop::new().map_err(platform_error)?;
        let app = UiHostWindowEventLoop::new(self.clone_strong());
        event_loop.run_app(app).map_err(platform_error)
    }

    pub(crate) fn window(&self) -> HostWindowHandle {
        HostWindowHandle {
            state: self.state.clone(),
        }
    }

    fn close_requested_response(&self) -> CloseRequestResponse {
        let callback = self.state.borrow().close_requested.clone();
        callback
            .as_ref()
            .map(|callback| callback())
            .unwrap_or(CloseRequestResponse::HideWindow)
    }

    pub(crate) fn global<T>(&self) -> T
    where
        T: HostContractGlobal,
    {
        T::from_state(self.state.clone())
    }

    pub(crate) fn set_host_presentation(&self, presentation: HostWindowPresentationData) {
        let mut state = self.state.borrow_mut();
        state.presentation_rebuild_count = state.presentation_rebuild_count.saturating_add(1);
        record_current_ui_perf_counter(UiPerfCounter::PresentationRebuildCount, 1.0);
        if state.presentation_rebuild_count <= 8
            || state.presentation_rebuild_count.is_power_of_two()
        {
            if diagnostic_log_allows(DiagnosticLogLevel::Verbose) {
                write_diagnostic_log(
                    "editor_host_window",
                    format!(
                        "set_host_presentation count={} project_path={} viewport_label={} status={} center={} document={} viewport={}",
                        state.presentation_rebuild_count,
                        presentation.host_shell.project_path,
                        presentation.host_shell.viewport_label,
                        presentation.host_shell.status_secondary,
                        frame_summary(&presentation.host_layout.center_band_frame),
                        frame_summary(&presentation.host_layout.document_region_frame),
                        frame_summary(&presentation.host_layout.viewport_content_frame)
                    ),
                );
            }
        }
        state.host_presentation = presentation;
    }

    pub(crate) fn get_host_presentation(&self) -> HostWindowPresentationData {
        let state = self.state.borrow();
        host_presentation_from_state(&state)
    }

    pub(crate) fn set_close_prompt(&self, prompt: HostClosePromptData) {
        let damage = {
            let mut state = self.state.borrow_mut();
            let current = state.host_presentation.close_prompt.clone();
            let damage = if current.visible {
                current.overlay_frame
            } else {
                prompt.overlay_frame.clone()
            };
            state.host_presentation.close_prompt = prompt;
            damage
        };
        self.queue_external_redraw(HostRedrawRequest::region(damage));
    }

    pub(crate) fn clear_close_prompt(&self) {
        self.set_close_prompt(HostClosePromptData::default());
    }

    pub(crate) fn set_host_refresh_invalidation_diagnostics(
        &self,
        diagnostics: HostInvalidationDiagnostics,
    ) {
        self.state.borrow_mut().refresh_invalidation_diagnostics = diagnostics;
    }

    fn refresh_invalidation_diagnostics(&self) -> HostInvalidationDiagnostics {
        self.state.borrow().refresh_invalidation_diagnostics
    }

    fn set_host_refresh_diagnostics_overlay(&self, diagnostics: HostRefreshDiagnostics) {
        self.set_host_refresh_diagnostics_overlay_text(diagnostics.overlay_text().into());
    }

    fn set_host_refresh_diagnostics_overlay_text(
        &self,
        overlay_text: crate::ui::retained_host::primitives::SharedString,
    ) {
        let mut state = self.state.borrow_mut();
        state.host_presentation.host_shell.debug_refresh_rate = overlay_text;
    }

    pub(crate) fn get_menu_state(&self) -> HostMenuStateData {
        self.state.borrow().menu_state.clone()
    }

    pub(crate) fn get_pane_interaction_state(&self) -> HostPaneInteractionStateData {
        self.state.borrow().pane_interaction_state.clone()
    }

    pub(crate) fn set_hovered_template_node_for_pointer_move(
        &self,
        control_id: SharedString,
        frame: FrameRect,
    ) {
        let mut state = self.state.borrow_mut();
        state.pane_interaction_state.hovered_template_control_id = control_id;
        state.pane_interaction_state.hovered_template_frame = frame;
    }

    pub(crate) fn clear_hovered_template_node_for_pointer_move(&self) {
        let mut state = self.state.borrow_mut();
        state
            .pane_interaction_state
            .hovered_template_control_id
            .clear();
        state.pane_interaction_state.hovered_template_frame = FrameRect::default();
    }

    pub(crate) fn get_host_window_bootstrap(&self) -> HostWindowBootstrapData {
        let state = self.state.borrow();
        HostWindowBootstrapData {
            shell_frame: FrameRect {
                x: 0.0,
                y: 0.0,
                width: state.window_size.width as f32,
                height: state.window_size.height as f32,
            },
            viewport_content_frame: state
                .host_presentation
                .host_layout
                .viewport_content_frame
                .clone(),
        }
    }

    pub(crate) fn request_frame_update(&self) {
        self.global::<super::globals::UiHostContext>()
            .invoke_frame_requested();
    }

    pub(crate) fn request_redraw_region(&self, frame: FrameRect) {
        self.queue_external_redraw(HostRedrawRequest::region(frame));
    }

    pub(crate) fn request_frame_update_region(&self, frame: FrameRect) {
        let redraw = HostRedrawRequest::region_with_frame_update(frame);
        if redraw.request_redraw() {
            self.queue_external_redraw(redraw);
        } else {
            self.queue_external_redraw(HostRedrawRequest::full_frame());
        }
    }

    pub(crate) fn text_input_focus_active(&self) -> bool {
        self.state.borrow().text_input_focus.is_active()
    }

    pub(crate) fn request_exit(&self) {
        let mut state = self.state.borrow_mut();
        state.window_visible = false;
        state.exit_requested = true;
    }

    #[cfg(test)]
    pub(crate) fn exit_requested_for_test(&self) -> bool {
        self.state.borrow().exit_requested
    }

    pub(crate) fn dispatch_focused_text_insert(&self, text: &str) -> NativePointerDispatchResult {
        let text: String = text.chars().filter(|ch| !ch.is_control()).collect();
        if text.is_empty() {
            return NativePointerDispatchResult::idle();
        }
        let (focus, value) = {
            let mut state = self.state.borrow_mut();
            let focus = state.text_input_focus.clone();
            if !focus.is_active() {
                return NativePointerDispatchResult::idle();
            }
            let mut value = focus.value_text.to_string();
            value.push_str(&text);
            state.text_input_focus.value_text = value.clone().into();
            (focus, value)
        };
        self.dispatch_text_focus_value(focus.clone(), focus.edit_target_id(), value)
    }

    pub(crate) fn dispatch_focused_text_backspace(&self) -> NativePointerDispatchResult {
        let (focus, value) = {
            let mut state = self.state.borrow_mut();
            let focus = state.text_input_focus.clone();
            if !focus.is_active() {
                return NativePointerDispatchResult::idle();
            }
            let mut value = focus.value_text.to_string();
            if value.pop().is_none() {
                return NativePointerDispatchResult::idle();
            }
            state.text_input_focus.value_text = value.clone().into();
            (focus, value)
        };
        self.dispatch_text_focus_value(focus.clone(), focus.edit_target_id(), value)
    }

    fn dispatch_focused_key_event(&self, event: &KeyEvent) -> NativePointerDispatchResult {
        if event.state != ElementState::Pressed {
            return NativePointerDispatchResult::idle();
        }
        match &event.logical_key {
            Key::Named(NamedKey::Backspace) => self.dispatch_focused_text_backspace(),
            Key::Named(NamedKey::Escape) => {
                self.global::<UiHostContext>().clear_text_input_focus();
                NativePointerDispatchResult::idle()
            }
            Key::Named(NamedKey::Enter) => self.dispatch_focused_text_commit(),
            _ => event
                .text
                .as_deref()
                .map_or_else(NativePointerDispatchResult::idle, |text| {
                    self.dispatch_focused_text_insert(text)
                }),
        }
    }

    fn dispatch_focused_text_commit(&self) -> NativePointerDispatchResult {
        let focus = self.state.borrow().text_input_focus.clone();
        if !focus.is_active() || focus.commit_action_id.is_empty() {
            return NativePointerDispatchResult::idle();
        }
        self.dispatch_text_focus_value(
            focus.clone(),
            focus.commit_target_id(),
            focus.value_text.to_string(),
        )
    }

    fn dispatch_text_focus_value(
        &self,
        focus: super::data::HostTextInputFocusData,
        target_id: crate::ui::retained_host::primitives::SharedString,
        value: String,
    ) -> NativePointerDispatchResult {
        let value: crate::ui::retained_host::primitives::SharedString = value.into();
        let control_id = focus.control_id.clone();
        let pane_host = self.global::<PaneSurfaceHostContext>();
        match focus.dispatch_kind.as_str() {
            "welcome_text" => pane_host.invoke_welcome_control_changed(target_id, value),
            "showcase" => {
                pane_host.invoke_component_showcase_control_edited(control_id, target_id, value)
            }
            "inspector" => pane_host.invoke_inspector_control_changed(control_id, value),
            kind if asset_dispatch_source(kind).is_some() => pane_host
                .invoke_asset_control_changed(
                    asset_dispatch_source(kind).unwrap_or("activity").into(),
                    control_id,
                    value,
                ),
            "commit_only" if target_id == focus.edit_target_id() => {
                return text_input_focus_redraw(&focus);
            }
            "commit_only" => pane_host.invoke_surface_control_edited(control_id, target_id, value),
            _ if !focus.edit_action_id.is_empty() => {
                pane_host.invoke_surface_control_edited(control_id, target_id, value)
            }
            _ => return NativePointerDispatchResult::idle(),
        }
        text_input_focus_redraw(&focus)
    }

    fn queue_external_redraw(&self, redraw: HostRedrawRequest) {
        if !redraw.request_redraw() {
            return;
        }
        let mut state = self.state.borrow_mut();
        let existing = std::mem::replace(
            &mut state.external_redraw_request,
            HostRedrawRequest::none(),
        );
        if existing.request_redraw() {
            state.external_redraw_coalesced_count =
                state.external_redraw_coalesced_count.saturating_add(1);
        }
        state.external_redraw_request = existing.merge(redraw);
        state.external_redraw_queued_count = state.external_redraw_queued_count.saturating_add(1);
    }

    fn take_external_redraw(&self) -> HostRedrawRequest {
        let mut state = self.state.borrow_mut();
        let redraw = std::mem::replace(
            &mut state.external_redraw_request,
            HostRedrawRequest::none(),
        );
        if redraw.request_redraw() {
            state.external_redraw_drained_count =
                state.external_redraw_drained_count.saturating_add(1);
        }
        redraw
    }

    #[cfg(test)]
    pub(crate) fn request_host_frame_for_test(&self) {
        self.request_frame_update();
    }

    #[cfg(test)]
    pub(crate) fn presentation_rebuild_count_for_test(&self) -> u64 {
        self.state.borrow().presentation_rebuild_count
    }

    #[cfg(test)]
    pub(crate) fn dispatch_native_pointer_move_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        dispatch_native_pointer_move(self, x, y)
    }

    #[cfg(test)]
    pub(crate) fn dispatch_native_primary_press_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        dispatch_native_pointer_button(
            self,
            NativePointerButtonState::Pressed,
            Some(UiPointerButton::Primary),
            x,
            y,
        )
    }

    #[cfg(test)]
    pub(crate) fn dispatch_native_primary_release_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        dispatch_native_pointer_button(
            self,
            NativePointerButtonState::Released,
            Some(UiPointerButton::Primary),
            x,
            y,
        )
    }

    #[cfg(test)]
    pub(crate) fn dispatch_native_secondary_press_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        dispatch_native_pointer_button(
            self,
            NativePointerButtonState::Pressed,
            Some(UiPointerButton::Secondary),
            x,
            y,
        )
    }

    #[cfg(test)]
    pub(crate) fn dispatch_native_middle_press_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        dispatch_native_pointer_button(
            self,
            NativePointerButtonState::Pressed,
            Some(UiPointerButton::Middle),
            x,
            y,
        )
    }

    #[cfg(test)]
    pub(crate) fn dispatch_native_pointer_scroll_for_test(
        &self,
        x: f32,
        y: f32,
        delta: f32,
    ) -> NativePointerDispatchResult {
        dispatch_native_pointer_scroll(self, x, y, delta)
    }

    #[cfg(test)]
    pub(crate) fn dispatch_native_text_input_for_test(
        &self,
        text: &str,
    ) -> NativePointerDispatchResult {
        self.dispatch_focused_text_insert(text)
    }

    #[cfg(test)]
    pub(crate) fn dispatch_native_backspace_for_test(&self) -> NativePointerDispatchResult {
        self.dispatch_focused_text_backspace()
    }

    #[cfg(test)]
    pub(crate) fn dispatch_native_enter_for_test(&self) -> NativePointerDispatchResult {
        self.dispatch_focused_text_commit()
    }
}

#[derive(Clone)]
pub(crate) struct HostWindowHandle {
    state: Rc<RefCell<HostContractState>>,
}

impl HostWindowHandle {
    pub(crate) fn set_position(&self, position: PhysicalPosition) {
        self.state.borrow_mut().window_position = position;
    }

    pub(crate) fn set_size(&self, size: PhysicalSize) {
        self.state.borrow_mut().window_size = size;
    }

    pub(crate) fn size(&self) -> PhysicalSize {
        self.state.borrow().window_size.clone()
    }

    pub(crate) fn is_visible(&self) -> bool {
        self.state.borrow().window_visible
    }

    pub(crate) fn set_maximized(&self, maximized: bool) {
        self.state.borrow_mut().window_maximized = maximized;
    }

    pub(crate) fn is_maximized(&self) -> bool {
        self.state.borrow().window_maximized
    }

    pub(crate) fn on_close_requested(&self, callback: impl Fn() -> CloseRequestResponse + 'static) {
        self.state.borrow_mut().close_requested = Some(Rc::new(callback));
    }

    pub(crate) fn take_snapshot(&self) -> Result<HostWindowSnapshot, PlatformError> {
        let state = self.state.borrow();
        let presentation = host_presentation_from_state(&state);
        let frame = paint_host_frame(
            state.window_size.width,
            state.window_size.height,
            &presentation,
        );
        Ok(HostWindowSnapshot::from_rgba_frame(frame))
    }
}

fn host_presentation_from_state(state: &HostContractState) -> HostWindowPresentationData {
    let mut presentation = state.host_presentation.clone();
    presentation.menu_state = state.menu_state.clone();
    presentation.pane_interaction_state = state.pane_interaction_state.clone();
    presentation.text_input_focus = state.text_input_focus.clone();
    presentation.viewport_image = state.viewport_image.clone();
    apply_template_hover_to_presentation(&mut presentation, &state.pane_interaction_state);
    presentation
}

fn apply_template_hover_to_presentation(
    presentation: &mut HostWindowPresentationData,
    interaction: &HostPaneInteractionStateData,
) {
    if interaction.hovered_template_control_id.is_empty() {
        return;
    }
    let hovered = &interaction.hovered_template_control_id;
    apply_template_hover_to_pane(
        &mut presentation.host_scene_data.document_dock.pane,
        hovered,
    );
    apply_template_hover_to_pane(&mut presentation.host_scene_data.left_dock.pane, hovered);
    apply_template_hover_to_pane(&mut presentation.host_scene_data.right_dock.pane, hovered);
    apply_template_hover_to_pane(&mut presentation.host_scene_data.bottom_dock.pane, hovered);

    let mut floating_changed = false;
    let floating_windows: Vec<_> = (0..presentation
        .host_scene_data
        .floating_layer
        .floating_windows
        .row_count())
        .filter_map(|row| {
            presentation
                .host_scene_data
                .floating_layer
                .floating_windows
                .row_data(row)
        })
        .map(|mut window| {
            floating_changed |= apply_template_hover_to_pane(&mut window.active_pane, hovered);
            window
        })
        .collect();
    if floating_changed {
        presentation.host_scene_data.floating_layer.floating_windows =
            ModelRc::from(Rc::new(VecModel::from(floating_windows)));
    }
}

fn apply_template_hover_to_pane(pane: &mut PaneData, hovered: &SharedString) -> bool {
    let nodes = match pane.kind.as_str() {
        "Hierarchy" => &mut pane.hierarchy.nodes,
        "Inspector" => &mut pane.inspector.nodes,
        "Console" => &mut pane.console.nodes,
        "Assets" => &mut pane.assets_activity.nodes,
        "AssetBrowser" => &mut pane.asset_browser.nodes,
        "Welcome" => &mut pane.welcome.nodes,
        "Project" | "UiComponentShowcase" => &mut pane.project_overview.nodes,
        "RuntimeDiagnostics" => &mut pane.runtime_diagnostics.nodes,
        "PerformanceTimeline" => &mut pane.performance_timeline.nodes,
        "ModulePlugins" => &mut pane.module_plugins.nodes,
        "BuildExport" => &mut pane.build_export.nodes,
        "UiAssetEditor" => &mut pane.ui_asset.nodes,
        "AnimationSequenceEditor" | "AnimationGraphEditor" => &mut pane.animation.nodes,
        _ => return false,
    };
    apply_template_hover_to_nodes(nodes, hovered)
}

fn apply_template_hover_to_nodes(
    nodes: &mut ModelRc<TemplatePaneNodeData>,
    hovered: &SharedString,
) -> bool {
    let mut changed = false;
    let values: Vec<_> = (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .map(|mut node| {
            if node.control_id.as_str() == hovered.as_str() && !node.hovered {
                node.hovered = true;
                changed = true;
            }
            node
        })
        .collect();
    if changed {
        *nodes = ModelRc::from(Rc::new(VecModel::from(values)));
    }
    changed
}

fn asset_dispatch_source(dispatch_kind: &str) -> Option<&str> {
    if dispatch_kind == "asset" {
        return Some("activity");
    }
    dispatch_kind.strip_prefix("asset:")
}

fn text_input_focus_redraw(focus: &HostTextInputFocusData) -> NativePointerDispatchResult {
    let result = NativePointerDispatchResult::region(focus.edit_frame.clone());
    if result.request_redraw() {
        result
    } else {
        NativePointerDispatchResult::full_frame()
    }
}

struct UiHostWindowEventLoop {
    host: UiHostWindow,
    window: Option<Arc<dyn Window>>,
    presenter: Option<Box<dyn HostChromePresenter>>,
    last_pointer_position: Option<(f32, f32)>,
    pending_redraw: HostRedrawRequest,
    ime_allowed: bool,
}

impl UiHostWindowEventLoop {
    fn new(host: UiHostWindow) -> Self {
        Self {
            host,
            window: None,
            presenter: None,
            last_pointer_position: None,
            pending_redraw: HostRedrawRequest::full_frame_for_scenario(
                UiPerfScenario::Startup,
                true,
            ),
            ime_allowed: false,
        }
    }

    fn sync_host_window_state(&self, window: &dyn Window) {
        let size = window.surface_size();
        let mut state = self.host.state.borrow_mut();
        state.window_size = PhysicalSize::new(size.width, size.height);
        state.window_visible = true;
        state.window_maximized = window.is_maximized();
        if let Ok(position) = window.outer_position() {
            state.window_position = PhysicalPosition::new(position.x, position.y);
        }
    }

    fn dispatch_pointer_result(&mut self, result: NativePointerDispatchResult) {
        let redraw = result.redraw();
        if redraw.request_redraw() {
            self.queue_redraw(redraw);
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
    }

    fn queue_redraw(&mut self, redraw: HostRedrawRequest) {
        self.pending_redraw = self.pending_redraw.clone().merge(redraw);
    }

    fn take_pending_redraw(&mut self) -> HostRedrawRequest {
        std::mem::replace(&mut self.pending_redraw, HostRedrawRequest::None)
    }

    fn drain_external_redraw_request(&mut self) {
        let redraw = self.host.take_external_redraw();
        if redraw.request_redraw() {
            self.queue_redraw(redraw);
            if let Some(window) = self.window.as_ref() {
                schedule_native_redraw(window.as_ref());
            }
        }
    }

    fn sync_ime_allowed(&mut self) {
        let allowed = self.host.text_input_focus_active();
        if self.ime_allowed == allowed {
            return;
        }
        if let Some(window) = self.window.as_ref() {
            set_window_ime_allowed(window.as_ref(), allowed);
            self.ime_allowed = allowed;
        }
    }
}

impl ApplicationHandler for UiHostWindowEventLoop {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let size = self.host.window().size();
        let window_attributes = WindowAttributes::default()
            .with_title("Zircon Editor")
            .with_surface_size(winit::dpi::LogicalSize::new(
                size.width as f64,
                size.height as f64,
            ));
        let window: Arc<dyn Window> = match event_loop.create_window(window_attributes) {
            Ok(window) => Arc::from(window),
            Err(_) => {
                write_error("editor_host_window", "failed to create native window");
                event_loop.exit();
                return;
            }
        };
        self.sync_host_window_state(window.as_ref());
        let presenter_backend = HostPresenterBackend::default_native();
        let (presenter_backend, presenter) =
            match create_host_chrome_presenter(presenter_backend, window.clone()) {
                Ok(presenter) => (presenter_backend, presenter),
                Err(error) if presenter_backend.is_gpu() => {
                    write_error(
                        "editor_host_window",
                        format!(
                            "failed to create {} presenter, falling back to softbuffer: {error}",
                            presenter_backend.label()
                        ),
                    );
                    let fallback_backend = HostPresenterBackend::fallback();
                    match create_host_chrome_presenter(fallback_backend, window.clone()) {
                        Ok(presenter) => (fallback_backend, presenter),
                        Err(error) => {
                            write_error(
                                "editor_host_window",
                                format!(
                                    "failed to create {} presenter: {error}",
                                    fallback_backend.label()
                                ),
                            );
                            event_loop.exit();
                            return;
                        }
                    }
                }
                Err(error) => {
                    write_error(
                        "editor_host_window",
                        format!(
                            "failed to create {} presenter: {error}",
                            presenter_backend.label()
                        ),
                    );
                    event_loop.exit();
                    return;
                }
            };
        if diagnostic_log_allows(DiagnosticLogLevel::Verbose) {
            write_diagnostic_log(
                "editor_host_window",
                format!(
                    "created native window size={}x{} presenter_backend={}",
                    size.width,
                    size.height,
                    presenter_backend.label()
                ),
            );
        }
        window.request_redraw();
        self.window = Some(window);
        self.presenter = Some(presenter);
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                let response = self.host.close_requested_response();
                if matches!(response, CloseRequestResponse::HideWindow) {
                    self.host.state.borrow_mut().window_visible = false;
                    event_loop.exit();
                }
            }
            WindowEvent::SurfaceResized(size) => {
                self.host
                    .window()
                    .set_size(PhysicalSize::new(size.width, size.height));
                self.queue_redraw(HostRedrawRequest::full_frame_for_scenario(
                    UiPerfScenario::Startup,
                    true,
                ));
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
                if let Some(presenter) = self.presenter.as_mut() {
                    if let Err(error) = presenter.resize((size.width, size.height)) {
                        write_error(
                            "editor_host_window",
                            format!(
                                "presenter resize failed size={}x{}: {error}",
                                size.width, size.height
                            ),
                        );
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::Moved(position) => {
                self.host
                    .window()
                    .set_position(PhysicalPosition::new(position.x, position.y));
            }
            WindowEvent::PointerMoved { position, .. } => {
                self.last_pointer_position = Some((position.x as f32, position.y as f32));
                self.dispatch_pointer_result(dispatch_native_pointer_move(
                    &self.host,
                    position.x as f32,
                    position.y as f32,
                ));
            }
            WindowEvent::PointerButton {
                state,
                button,
                position,
                ..
            } => {
                self.last_pointer_position = Some((position.x as f32, position.y as f32));
                if let Some(state) = pointer_button_state(state) {
                    let result = dispatch_native_pointer_button(
                        &self.host,
                        state,
                        pointer_button(button),
                        position.x as f32,
                        position.y as f32,
                    );
                    self.dispatch_pointer_result(result);
                    self.sync_ime_allowed();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let result = self.host.dispatch_focused_key_event(&event);
                self.dispatch_pointer_result(result);
                self.sync_ime_allowed();
            }
            WindowEvent::Ime(Ime::Commit(text)) => {
                let result = self.host.dispatch_focused_text_insert(&text);
                self.dispatch_pointer_result(result);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (x, y) = self.last_pointer_position.unwrap_or((0.0, 0.0));
                self.dispatch_pointer_result(dispatch_native_pointer_scroll(
                    &self.host,
                    x,
                    y,
                    scroll_delta(delta),
                ));
            }
            WindowEvent::RedrawRequested => {
                let redraw = self.take_pending_redraw();
                if !redraw.request_redraw() {
                    return;
                }
                let _scenario = enter_ui_perf_scenario(redraw.scenario());
                if redraw.requires_frame_update() {
                    self.host.request_frame_update();
                }
                if let Some(presenter) = self.presenter.as_mut() {
                    let presentation = self.host.get_host_presentation();
                    let invalidation = self.host.refresh_invalidation_diagnostics();
                    match presenter.present(
                        &presentation,
                        redraw.damage_region().cloned(),
                        invalidation,
                    ) {
                        Ok(diagnostics) => {
                            self.host.set_host_refresh_diagnostics_overlay(diagnostics)
                        }
                        Err(error) => {
                            write_error(
                                "editor_host_window",
                                format!("presenter present failed: {error}"),
                            );
                            event_loop.exit();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &dyn ActiveEventLoop) {
        if self.host.state.borrow().exit_requested {
            _event_loop.exit();
            return;
        }
        if let Some(window) = self.window.as_ref() {
            self.sync_host_window_state(window.as_ref());
        }
        self.drain_external_redraw_request();
    }
}

fn schedule_native_redraw(window: &dyn Window) {
    window.request_redraw();
}

fn frame_summary(frame: &FrameRect) -> String {
    format!(
        "{:.1},{:.1},{:.1},{:.1}",
        frame.x, frame.y, frame.width, frame.height
    )
}

fn platform_error(error: impl std::fmt::Display) -> PlatformError {
    PlatformError::Other(error.to_string())
}

fn pointer_button(button: ButtonSource) -> Option<UiPointerButton> {
    match button.mouse_button() {
        Some(MouseButton::Left) => Some(UiPointerButton::Primary),
        Some(MouseButton::Right) => Some(UiPointerButton::Secondary),
        Some(MouseButton::Middle) => Some(UiPointerButton::Middle),
        _ => None,
    }
}

fn pointer_button_state(state: ElementState) -> Option<NativePointerButtonState> {
    match state {
        ElementState::Pressed => Some(NativePointerButtonState::Pressed),
        ElementState::Released => Some(NativePointerButtonState::Released),
    }
}

fn scroll_delta(delta: MouseScrollDelta) -> f32 {
    match delta {
        MouseScrollDelta::LineDelta(_, y) => y,
        MouseScrollDelta::PixelDelta(position) => position.y as f32 * 0.1,
    }
}

#[allow(deprecated)]
fn set_window_ime_allowed(window: &dyn Window, allowed: bool) {
    window.set_ime_allowed(allowed);
}

pub(crate) struct HostWindowSnapshot {
    width: u32,
    height: u32,
    bytes: Vec<u8>,
}

impl HostWindowSnapshot {
    fn from_rgba_frame(frame: HostRgbaFrame) -> Self {
        Self {
            width: frame.width(),
            height: frame.height(),
            bytes: frame.into_bytes(),
        }
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn width(&self) -> u32 {
        self.width
    }

    pub(crate) fn height(&self) -> u32 {
        self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_window_refresh_diagnostics_update_state_overlay_text() {
        let host = UiHostWindow::new().expect("host window should construct for state test");
        host.set_host_presentation(HostWindowPresentationData::default());

        let mut diagnostics = HostRefreshDiagnostics::default();
        diagnostics.record_present(96, false, true);
        host.set_host_refresh_diagnostics_overlay(diagnostics.with_invalidation_diagnostics(
            HostInvalidationDiagnostics {
                slow_path_rebuild_count: 2,
                render_rebuild_count: 3,
                paint_only_request_count: 4,
            },
        ));

        let presentation = host.get_host_presentation();
        let overlay = presentation.host_shell.debug_refresh_rate.as_str();
        assert!(overlay.contains("present 1"));
        assert!(overlay.contains("full 0"));
        assert!(overlay.contains("region 1"));
        assert!(overlay.contains("pixels 96"));
        assert!(overlay.contains("slow 2"));
        assert!(overlay.contains("render 3"));
        assert!(overlay.contains("paint-only 4"));
    }

    #[test]
    fn close_requested_callback_can_mutate_host_state_without_reentrant_borrow() {
        let host = UiHostWindow::new().expect("host window should construct for state test");
        let callback_host = host.clone_strong();
        host.window().on_close_requested(move || {
            callback_host.set_host_refresh_invalidation_diagnostics(HostInvalidationDiagnostics {
                slow_path_rebuild_count: 1,
                render_rebuild_count: 2,
                paint_only_request_count: 3,
            });
            CloseRequestResponse::HideWindow
        });

        assert_eq!(
            host.close_requested_response(),
            CloseRequestResponse::HideWindow
        );
        let diagnostics = host.refresh_invalidation_diagnostics();
        assert_eq!(diagnostics.slow_path_rebuild_count, 1);
        assert_eq!(diagnostics.render_rebuild_count, 2);
        assert_eq!(diagnostics.paint_only_request_count, 3);
    }

    #[test]
    fn frame_update_region_queues_external_redraw_with_frame_update() {
        let host = UiHostWindow::new().expect("host window should construct for redraw test");
        let frame = FrameRect {
            x: 12.0,
            y: 24.0,
            width: 128.0,
            height: 72.0,
        };

        host.request_frame_update_region(frame.clone());

        let redraw = host.take_external_redraw();
        assert!(redraw.request_redraw());
        assert!(redraw.requires_frame_update());
        assert_eq!(redraw.damage_region(), Some(&frame));
    }
}
