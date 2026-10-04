fn main() -> std::process::ExitCode {
    use zircon_runtime::diagnostic_log::{
        install_process_log_panic_flush, shutdown_process_log,
        DEFAULT_DIAGNOSTIC_LOG_CRASH_FLUSH_TIMEOUT,
    };
    install_process_log_panic_flush(DEFAULT_DIAGNOSTIC_LOG_CRASH_FLUSH_TIMEOUT);
    let controller = zircon_app::HeadlessController::default();
    let signal_guard = match signals::install(controller.clone()) {
        Ok(guard) => guard,
        Err(error) => {
            eprintln!("headless signal registration failed: {error}");
            return std::process::ExitCode::FAILURE;
        }
    };
    let result = zircon_app::EntryRunner::run_headless_with_args(
        std::env::args().skip(1),
        controller.clone(),
    );
    let successful = match result {
        Ok(_) => true,
        Err(error) => {
            eprintln!("{error}");
            false
        }
    };
    let shutdown_deadline = controller.begin_shutdown();
    let runtime_stopped = controller.finish_runtime_until(shutdown_deadline);
    let log_stopped = shutdown_process_log(
        shutdown_deadline.saturating_duration_since(std::time::Instant::now()),
    );
    if !runtime_stopped {
        eprintln!("headless shutdown deadline expired with active DLL owner");
        std::process::abort();
    }
    controller.mark_shutdown_complete();
    if controller.shutdown_deadline_expired() {
        eprintln!("zircon_server phase=terminal shutdown=incomplete deadline=expired");
    }
    drop(signal_guard);
    if log_stopped && successful && !controller.shutdown_deadline_expired() {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}

#[cfg(windows)]
mod signals {
    use std::sync::OnceLock;
    use windows_sys::Win32::System::Console::{
        SetConsoleCtrlHandler, CTRL_BREAK_EVENT, CTRL_CLOSE_EVENT, CTRL_C_EVENT, CTRL_LOGOFF_EVENT,
        CTRL_SHUTDOWN_EVENT,
    };
    static CONTROLLER: OnceLock<zircon_app::HeadlessController> = OnceLock::new();
    pub(super) struct SignalGuard;
    pub(super) fn install(
        controller: zircon_app::HeadlessController,
    ) -> std::io::Result<SignalGuard> {
        CONTROLLER
            .set(controller)
            .map_err(|_| std::io::Error::other("headless signal owner already installed"))?;
        if unsafe { SetConsoleCtrlHandler(Some(handle_signal), 1) } == 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(SignalGuard)
    }
    unsafe extern "system" fn handle_signal(kind: u32) -> i32 {
        if matches!(
            kind,
            CTRL_C_EVENT
                | CTRL_BREAK_EVENT
                | CTRL_CLOSE_EVENT
                | CTRL_LOGOFF_EVENT
                | CTRL_SHUTDOWN_EVENT
        ) {
            if let Some(controller) = CONTROLLER.get() {
                controller.cancel();
                if matches!(
                    kind,
                    CTRL_CLOSE_EVENT | CTRL_LOGOFF_EVENT | CTRL_SHUTDOWN_EVENT
                ) {
                    // Windows may terminate the process as soon as this handler returns.
                    // The main owner destroys and flushes within the cancellation deadline.
                    controller.wait_shutdown_until(controller.begin_shutdown());
                }
            }
            1
        } else {
            0
        }
    }
    impl Drop for SignalGuard {
        fn drop(&mut self) {
            unsafe {
                SetConsoleCtrlHandler(Some(handle_signal), 0);
            }
        }
    }
}

#[cfg(not(windows))]
mod signals {
    pub(super) struct SignalGuard;
    pub(super) fn install(_: zircon_app::HeadlessController) -> std::io::Result<SignalGuard> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "native headless signal support is not implemented for this platform",
        ))
    }
}
