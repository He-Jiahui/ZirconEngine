#[cfg(feature = "profiling")]
use std::env;

use super::UiHostWindowEventLoop;
#[cfg(feature = "profiling")]
use crate::ui::retained_host::host_contract::diagnostics::HostWindowDiagnosticSeverity;
#[cfg(feature = "profiling")]
use crate::ui::retained_host::host_contract::profiling_artifacts::{
    profile_capture_enabled, profile_export_dir,
};

#[cfg(feature = "profiling")]
const PROFILE_WARMUP_PRESENTS_ENV: &str = "ZIRCON_PROFILE_WITHIN_PROCESS_WARMUP_PRESENTS";
#[cfg(feature = "profiling")]
const PROFILE_MEASUREMENT_READY_FILE: &str = "ui_profile_measurement_ready.json";
#[cfg(any(feature = "profiling", test))]
const MAX_PROFILE_WARMUP_PRESENTS: u32 = 120;

#[cfg(any(feature = "profiling", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UiProfileWarmupState {
    Measuring,
    Warmup { remaining_presents: u32 },
    RestartPending,
    RestartFailed,
}

#[cfg(any(feature = "profiling", test))]
impl UiProfileWarmupState {
    const fn new(presents: u32) -> Self {
        let remaining_presents = if presents > MAX_PROFILE_WARMUP_PRESENTS {
            MAX_PROFILE_WARMUP_PRESENTS
        } else {
            presents
        };
        if remaining_presents == 0 {
            Self::Measuring
        } else {
            Self::Warmup { remaining_presents }
        }
    }

    #[cfg(feature = "profiling")]
    pub(super) fn from_env() -> Self {
        if !profile_capture_enabled() {
            return Self::new(0);
        }
        let presents = env::var(PROFILE_WARMUP_PRESENTS_ENV)
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(0);
        Self::new(presents)
    }

    const fn measurement_active(self) -> bool {
        matches!(self, Self::Measuring)
    }

    fn complete_present(&mut self) -> bool {
        match *self {
            Self::Warmup {
                remaining_presents: 1,
            } => {
                *self = Self::RestartPending;
                true
            }
            Self::Warmup { remaining_presents } => {
                *self = Self::Warmup {
                    remaining_presents: remaining_presents - 1,
                };
                false
            }
            Self::Measuring | Self::RestartPending | Self::RestartFailed => false,
        }
    }

    const fn restart_pending(self) -> bool {
        matches!(self, Self::RestartPending)
    }

    fn complete_restart(&mut self, restarted: bool) {
        debug_assert!(self.restart_pending());
        *self = if restarted {
            Self::Measuring
        } else {
            Self::RestartFailed
        };
    }
}

impl UiHostWindowEventLoop {
    pub(super) fn profile_measurement_active(&self) -> bool {
        #[cfg(feature = "profiling")]
        {
            self.profile_warmup.measurement_active()
        }
        #[cfg(not(feature = "profiling"))]
        {
            true
        }
    }

    pub(super) fn complete_profile_warmup_present(&mut self) {
        #[cfg(feature = "profiling")]
        self.profile_warmup.complete_present();
    }

    pub(super) fn restart_profile_measurement_if_ready(&mut self) {
        #[cfg(feature = "profiling")]
        if self.profile_warmup.restart_pending() {
            zircon_runtime::core::diagnostics::profiling::reset_capture();
            self.reset_input_outcome_tracking();
            let restarted =
                zircon_runtime::core::diagnostics::profiling::start_capture_from_env("editor")
                    .is_some_and(|status| status.active);
            self.profile_warmup.complete_restart(restarted);
            if restarted {
                if let Err(error) = publish_profile_measurement_ready() {
                    self.host.record_host_diagnostic(
                        HostWindowDiagnosticSeverity::Warning,
                        format!("within-process UI profile readiness was not published: {error}"),
                    );
                }
            } else {
                self.host.record_host_diagnostic(
                    HostWindowDiagnosticSeverity::Warning,
                    "within-process UI profile measurement did not restart after warmup",
                );
            }
        }
    }
}

#[cfg(feature = "profiling")]
fn publish_profile_measurement_ready() -> Result<(), String> {
    let export_dir = profile_export_dir()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "profile output directory is not configured".to_owned())?;
    std::fs::create_dir_all(&export_dir).map_err(|error| error.to_string())?;
    let ready_path = export_dir.join(PROFILE_MEASUREMENT_READY_FILE);
    let temporary_path = export_dir.join(format!(
        "{PROFILE_MEASUREMENT_READY_FILE}.{}.tmp",
        std::process::id()
    ));
    let payload = format!(
        "{{\"schema_version\":1,\"measurement_ready\":true,\"process_id\":{}}}\n",
        std::process::id()
    );
    std::fs::write(&temporary_path, payload).map_err(|error| error.to_string())?;
    std::fs::rename(&temporary_path, &ready_path).map_err(|error| error.to_string())
}

#[cfg(test)]
#[path = "tests/profile_capture.rs"]
mod tests;
