//! MVP 产品验收显式启用的宿主输入 ABI 探针。
//! 通过正常事件边界验证本批输入消费；完整投递成功后以配对释放和恢复 viewport 收尾。
//! 中途事件失败立即请求产品退出，剩余释放与恢复事件不会继续投递；历史计数不能替代本批增长。

use winit::event_loop::ActiveEventLoop;
use zircon_runtime::diagnostic_log::write_log;
use zircon_runtime_interface::{
    ProfileControlCommand, ProfileControlRequest, ProfileControlResponse,
    RuntimeInputDiagnosticsSnapshot, ZrByteSlice, ZrRuntimeEventV1, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1, ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
    ZR_RUNTIME_BUTTON_STATE_RELEASED_V1, ZR_RUNTIME_KEY_ACTION_PRESSED_V1,
    ZR_RUNTIME_KEY_ACTION_RELEASED_V1, ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
};

use super::RuntimeEntryApp;

const MVP_INPUT_PROBE_ENV: &str = "ZIRCON_RUNTIME_MVP_INPUT_PROBE";
const MVP_INPUT_PROBE_SINGLE_EVENT_COUNT: u64 = 1;
const MVP_INPUT_PROBE_VIEWPORT_RESIZE_EVENT_COUNT: u64 = 2;
const MVP_INPUT_PROBE_W_KEY_CODE: u32 = b'W' as u32;

impl RuntimeEntryApp {
    /// 首次表面可用时执行显式启用的 ABI 探针，失败沿用产品事件错误与退出策略。
    pub(in crate::entry::runtime_entry_app) fn submit_mvp_input_probe_if_requested(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) -> bool {
        if self.mvp_input_probe_submitted || !mvp_input_probe_enabled() {
            return true;
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
            return false;
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
                return false;
            }
        };

        for event in mvp_input_probe_events(self.viewport, self.viewport_size) {
            if !self.dispatch_runtime_event(event_loop, event) {
                return false;
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
            return false;
        }
        write_log(
            "runtime_input_probe",
            "runtime_mvp_input_probe_submitted viewport_resize=2 pointer_move=1 mouse_press=1 mouse_release=1 keyboard_press=1 keyboard_release=1",
        );
        true
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

// 该数组在完整投递成功时恢复 viewport 并配对释放按键/按钮；投递方遇错即停，不能把尾事件当作无条件清理。
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

/// 对本次前后快照要求逐类增量，避免已有用户输入使未消费的探针看起来成功。
fn mvp_input_probe_counts_advanced(
    before: &RuntimeInputDiagnosticsSnapshot,
    after: &RuntimeInputDiagnosticsSnapshot,
) -> Result<(), String> {
    let missing = [
        (
            "viewport_resize_count",
            before.viewport_resize_count,
            after.viewport_resize_count,
            MVP_INPUT_PROBE_VIEWPORT_RESIZE_EVENT_COUNT,
        ),
        (
            "pointer_move_count",
            before.pointer_move_count,
            after.pointer_move_count,
            MVP_INPUT_PROBE_SINGLE_EVENT_COUNT,
        ),
        (
            "mouse_button_press_count",
            before.mouse_button_press_count,
            after.mouse_button_press_count,
            MVP_INPUT_PROBE_SINGLE_EVENT_COUNT,
        ),
        (
            "mouse_button_release_count",
            before.mouse_button_release_count,
            after.mouse_button_release_count,
            MVP_INPUT_PROBE_SINGLE_EVENT_COUNT,
        ),
        (
            "keyboard_press_count",
            before.keyboard_press_count,
            after.keyboard_press_count,
            MVP_INPUT_PROBE_SINGLE_EVENT_COUNT,
        ),
        (
            "keyboard_release_count",
            before.keyboard_release_count,
            after.keyboard_release_count,
            MVP_INPUT_PROBE_SINGLE_EVENT_COUNT,
        ),
    ]
    .into_iter()
    .filter(|(_, before, after, required_delta)| after.saturating_sub(*before) < *required_delta)
    .map(|(name, before, after, required_delta)| {
        format!("{name} before={before} after={after} required_delta={required_delta}")
    })
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
#[path = "tests/mvp_input_probe.rs"]
mod tests;
