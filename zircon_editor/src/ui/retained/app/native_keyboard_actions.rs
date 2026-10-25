use super::hierarchy_pointer::HierarchyTerminalReason;
use super::*;
use crate::core::editor_event::EditorEventSource;
use crate::ui::retained_host::event_bridge::apply_record_effects;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;
use zircon_runtime_interface::ui::dispatch::{UiKeyboardInputEvent, UiKeyboardInputState};

const STOP_PLAY_COMMAND_ID: &str = "runtime.play_mode.exit";

impl RetainedEditorHost {
    pub(super) fn dispatch_unhandled_native_keyboard_input(
        &mut self,
        keyboard: UiKeyboardInputEvent,
    ) {
        let shift_f5 = play_stop_trace_enabled() && is_shift_f5(&keyboard);
        if is_escape_pressed(&keyboard) && self.workbench_window_bridge.content_drawer_is_open() {
            match self.workbench_window_bridge.dismiss_content_drawer() {
                Ok(()) => {
                    let mut effects = UiHostEventEffects::default();
                    effects.request_paint_only();
                    self.apply_dispatch_effects(effects);
                }
                Err(error) => self.set_status_line(error.to_string()),
            }
            return;
        }
        let stop_shortcut_match = keyboard.state == UiKeyboardInputState::Pressed
            && match self.focused_game_stop_shortcut_matches(&keyboard) {
                Ok(matches) => matches,
                Err(error) => {
                    self.set_status_line(error);
                    return;
                }
            };
        if shift_f5 {
            eprintln!(
                "mvp_play_trace component=editor_native_input stage=shift_f5_unhandled input_seq={:?} stop_shortcut_match={stop_shortcut_match}",
                keyboard.metadata.sequence
            );
        }
        if stop_shortcut_match {
            self.dispatch_native_editor_keymap_command(&keyboard);
            return;
        }
        if self.route_focused_game_keyboard_input(&keyboard) {
            return;
        }
        if keyboard.state != UiKeyboardInputState::Pressed {
            return;
        }
        if self.dispatch_module_plugin_details_keyboard(&keyboard) {
            return;
        }
        if self.try_begin_hierarchy_rename_from_keyboard(&keyboard) {
            return;
        }
        if is_escape_pressed(&keyboard) {
            self.retire_hierarchy_drag_with_reason(HierarchyTerminalReason::Escape);
            self.cancel_viewport_interaction();
            self.dispatch_native_editor_keymap_command(&keyboard);
            return;
        }
        self.dispatch_native_editor_keymap_command(&keyboard);
    }

    fn focused_game_stop_shortcut_matches(
        &self,
        keyboard: &UiKeyboardInputEvent,
    ) -> Result<bool, String> {
        if !self.runtime.play_preview_input_active()
            || !self
                .runtime
                .play_preview_view_focused()
                .map_err(|error| error.to_string())?
        {
            return Ok(false);
        }
        Ok(self
            .editor_manager
            .enter()
            .map_err(|error| error.to_string())?
            .resolve_keyboard_input(keyboard)
            .as_deref()
            == Some(STOP_PLAY_COMMAND_ID))
    }

