use std::fmt;
use std::sync::Arc;
use std::thread::JoinHandle;

use crossbeam_channel::{bounded, Receiver, Sender};
use zircon_runtime_interface::ui::surface::UiRenderExtract;

use crate::core::framework::render::{
    RenderFrameExtract, RenderFrameworkError, RenderSubmissionConfig, RenderViewportHandle,
};
use crate::graphics::types::ViewportRenderFrame;

use super::super::wgpu_render_framework::WgpuRenderFrameworkCore;

pub(in crate::graphics::runtime::render_framework) type FrameSubmissionExecutor =
    fn(
        &WgpuRenderFrameworkCore,
        RenderViewportHandle,
        RenderFrameExtract,
        Option<UiRenderExtract>,
    ) -> Result<(), RenderFrameworkError>;

pub(in crate::graphics::runtime::render_framework) type RuntimeFrameSubmissionExecutor =
    fn(
        &WgpuRenderFrameworkCore,
        RenderViewportHandle,
        ViewportRenderFrame,
    ) -> Result<(), RenderFrameworkError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PipelinedSubmissionQueueError {
    WorkerUnavailable,
}

impl fmt::Display for PipelinedSubmissionQueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorkerUnavailable => f.write_str("pipelined render worker is unavailable"),
        }
    }
}

struct PipelinedSubmissionQueue<T: Send + 'static, R: Send + 'static> {
    // Reused one-slot channels preserve bounded feedback without per-frame channel allocation.
    sender: Option<Sender<T>>,
    started: Receiver<()>,
    completed: Receiver<R>,
    pending: bool,
    worker: Option<JoinHandle<()>>,
}

impl<T: Send + 'static, R: Send + 'static> PipelinedSubmissionQueue<T, R> {
    fn new(
        execute: impl Fn(T, &Sender<()>) -> R + Send + Sync + 'static,
    ) -> Result<Self, PipelinedSubmissionQueueError> {
        let (sender, receiver) = bounded(1);
        let (started_sender, started) = bounded(1);
        let (completed_sender, completed) = bounded(1);
        let execute = Arc::new(execute);
        let worker = std::thread::Builder::new()
            .name("zircon-render-submit".to_string())
            .spawn(move || worker_loop(receiver, execute, started_sender, completed_sender))
            .map_err(|_| PipelinedSubmissionQueueError::WorkerUnavailable)?;
        Ok(Self {
            sender: Some(sender),
            started,
            completed,
            pending: false,
            worker: Some(worker),
        })
    }

    // A producer observes N-1 before it queues N, so the queue stays bounded
    // while simulation can overlap the render worker between submissions.
    fn submit(&mut self, payload: T) -> Result<Option<R>, PipelinedSubmissionQueueError> {
        let previous = {
            crate::profile_scope!(
                "runtime",
                "render_framework.scheduler",
                "wait_previous_submission"
            );
            self.take_completed()?
        };
        // This transition is emitted only after the prior submission has
        // completed, so a 0/1 trace describes scheduler-worker occupancy.
        crate::profile_counter!(
            "runtime",
            "render_framework.scheduler.worker_utilization",
            0
        );
        self.sender
            .as_ref()
            .ok_or(PipelinedSubmissionQueueError::WorkerUnavailable)?
            .send(payload)
            .map_err(|_| PipelinedSubmissionQueueError::WorkerUnavailable)?;
        {
            crate::profile_scope!("runtime", "render_framework.scheduler", "wait_worker_start");
            self.started
                .recv()
                .map_err(|_| PipelinedSubmissionQueueError::WorkerUnavailable)?;
        }
        self.pending = true;
        self.record_pending_depth();
        Ok(previous)
    }

    fn finish(&mut self) -> Result<Option<R>, PipelinedSubmissionQueueError> {
        crate::profile_scope!(
            "runtime",
            "render_framework.scheduler",
            "wait_pending_submission"
        );
        self.take_completed()
    }

    fn take_completed(&mut self) -> Result<Option<R>, PipelinedSubmissionQueueError> {
        if !self.pending {
            return Ok(None);
        }
        self.completed
            .recv()
            .map(|result| {
                self.pending = false;
                self.record_pending_depth();
                Some(result)
            })
            .map_err(|_| PipelinedSubmissionQueueError::WorkerUnavailable)
    }

    fn record_pending_depth(&self) {
        crate::profile_counter!(
            "runtime",
            "render_framework.scheduler.pending_depth",
            usize::from(self.pending)
        );
    }
}

