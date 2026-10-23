use std::sync::{Arc, Weak};
use std::thread::JoinHandle;

use crate::core::editor_event::EditorEventRecord;

use super::EditorHostEventController;

pub(super) struct AsyncPlayTask {
    pub(super) label: &'static str,
    pub(super) started: std::time::Instant,
    pub(super) worker: JoinHandle<Result<EditorEventRecord, String>>,
}

#[derive(Default)]
pub(super) struct AsyncPlayCommands {
    pub(super) owner: Weak<EditorHostEventController>,
    pub(super) wake: Option<Arc<dyn Fn() + Send + Sync>>,
    pub(super) task: Option<AsyncPlayTask>,
    preview: Option<
        JoinHandle<
            Result<
                Option<crate::core::play::PlayPreviewFrame>,
                crate::core::play::PlayPreviewCaptureError,
            >,
        >,
    >,
}

impl EditorHostEventController {
    pub(crate) fn native_preview_in_flight(&self) -> bool {
        self.async_play
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .preview
            .as_ref()
            .is_some_and(|task| !task.is_finished())
    }

    pub(crate) fn async_play_enabled(&self) -> bool {
        self.async_play
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .owner
            .strong_count()
            > 0
    }

    /// At most one capture is in flight. Pending frames keep the last image; completed
    /// products are checked against the active session before the UI publishes them.
    pub(crate) fn poll_native_preview_frame(
        &self,
        size: zircon_runtime_interface::ZrRuntimeViewportSizeV1,
    ) -> Option<
        Result<
            Option<crate::core::play::PlayPreviewFrame>,
            crate::core::play::PlayPreviewCaptureError,
        >,
    > {
        let mut commands = self
            .async_play
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if commands
            .preview
            .as_ref()
            .is_some_and(|task| !task.is_finished())
        {
            return None;
        }
        let completed = commands.preview.take().map(|task| {
            task.join().unwrap_or_else(|_| {
                Err(crate::core::play::PlayPreviewCaptureError::Capture(
                    crate::core::gateway::GatewayError::Runtime {
                        message: "Play preview worker panicked".into(),
                    },
                ))
            })
        });
        if let Some(controller) = commands.owner.upgrade() {
            let wake = commands.wake.clone();
            match std::thread::Builder::new()
                .name("editor-play-preview".into())
                .spawn(move || {
                    static ENTERED: std::sync::Once = std::sync::Once::new();
                    static RETURNED: std::sync::Once = std::sync::Once::new();
                    ENTERED.call_once(|| {
                        eprintln!("mvp_play_boundary editor_first_preview_capture_entered")
                    });
                    let result = controller.play_sessions().capture_preview_frame(size);
                    RETURNED.call_once(|| {
                        eprintln!("mvp_play_boundary editor_first_preview_capture_returned")
                    });
                    if let Some(wake) = wake {
                        wake();
                    }
                    result
                }) {
                Ok(task) => commands.preview = Some(task),
                Err(error) => {
                    return Some(Err(crate::core::play::PlayPreviewCaptureError::Capture(
                        crate::core::gateway::GatewayError::Runtime {
                            message: error.to_string(),
                        },
                    )))
                }
            }
        }
        completed
    }

    pub(crate) fn pending_play_elapsed(&self) -> std::time::Duration {
        self.async_play
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .task
            .as_ref()
            .map(|task| task.started.elapsed())
            .unwrap_or_default()
    }
    /// Native windows opt in; commandlets retain their synchronous receipt contract.
    pub(crate) fn enable_async_play(self: &Arc<Self>, wake: Arc<dyn Fn() + Send + Sync>) {
        let mut commands = self
            .async_play
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        commands.owner = Arc::downgrade(self);
        commands.wake = Some(wake);
    }

    pub(crate) fn pending_play_label(&self) -> Option<&'static str> {
        self.async_play
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .task
            .as_ref()
            .map(|task| task.label)
    }

    pub(crate) fn take_completed_play_command(&self) -> Option<Result<EditorEventRecord, String>> {
        let task = {
            let mut commands = self
                .async_play
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            if !commands
                .task
                .as_ref()
                .is_some_and(|task| task.worker.is_finished())
            {
                return None;
            }
            commands.task.take()?
        };
        Some(task.worker.join().unwrap_or_else(|_| {
            Err("The Play transition worker panicked; see the editor log.".to_string())
        }))
    }
}
