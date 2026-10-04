use std::process::ExitStatus;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EditorChildDisposition {
    Exited {
        code: Option<i32>,
        signal: Option<i32>,
    },
    StoppedByHub,
    StoppedBeforeReady,
    FailedBeforeReady,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EditorChildTerminalReceipt {
    pub(crate) attempt_id: u64,
    pub(crate) process_id: u32,
    pub(crate) disposition: EditorChildDisposition,
    pub(crate) cleanup_error: Option<String>,
}

impl EditorChildTerminalReceipt {
    pub(crate) fn exited(
        attempt_id: u64,
        process_id: u32,
        status: ExitStatus,
        cleanup_error: Option<String>,
    ) -> Self {
        #[cfg(unix)]
        let signal = {
            use std::os::unix::process::ExitStatusExt;
            status.signal()
        };
        #[cfg(not(unix))]
        let signal = None;

        Self {
            attempt_id,
            process_id,
            disposition: EditorChildDisposition::Exited {
                code: status.code(),
                signal,
            },
            cleanup_error,
        }
    }

    pub(crate) fn stopped_by_hub(
        attempt_id: u64,
        process_id: u32,
        cleanup_error: Option<String>,
    ) -> Self {
        Self {
            attempt_id,
            process_id,
            disposition: EditorChildDisposition::StoppedByHub,
            cleanup_error,
        }
    }

    pub(crate) fn stopped_before_ready(
        attempt_id: u64,
        process_id: u32,
        cleanup_error: Option<String>,
    ) -> Self {
        Self {
            attempt_id,
            process_id,
            disposition: EditorChildDisposition::StoppedBeforeReady,
            cleanup_error,
        }
    }

    pub(crate) fn failed_before_ready(
        attempt_id: u64,
        process_id: u32,
        cleanup_error: Option<String>,
    ) -> Self {
        Self {
            attempt_id,
            process_id,
            disposition: EditorChildDisposition::FailedBeforeReady,
            cleanup_error,
        }
    }
}