impl<T: Send + 'static, R: Send + 'static> Drop for PipelinedSubmissionQueue<T, R> {
    fn drop(&mut self) {
        drop(self.sender.take());
        let Some(worker) = self.worker.take() else {
            return;
        };
        if worker.thread().id() != std::thread::current().id() {
            let _ = worker.join();
        }
    }
}

fn worker_loop<T: Send + 'static, R: Send + 'static>(
    receiver: Receiver<T>,
    execute: Arc<impl Fn(T, &Sender<()>) -> R + Send + Sync + 'static>,
    started: Sender<()>,
    completed: Sender<R>,
) {
    while let Ok(payload) = receiver.recv() {
        let result = execute(payload, &started);
        let _ = completed.send(result);
    }
}

struct FrameSubmission {
    kind: FrameSubmissionKind,
}

enum FrameSubmissionKind {
    Extract {
        execute: FrameSubmissionExecutor,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
        ui: Option<UiRenderExtract>,
    },
    RuntimeFrame {
        execute: RuntimeFrameSubmissionExecutor,
        viewport: RenderViewportHandle,
        frame: ViewportRenderFrame,
    },
}

impl FrameSubmission {
    fn extract(
        execute: FrameSubmissionExecutor,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
        ui: Option<UiRenderExtract>,
    ) -> Self {
        Self {
            kind: FrameSubmissionKind::Extract {
                execute,
                viewport,
                extract,
                ui,
            },
        }
    }

    fn runtime_frame(
        execute: RuntimeFrameSubmissionExecutor,
        viewport: RenderViewportHandle,
        frame: ViewportRenderFrame,
    ) -> Self {
        Self {
            kind: FrameSubmissionKind::RuntimeFrame {
                execute,
                viewport,
                frame,
            },
        }
    }

    fn execute(self, core: &WgpuRenderFrameworkCore) -> Result<(), RenderFrameworkError> {
        match self.kind {
            FrameSubmissionKind::Extract {
                execute,
                viewport,
                extract,
                ui,
            } => execute(core, viewport, extract, ui),
            FrameSubmissionKind::RuntimeFrame {
                execute,
                viewport,
                frame,
            } => execute(core, viewport, frame),
        }
    }
}

pub(in crate::graphics::runtime::render_framework) struct RenderSubmissionScheduler {
    config: RenderSubmissionConfig,
    queue: Option<PipelinedSubmissionQueue<FrameSubmission, Result<(), RenderFrameworkError>>>,
}

impl Default for RenderSubmissionScheduler {
    fn default() -> Self {
        Self {
            config: RenderSubmissionConfig::default(),
            queue: None,
        }
    }
}

impl RenderSubmissionScheduler {
    pub(in crate::graphics::runtime::render_framework) fn config(&self) -> RenderSubmissionConfig {
        self.config
    }

    pub(in crate::graphics::runtime::render_framework) fn set_config(
        &mut self,
        core: Arc<WgpuRenderFrameworkCore>,
        config: RenderSubmissionConfig,
    ) -> Result<(), RenderFrameworkError> {
        self.finish_pending()?;
        if !config.pipelined_render {
            self.queue = None;
        } else {
            self.ensure_queue(core)?;
        }
        self.config = config;
        Ok(())
    }

    pub(in crate::graphics::runtime::render_framework) fn submit(
        &mut self,
        core: Arc<WgpuRenderFrameworkCore>,
        execute: FrameSubmissionExecutor,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
        ui: Option<UiRenderExtract>,
    ) -> Result<(), RenderFrameworkError> {
        self.submit_submission(
            core,
            FrameSubmission::extract(execute, viewport, extract, ui),
        )
    }

    pub(in crate::graphics::runtime::render_framework) fn submit_runtime_frame(
        &mut self,
        core: Arc<WgpuRenderFrameworkCore>,
        execute: RuntimeFrameSubmissionExecutor,
        viewport: RenderViewportHandle,
        frame: ViewportRenderFrame,
    ) -> Result<(), RenderFrameworkError> {
        self.submit_submission(
            core,
            FrameSubmission::runtime_frame(execute, viewport, frame),
        )
    }

