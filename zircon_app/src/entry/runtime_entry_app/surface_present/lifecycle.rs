//! 关闭主窗口或 App 时先尝试解绑动态 Runtime 表面，再清理宿主窗口状态与诊断 presenter。
//! 解绑失败保留窗口到会话销毁成功；销毁失败沿用进程致命边界，不能释放仍被 Runtime 借用的句柄。

use zircon_runtime::diagnostic_log::write_log;

use super::super::RuntimeEntryApp;
use crate::entry::runtime_library::RuntimeLibraryError;

impl RuntimeEntryApp {
    /// 窗口保持存活直到解绑或会话销毁成功；回退销毁不清除最初的产品失败。
    pub(in crate::entry::runtime_entry_app) fn teardown_primary_window(&mut self) -> bool {
        // IME cancellation can call the session, so it must precede a terminal destroy fallback.
        self.retire_native_ime_composition();
        self.clear_native_ime_candidate();
        let surface_released = self.teardown_surface_present();
        if !surface_released {
            // Keep the App's native Window owner and presenter alive
            // while the session retries release once and destroys its remaining surface resources.
            self.session.destroy_for_host_resource_release();
            self.surface_present_enabled = false;
            self.surface_present_attempted = false;
        }
        if let Some(presenter) = self.presenter.take() {
            presenter.publish_summary();
        }
        self.ime_window_generation = self.ime_window_generation.checked_add(1).unwrap_or(0);
        self.window = None;
        surface_released
    }

    pub(super) fn disable_surface_present(&mut self) -> bool {
        self.teardown_surface_present()
    }

    fn teardown_surface_present(&mut self) -> bool {
        match self.release_surface_present() {
            Ok(()) => true,
            Err(error) => {
                self.report_fatal_failure(
                    "runtime_surface_present",
                    format!("viewport={:?}", self.viewport),
                    format!("runtime surface unbind failed: {error}"),
                    "verify the runtime surface lifecycle and restart zircon_runtime",
                );
                false
            }
        }
    }

    /// 会话账本决定是否存在绑定；只有释放成功后才清除 App 侧呈现状态。
    fn release_surface_present(&mut self) -> Result<(), RuntimeLibraryError> {
        match self.session.unbind_viewport_surface(self.viewport)? {
            true => write_log(
                "runtime_surface_present",
                "runtime_product_teardown surface_unbind=ok",
            ),
            false => write_log(
                "runtime_surface_present",
                "runtime_product_teardown surface_unbind=unavailable",
            ),
        }
        self.surface_present_enabled = false;
        self.surface_present_attempted = false;
        Ok(())
    }

    pub(in crate::entry::runtime_entry_app) fn enable_surface_present(&mut self) {
        self.surface_present_enabled = true;
        write_log("runtime_surface_present", "runtime_surface_present_enabled");
    }

    pub(in crate::entry::runtime_entry_app) fn enable_reference_cpu_presenter(&mut self) -> bool {
        if self.failure_state.is_recorded() {
            return false;
        }
        if !self.disable_surface_present() {
            return false;
        }
        write_log(
            "runtime_surface_present",
            "runtime_reference_cpu_presenter_enabled capability=degraded",
        );
        true
    }
}

// 退出时记录帧调度统计、停止手柄反馈，再按先尝试解绑后丢弃窗口状态的顺序收尾。
impl Drop for RuntimeEntryApp {
    fn drop(&mut self) {
        let cadence = self.frame_cadence.report();
        write_log(
            "runtime_frame_cadence",
            format!(
                "runtime_frame_cadence_summary policy={} request_attempts={} requests_accepted={} requests_coalesced={} requests_ignored={} pumps={} idle_suppressed={} redraw_requests={} focus_transitions={} occlusion_transitions={} low_power_pumps={} low_power_suppressed={}",
                self.frame_cadence.policy().as_str(),
                cadence.frame_requests,
                cadence.frame_requests_accepted,
                cadence.frame_requests_coalesced,
                cadence.frame_requests_ignored,
                cadence.frame_pumps,
                cadence.idle_pumps_suppressed,
                cadence.redraw_requests,
                cadence.focus_transitions,
                cadence.occlusion_transitions,
                cadence.low_power_pumps,
                cadence.low_power_pumps_suppressed,
            ),
        );
        #[cfg(feature = "gamepad-gilrs")]
        super::super::gamepad::clear_gamepad_rumble_effects(&mut self.gamepad_rumble_effects);
        let _ = self.teardown_primary_window();
    }
}

#[cfg(all(
    test,
    target_os = "windows",
    feature = "dynamic-api",
    not(feature = "gamepad-gilrs")
))]
#[path = "lifecycle/tests/cases.rs"]
mod tests;