    fn dispatch_native_editor_keymap_command(&mut self, keyboard: &UiKeyboardInputEvent) {
        let trace_sequence = if play_stop_trace_enabled() && is_shift_f5(keyboard) {
            static SEQUENCE: AtomicU64 = AtomicU64::new(0);
            Some(SEQUENCE.fetch_add(1, Ordering::Relaxed) + 1)
        } else {
            None
        };
        let started = trace_sequence.map(|sequence| {
            eprintln!(
                "mvp_play_trace component=editor_native_input seq={sequence} stage=shift_f5_keymap_enter"
            );
            Instant::now()
        });
        let dispatch = self
            .runtime
            .dispatch_keyboard_keymap_command(keyboard, EditorEventSource::RetainedHost);
        if let (Some(sequence), Some(started)) = (trace_sequence, started) {
            eprintln!(
                "mvp_play_trace component=editor_native_input seq={sequence} stage=shift_f5_keymap_return result={} elapsed_us={}",
                if dispatch.is_ok() { "ok" } else { "error" },
                started.elapsed().as_micros()
            );
        }
        match dispatch {
            Ok(Some(record)) => {
                let mut effects = UiHostEventEffects::default();
                apply_record_effects(&mut effects, &record);
                self.apply_dispatch_effects(effects);
            }
            Ok(None) => {}
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn cancel_viewport_interaction(&mut self) {
        match self
            .viewport_pointer_bridge
            .cancel_interaction(&self.runtime)
        {
            Ok(effects) => self.apply_dispatch_effects(effects),
            Err(error) => self.set_status_line(error),
        }
    }
}

fn is_escape_pressed(keyboard: &UiKeyboardInputEvent) -> bool {
    keyboard.state == UiKeyboardInputState::Pressed
        && (keyboard.logical_key.eq_ignore_ascii_case("escape")
            || keyboard.logical_key.eq_ignore_ascii_case("esc")
            || keyboard.key_code == 27)
}

fn is_shift_f5(keyboard: &UiKeyboardInputEvent) -> bool {
    keyboard.state == UiKeyboardInputState::Pressed
        && keyboard.metadata.modifiers.shift
        && (keyboard.logical_key.eq_ignore_ascii_case("f5")
            || keyboard.physical_key.eq_ignore_ascii_case("f5")
            || keyboard.key_code == 116)
}

fn play_stop_trace_enabled() -> bool {
    static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
    *TRACE_ENABLED.get_or_init(|| {
        std::env::var("ZIRCON_TRACE_PLAY_STOP")
            .is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
    })
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn sync_content_drawer_native_focus(&mut self) {
        self.sync_content_drawer_native_focus_state();
        // Every existing full/sparse/geometry commit reaches this prepare hook,
        // including the unchanged-drawer early returns inside the focus helper.
        self.publish_current_normal_source_capture_metadata();
    }

    fn sync_content_drawer_native_focus_state(&mut self) {
        use crate::ui::retained_host::host_contract::data::{FrameRect, HostTextInputFocusData};
        use crate::ui::retained_host::host_contract::globals::UiHostContext;
        let open = self.workbench_window_bridge.content_drawer_is_open();
        if self
            .workbench_window_bridge
            .content_drawer_native_focus_open
            == open
        {
            return;
        }
        let context = self.ui.global::<UiHostContext>();
        if open {
            let generation = self.ui.get_host_presentation_generation();
            let Some(node) = generation
                .structure()
                .workbench_window_nodes
                .iter()
                .find(|node| node.control_id.as_str() == "SearchEdited")
            else {
                return;
            };
            if node.edit_action_id.is_empty() {
                return;
            }
            self.workbench_window_bridge
                .content_drawer_native_previous_focus = Some(context.get_text_input_focus());
            let frame = self
                .workbench_window_bridge
                .control_frame("SearchEdited")
                .map(|frame| {
                    let mount = self.workbench_window_bridge.layout_frames().mount_frame;
                    FrameRect {
                        x: frame.x + mount.map_or(0.0, |mount| mount.x),
                        y: frame.y + mount.map_or(0.0, |mount| mount.y),
                        width: frame.width,
                        height: frame.height,
                    }
                })
                .unwrap_or_else(|| FrameRect {
                    x: node.frame.x,
                    y: node.frame.y,
                    width: node.frame.width,
                    height: node.frame.height,
                });
            context.set_text_input_focus(HostTextInputFocusData {
                control_id: node.control_id.clone(),
                dispatch_kind: node.dispatch_kind.clone(),
                action_id: node.action_id.clone(),
                edit_action_id: node.edit_action_id.clone(),
                commit_action_id: node.commit_action_id.clone(),
                value_text: node.value_text.clone(),
                edit_frame: frame,
                ..HostTextInputFocusData::default()
            });
        } else {
            let current = context.get_text_input_focus();
            let drawer_had_focus = current.control_id.as_str() == "SearchEdited";
            if drawer_had_focus {
                context.clear_text_input_focus();
            }
            let previous = self
                .workbench_window_bridge
                .content_drawer_native_previous_focus
                .take();
            if drawer_had_focus && !context.get_text_input_focus().is_active() {
                if let Some(previous) = previous {
                    context.set_text_input_focus(previous);
                }
            }
        }
        self.workbench_window_bridge
            .content_drawer_native_focus_open = open;
    }
}
