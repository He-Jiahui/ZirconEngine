//! 场景构造与 Winit 主循环之间的单任务交接。
//! 取消是协作信号；结果已发送仍不等于工作线程已经完全退出。

use std::any::Any;
use std::io;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, TryRecvError},
    Arc,
};
use std::thread::JoinHandle;
use std::time::Duration;

use zircon_runtime::core::runtime::tasks::spawn_named_thread;

/// 主循环读取后台场景的状态；完成分支把任务错误和汇合错误统一交给加载失败路径。
pub(crate) enum BackgroundTaskPoll<T> {
    Pending,
    Completed(Result<T, String>),
}

/// 场景构造阶段共用的协作取消令牌；请求取消不会强制停止当前阻塞工作。
#[derive(Clone)]
pub(crate) struct BackgroundTaskCancellation {
    cancelled: Arc<AtomicBool>,
}

impl BackgroundTaskCancellation {
    pub(crate) fn is_cancel_requested(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

/// 供终态记录区分已汇合的取消、线程错误与仍未停止的工作。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BackgroundTaskShutdown {
    CompletedAndJoined,
    CancelledAndJoined,
    TimedOut,
    JoinPanicked,
}

/// 主循环唯一持有的后台结果和线程所有权；取出结果或退出时负责汇合。
pub(crate) struct BackgroundTask<T> {
    receiver: Receiver<Result<T, String>>,
    cancellation: BackgroundTaskCancellation,
    join_handle: Option<JoinHandle<()>>,
}

impl<T: Send + 'static> BackgroundTask<T> {
    /// 由事件循环宿主启动可取消的场景构造；唤醒回调必须尽快返回，避免结果接收后的汇合阻塞。
    pub(crate) fn spawn(
        thread_name: &str,
        job: impl FnOnce(BackgroundTaskCancellation) -> Result<T, String> + Send + 'static,
        wake_event_loop: impl FnOnce() + Send + 'static,
    ) -> io::Result<Self> {
        let (sender, receiver) = mpsc::channel();
        let cancellation = BackgroundTaskCancellation {
            cancelled: Arc::new(AtomicBool::new(false)),
        };
        let worker_cancellation = cancellation.clone();
        let join_handle = spawn_named_thread(thread_name, move || {
            let result = if worker_cancellation.is_cancel_requested() {
                Err(background_cancellation_message())
            } else {
                catch_unwind(AssertUnwindSafe(|| job(worker_cancellation.clone())))
                    .map_err(background_panic_message)
                    .and_then(|result| result)
            };
            let result = if worker_cancellation.is_cancel_requested() {
                Err(background_cancellation_message())
            } else {
                result
            };
            let _ = sender.send(result);
            let _ = catch_unwind(AssertUnwindSafe(wake_event_loop));
        })
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;
        Ok(Self {
            receiver,
            cancellation,
            join_handle: Some(join_handle),
        })
    }

    pub(crate) fn request_cancel(&self) -> bool {
        !self.cancellation.cancelled.swap(true, Ordering::AcqRel)
    }

    pub(crate) fn is_cancellation_requested(&self) -> bool {
        self.cancellation.is_cancel_requested()
    }

    pub(crate) fn try_take(&mut self) -> BackgroundTaskPoll<T> {
        match self.receiver.try_recv() {
            Ok(result) => {
                BackgroundTaskPoll::Completed(self.join_worker().map_or_else(Err, |_| result))
            }
            Err(TryRecvError::Empty) => BackgroundTaskPoll::Pending,
            Err(TryRecvError::Disconnected) => {
                BackgroundTaskPoll::Completed(self.join_worker().map_or_else(Err, |_| {
                    Err(
                        "background scene loader disconnected before returning a result"
                            .to_string(),
                    )
                }))
            }
        }
    }

    /// 退出时消费任务并等待结果；超时分支分离仍在运行的线程，由终态报告清理失败。
    /// 等待期限只覆盖结果接收，收到结果后仍需等工作线程及唤醒回调退出。
    pub(crate) fn cancel_and_join(mut self, timeout: Duration) -> BackgroundTaskShutdown {
        match self.receiver.try_recv() {
            Ok(_) => return self.join_shutdown_outcome(),
            Err(TryRecvError::Disconnected) => return self.join_shutdown_outcome(),
            Err(TryRecvError::Empty) => {}
        }

        self.request_cancel();
        match self.receiver.recv_timeout(timeout) {
            Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => self.join_shutdown_outcome(),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if self
                    .join_handle
                    .as_ref()
                    .is_some_and(JoinHandle::is_finished)
                {
                    self.join_shutdown_outcome()
                } else {
                    BackgroundTaskShutdown::TimedOut
                }
            }
        }
    }

    fn join_shutdown_outcome(&mut self) -> BackgroundTaskShutdown {
        match self.join_worker() {
            Ok(()) if self.is_cancellation_requested() => {
                BackgroundTaskShutdown::CancelledAndJoined
            }
            Ok(()) => BackgroundTaskShutdown::CompletedAndJoined,
            Err(_) => BackgroundTaskShutdown::JoinPanicked,
        }
    }

    // TODO: [CR-APP-VIEWER-0003] 确认退出等待是否需要覆盖结果已发送但唤醒尚未返回的阶段；缺少此竞争测试；下一步检查事件代理唤醒和最终汇合的阻塞上界。
    fn join_worker(&mut self) -> Result<(), String> {
        let Some(join_handle) = self.join_handle.take() else {
            return Ok(());
        };
        join_handle.join().map_err(background_join_panic_message)
    }
}

fn background_panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        format!("background scene loader panicked: {message}")
    } else if let Some(message) = payload.downcast_ref::<String>() {
        format!("background scene loader panicked: {message}")
    } else {
        "background scene loader panicked with a non-string payload".to_string()
    }
}

fn background_join_panic_message(payload: Box<dyn Any + Send>) -> String {
    format!(
        "background scene loader panicked outside its job boundary: {}",
        background_panic_message(payload)
    )
}

fn background_cancellation_message() -> String {
    "background scene loader was cancelled before publication".to_string()
}

#[cfg(test)]
#[path = "tests/background_load.rs"]
mod tests;
