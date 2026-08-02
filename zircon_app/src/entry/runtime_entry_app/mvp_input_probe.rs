use winit::event_loop::ActiveEventLoop;
use zircon_runtime::diagnostic_log::write_log;
use zircon_runtime_interface::{
    ProfileControlCommand, ProfileControlRequest, ProfileControlResponse,
    RuntimeInputDiagnosticsSnapshot, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
    ZR_RUNTIME_KEY_ACTION_PRESSED_V1, ZR_RUNTIME_KEY_ACTION_RELEASED_V1,
    ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1, ZrByteSlice, ZrRuntimeEventV1, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1,
};

use super::RuntimeEntryApp;

const MVP_INPUT_PROBE_ENV: &str = "ZIRCON_RUNTIME_MVP_INPUT_PROBE";
const MVP_INPUT_PROBE_W_KEY_CODE: u32 = b'W' as u32;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn submit_mvp_input_probe_if_requested(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        if self.mvp_input_probe_submitted || !mvp_input_probe_enabled() {
            return;
        }
        self.mvp_input_probe_submitted = true;
        if let Err(error) = mvp_input_probe_viewport_supported(self.viewport_size) {
            self.report_fatal_failure(
                "runtime_input_probe",
                "viewport/pointer/mouse/keyboard host ABI probe",
                error,
                "restore the staged window to a positive viewport before accepting the runtime input probe",
            );
            event_loop.exit();
            return;
        }
        let before = match self.mvp_input_probe_input_snapshot() {
            Ok(snapshot) => snapshot,
            Err(error) => {
                self.report_fatal_failure(
                    "runtime_input_probe",
                    "viewport/pointer/mouse/keyboard host ABI probe",
                    error,
                    "verify the runtime exposes viewport and input diagnostics before accepting the staged host probe",
                );
                event_loop.exit();
                return;
            }
        };

        for (index, event) in mvp_input_probe_events(self.viewport, self.viewport_size)
            .into_iter()
            .enumerate()
        {
            if let Err(error) = self.session.handle_event(event) {
                self.report_fatal_failure(
                    "runtime_input_probe",
                    format!("event_index={index}"),
                    format!("submit staging input probe event failed: {error}"),
                    "verify the runtime accepts viewport, pointer, mouse, and keyboard probe events before retrying zircon_runtime",
                );
                event_loop.exit();
                return;
            }
        }
        if let Err(error) = self.verify_mvp_input_probe_consumed(&before) {
            self.report_fatal_failure(
                "runtime_input_probe",
                "viewport/pointer/mouse/keyboard host ABI probe",
                error,
                "verify the runtime applies the viewport resize and consumes every host input probe event before retrying zircon_runtime",
            );
            event_loop.exit();
            return;
        }
        write_log(
            "runtime_input_probe",
            "runtime_mvp_input_probe_submitted viewport_resize=2 pointer_move=1 mouse_press=1 mouse_release=1 keyboard_press=1 keyboard_release=1",
        );
    }

    fn verify_mvp_input_probe_consumed(
        &self,
        before: &RuntimeInputDiagnosticsSnapshot,
    ) -> Result<(), String> {
        let after = self.mvp_input_probe_input_snapshot()?;
        mvp_input_probe_counts_advanced(before, &after)
    }

    fn mvp_input_probe_input_snapshot(&self) -> Result<RuntimeInputDiagnosticsSnapshot, String> {
        let request = ProfileControlRequest {
            command: ProfileControlCommand::RuntimeDiagnosticsSnapshot,
            config: None,
        };
        match self.session.profile_control(&request) {
            Ok(Some(response)) => mvp_input_probe_response_received(&response),
            Ok(None) => Err(
                "runtime input diagnostics unavailable: profile control is unsupported".to_owned(),
            ),
            Err(error) => Err(format!("runtime input diagnostics request failed: {error}")),
        }
    }
}

pub(super) fn mvp_input_probe_enabled() -> bool {
    mvp_input_probe_enabled_value(std::env::var(MVP_INPUT_PROBE_ENV).ok().as_deref())
}

fn mvp_input_probe_enabled_value(value: Option<&str>) -> bool {
    value.is_some_and(|value| {
        value == "1" || value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("yes")
    })
}

fn mvp_input_probe_events(
    viewport: ZrRuntimeViewportHandle,
    viewport_size: ZrRuntimeViewportSizeV1,
) -> [ZrRuntimeEventV1; 7] {
    let probe_viewport_size = mvp_input_probe_resize_size(viewport_size);
    let [pointer_x, pointer_y] = mvp_input_probe_pointer_position(probe_viewport_size);
    [
        ZrRuntimeEventV1::viewport_resized(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            probe_viewport_size,
        ),
        ZrRuntimeEventV1::pointer_moved(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            pointer_x,
            pointer_y,
        ),
        ZrRuntimeEventV1::mouse_button(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
            ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
            pointer_x,
            pointer_y,
        ),
        ZrRuntimeEventV1::mouse_button(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
            ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
            pointer_x,
            pointer_y,
        ),
        ZrRuntimeEventV1::keyboard(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            ZR_RUNTIME_KEY_ACTION_PRESSED_V1,
            MVP_INPUT_PROBE_W_KEY_CODE,
            0,
            ZrByteSlice::empty(),
        ),
        ZrRuntimeEventV1::keyboard(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            ZR_RUNTIME_KEY_ACTION_RELEASED_V1,
            MVP_INPUT_PROBE_W_KEY_CODE,
            0,
            ZrByteSlice::empty(),
        ),
        ZrRuntimeEventV1::viewport_resized(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport, viewport_size),
    ]
}