    fn submit_submission(
        &mut self,
        core: Arc<WgpuRenderFrameworkCore>,
        submission: FrameSubmission,
    ) -> Result<(), RenderFrameworkError> {
        if !self.config.pipelined_render {
            self.finish_pending()?;
            let _operation_guard = core.lock_operation();
            return submission.execute(core.as_ref());
        }

        self.ensure_queue(core)?;
        let queue = self
            .queue
            .as_mut()
            .ok_or_else(|| queue_error(PipelinedSubmissionQueueError::WorkerUnavailable))?;
        let result = queue.submit(submission).map_err(queue_error)?;
        result.unwrap_or(Ok(()))
    }

    fn ensure_queue(
        &mut self,
        core: Arc<WgpuRenderFrameworkCore>,
    ) -> Result<(), RenderFrameworkError> {
        if self.queue.is_some() {
            return Ok(());
        }
        self.queue = Some(
            PipelinedSubmissionQueue::<FrameSubmission, Result<(), RenderFrameworkError>>::new(
                move |submission, started| {
                    let _operation_guard = core.lock_operation();
                    let _ = started.send(());
                    // Do not include operation-lock wait in worker occupancy:
                    // the producer's start signal remains its own wait scope.
                    crate::profile_counter!(
                        "runtime",
                        "render_framework.scheduler.worker_utilization",
                        1
                    );
                    let result = submission.execute(core.as_ref());
                    crate::profile_counter!(
                        "runtime",
                        "render_framework.scheduler.worker_utilization",
                        0
                    );
                    result
                },
            )
            .map_err(queue_error)?,
        );
        Ok(())
    }

    pub(in crate::graphics::runtime::render_framework) fn finish_pending(
        &mut self,
    ) -> Result<(), RenderFrameworkError> {
        let Some(queue) = self.queue.as_mut() else {
            return Ok(());
        };
        match queue.finish().map_err(queue_error)? {
            Some(result) => result,
            None => Ok(()),
        }
    }
}

fn queue_error(error: PipelinedSubmissionQueueError) -> RenderFrameworkError {
    RenderFrameworkError::Backend(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crossbeam_channel::Sender;

    use super::PipelinedSubmissionQueue;

    #[test]
    fn render_perf_pipelined_feedback_is_one_submission_late() {
        let started = Arc::new(Mutex::new(Vec::new()));
        let worker_started = Arc::clone(&started);
        let mut queue = PipelinedSubmissionQueue::new(move |frame, ready: &Sender<()>| {
            worker_started.lock().unwrap().push(frame);
            ready.send(()).unwrap();
            frame
        })
        .unwrap();

        assert_eq!(queue.submit(1).unwrap(), None);
        assert_eq!(*started.lock().unwrap(), vec![1]);
        assert_eq!(queue.submit(2).unwrap(), Some(1));
        assert_eq!(queue.finish().unwrap(), Some(2));
    }

    #[test]
    fn finish_reports_a_completed_worker_error() {
        let mut queue = PipelinedSubmissionQueue::new(|(), ready: &Sender<()>| {
            ready.send(()).expect("test worker should signal readiness");
            Err::<(), _>("submission failed")
        })
        .expect("test queue should create its worker");

        assert_eq!(queue.submit(()).unwrap(), None);
        assert_eq!(queue.finish().unwrap(), Some(Err("submission failed")));
    }

    #[test]
    fn scheduler_profile_scopes_keep_submission_waits_distinct() {
        let source = include_str!("queue.rs");

        for name in [
            "wait_previous_submission",
            "wait_worker_start",
            "wait_pending_submission",
            "pending_depth",
            "worker_utilization",
        ] {
            assert!(
                source.contains(name),
                "scheduler profiling must retain the `{name}` observation point"
            );
        }

        let worker_execution = source
            .split("move |submission, started| {")
            .nth(1)
            .expect("scheduler must retain a worker execution closure");
        let operation_lock = worker_execution
            .find("let _operation_guard = core.lock_operation();")
            .expect("worker must retain the RHI operation owner");
        let started = worker_execution
            .find("let _ = started.send(());")
            .expect("worker must retain its producer start signal");
        let busy = worker_execution
            .find("worker_utilization")
            .expect("worker occupancy must begin after the operation lock");
        let execute = worker_execution
            .find("let result = submission.execute(core.as_ref());")
            .expect("worker must execute its sealed submission");
        let idle = worker_execution[execute..]
            .find("worker_utilization")
            .map(|relative_index| execute + relative_index)
            .expect("worker occupancy must finish after submission execution");

        assert!(operation_lock < started && started < busy && busy < execute && execute < idle);
    }
}