fn mvp_input_probe_resize_size(viewport_size: ZrRuntimeViewportSizeV1) -> ZrRuntimeViewportSizeV1 {
    ZrRuntimeViewportSizeV1::new(
        mvp_input_probe_resize_dimension(viewport_size.width),
        mvp_input_probe_resize_dimension(viewport_size.height),
    )
}

fn mvp_input_probe_resize_dimension(dimension: u32) -> u32 {
    match dimension {
        0 => 1,
        1 => 2,
        dimension => dimension / 2,
    }
}

fn mvp_input_probe_pointer_position(viewport_size: ZrRuntimeViewportSizeV1) -> [f32; 2] {
    [
        viewport_size.width as f32 * 0.5,
        viewport_size.height as f32 * 0.5,
    ]
}

fn mvp_input_probe_viewport_supported(
    viewport_size: ZrRuntimeViewportSizeV1,
) -> Result<(), String> {
    if viewport_size.width == 0 || viewport_size.height == 0 {
        return Err(format!(
            "runtime input probe requires a positive viewport, got {}x{}",
            viewport_size.width, viewport_size.height
        ));
    }
    Ok(())
}

fn mvp_input_probe_counts_advanced(
    before: &RuntimeInputDiagnosticsSnapshot,
    after: &RuntimeInputDiagnosticsSnapshot,
) -> Result<(), String> {
    let missing = [
        (
            "viewport_resize_count",
            before.viewport_resize_count,
            after.viewport_resize_count,
        ),
        (
            "pointer_move_count",
            before.pointer_move_count,
            after.pointer_move_count,
        ),
        (
            "mouse_button_press_count",
            before.mouse_button_press_count,
            after.mouse_button_press_count,
        ),
        (
            "mouse_button_release_count",
            before.mouse_button_release_count,
            after.mouse_button_release_count,
        ),
        (
            "keyboard_press_count",
            before.keyboard_press_count,
            after.keyboard_press_count,
        ),
        (
            "keyboard_release_count",
            before.keyboard_release_count,
            after.keyboard_release_count,
        ),
    ]
    .into_iter()
    .filter(|(_, before, after)| after <= before)
    .map(|(name, before, after)| format!("{name} before={before} after={after}"))
    .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "runtime input probe events did not advance runtime input diagnostics: {}",
            missing.join(", ")
        ))
    }
}

fn mvp_input_probe_response_received(
    response: &ProfileControlResponse,
) -> Result<RuntimeInputDiagnosticsSnapshot, String> {
    if response.status != "ok" {
        return Err(format!(
            "runtime input diagnostics request reported status={} message={}",
            response.status, response.message
        ));
    }
    let Some(snapshot) = response.runtime_diagnostics.as_ref() else {
        return Err(format!(
            "runtime input diagnostics unavailable status={} message={}",
            response.status, response.message
        ));
    };
    Ok(snapshot.input.clone())
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::{
        ProfileControlResponse, RuntimeDiagnosticsSnapshot, RuntimeInputDiagnosticsSnapshot,
        ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
        ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1, ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1,
        ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1, ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1,
        ZR_RUNTIME_KEY_ACTION_PRESSED_V1, ZR_RUNTIME_KEY_ACTION_RELEASED_V1,
        ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1,
    };

    use super::{
        MVP_INPUT_PROBE_W_KEY_CODE, mvp_input_probe_counts_advanced, mvp_input_probe_enabled_value,
        mvp_input_probe_events, mvp_input_probe_resize_size, mvp_input_probe_response_received,
        mvp_input_probe_viewport_supported,
    };

    #[test]
    fn input_probe_is_explicitly_opt_in() {
        assert!(!mvp_input_probe_enabled_value(None));
        assert!(!mvp_input_probe_enabled_value(Some("0")));
        assert!(mvp_input_probe_enabled_value(Some("1")));
        assert!(mvp_input_probe_enabled_value(Some("true")));
        assert!(mvp_input_probe_enabled_value(Some("YES")));
    }

    #[test]
    fn input_probe_covers_viewport_resize_and_restores_it_after_input_events() {
        let events = mvp_input_probe_events(
            ZrRuntimeViewportHandle::new(1),
            ZrRuntimeViewportSizeV1::new(1280, 720),
        );

        assert_eq!(events[0].kind, ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1);
        assert_eq!(events[0].size, ZrRuntimeViewportSizeV1::new(640, 360));
        assert_eq!(events[1].kind, ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1);
        assert_eq!([events[1].x, events[1].y], [320.0, 180.0]);
        assert_eq!(events[2].kind, ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1);
        assert_eq!(events[2].state, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1);
        assert_eq!([events[2].x, events[2].y], [320.0, 180.0]);
        assert_eq!(events[3].kind, ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1);
        assert_eq!(events[3].state, ZR_RUNTIME_BUTTON_STATE_RELEASED_V1);
        assert_eq!([events[3].x, events[3].y], [320.0, 180.0]);
        assert_eq!(events[4].kind, ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1);
        assert_eq!(events[4].button, ZR_RUNTIME_KEY_ACTION_PRESSED_V1);
        assert_eq!(events[4].key_code, MVP_INPUT_PROBE_W_KEY_CODE);
        assert_eq!(events[5].kind, ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1);
        assert_eq!(events[5].button, ZR_RUNTIME_KEY_ACTION_RELEASED_V1);
        assert_eq!(events[5].key_code, MVP_INPUT_PROBE_W_KEY_CODE);
        assert_eq!(events[6].kind, ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1);
        assert_eq!(events[6].size, ZrRuntimeViewportSizeV1::new(1280, 720));
    }

    #[test]
    fn input_probe_always_uses_a_distinct_positive_intermediate_viewport() {
        for viewport in [
            ZrRuntimeViewportSizeV1::new(1, 1),
            ZrRuntimeViewportSizeV1::new(1, 720),
            ZrRuntimeViewportSizeV1::new(1280, 1),
            ZrRuntimeViewportSizeV1::new(1280, 720),
        ] {
            let intermediate = mvp_input_probe_resize_size(viewport);

            assert_ne!(intermediate, viewport);
            assert!(intermediate.width > 0);
            assert!(intermediate.height > 0);
        }
    }

    #[test]
    fn input_probe_events_keep_pointer_and_buttons_inside_the_resized_viewport() {
        for viewport in [
            ZrRuntimeViewportSizeV1::new(1, 1),
            ZrRuntimeViewportSizeV1::new(1, 720),
            ZrRuntimeViewportSizeV1::new(1280, 1),
            ZrRuntimeViewportSizeV1::new(1280, 720),
        ] {
            let events = mvp_input_probe_events(ZrRuntimeViewportHandle::new(1), viewport);
            let resized = events[0].size;

            for event in [&events[1], &events[2], &events[3]] {
                assert!(event.x >= 0.0 && event.x <= resized.width as f32);
                assert!(event.y >= 0.0 && event.y <= resized.height as f32);
            }
        }
    }

    #[test]
    fn input_probe_rejects_zero_sized_viewports_before_submitting_events() {
        assert!(mvp_input_probe_viewport_supported(ZrRuntimeViewportSizeV1::new(1, 1)).is_ok());
        assert!(mvp_input_probe_viewport_supported(ZrRuntimeViewportSizeV1::new(0, 1)).is_err());
        assert!(mvp_input_probe_viewport_supported(ZrRuntimeViewportSizeV1::new(1, 0)).is_err());
    }

    #[test]
    fn input_probe_requires_the_requested_events_to_advance_each_input_counter() {
        let before = RuntimeInputDiagnosticsSnapshot {
            viewport_resize_count: 7,
            pointer_move_count: 7,
            mouse_button_press_count: 7,
            mouse_button_release_count: 7,
            keyboard_press_count: 7,
            keyboard_release_count: 7,
        };

        let error = mvp_input_probe_counts_advanced(&before, &before)
            .expect_err("pre-existing input activity must not satisfy the MVP probe");

        assert!(error.contains("pointer_move_count before=7 after=7"));
        assert!(error.contains("viewport_resize_count before=7 after=7"));
        assert!(error.contains("mouse_button_press_count before=7 after=7"));
        assert!(error.contains("mouse_button_release_count before=7 after=7"));
        assert!(error.contains("keyboard_press_count before=7 after=7"));
        assert!(error.contains("keyboard_release_count before=7 after=7"));

        let after = RuntimeInputDiagnosticsSnapshot {
            viewport_resize_count: 8,
            pointer_move_count: 8,
            mouse_button_press_count: 8,
            mouse_button_release_count: 8,
            keyboard_press_count: 8,
            keyboard_release_count: 8,
        };
        assert!(mvp_input_probe_counts_advanced(&before, &after).is_ok());
    }

    #[test]
    fn input_probe_rejects_non_ok_runtime_diagnostics_response() {
        let mut response = ProfileControlResponse::error("input manager unavailable");
        response.runtime_diagnostics = Some(RuntimeDiagnosticsSnapshot {
            input: RuntimeInputDiagnosticsSnapshot {
                viewport_resize_count: 1,
                pointer_move_count: 1,
                mouse_button_press_count: 1,
                mouse_button_release_count: 1,
                keyboard_press_count: 1,
                keyboard_release_count: 1,
            },
            ..RuntimeDiagnosticsSnapshot::default()
        });

        assert_eq!(
            mvp_input_probe_response_received(&response).unwrap_err(),
            "runtime input diagnostics request reported status=error message=input manager unavailable"
        );
    }
}
