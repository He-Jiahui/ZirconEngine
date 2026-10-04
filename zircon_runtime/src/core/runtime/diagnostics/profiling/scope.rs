use std::cell::RefCell;

use zircon_runtime_interface::{ProfileCounterSnapshot, ProfileFrameSnapshot, ProfileSpanSnapshot};

use super::with_recorder;

thread_local! {
    static SPAN_STACK: RefCell<Vec<SpanStackEntry>> = const { RefCell::new(Vec::new()) };
    static FRAME_STACK: RefCell<Vec<FrameStackEntry>> = const { RefCell::new(Vec::new()) };
}

#[derive(Clone, Debug)]
struct SpanStackEntry {
    capture_epoch: u64,
    id: u64,
    path: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FrameStackEntry {
    capture_epoch: u64,
    frame_index: u64,
}

#[derive(Clone, Debug)]
pub(crate) struct ProfileScopeToken {
    capture_epoch: u64,
    id: u64,
    parent_id: Option<u64>,
    frame_index: Option<u64>,
    stream: &'static str,
    category: &'static str,
    name: String,
    path: String,
    start_us: u64,
    depth: u16,
}

#[derive(Clone, Debug)]
pub(crate) struct ProfileFrameToken {
    capture_epoch: u64,
    stream: &'static str,
    name: &'static str,
    frame_index: u64,
    start_us: u64,
    budget_ms: f64,
}

#[derive(Debug)]
pub struct ProfileScope {
    token: Option<ProfileScopeToken>,
}

impl ProfileScope {
    pub fn enter(stream: &'static str, category: &'static str, name: &'static str) -> Self {
        Self {
            token: super::begin_scope(stream, category, name),
        }
    }

    pub fn enter_named(
        stream: &'static str,
        category: &'static str,
        name: impl Into<String>,
    ) -> Self {
        let name = name.into();
        Self {
            token: super::begin_scope_named(stream, category, name),
        }
    }
}

// scope 退出时由 recorder 计算主机侧经过时间；GPU 耗时由独立的异步计时链提供。
impl Drop for ProfileScope {
    fn drop(&mut self) {
        if let Some(token) = self.token.take() {
            super::finish_scope(token);
        }
    }
}

#[derive(Debug)]
pub struct ProfileFrameScope {
    token: Option<ProfileFrameToken>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ProfileFrameContext {
    capture_epoch: Option<u64>,
    frame_index: Option<u64>,
}

#[derive(Debug)]
pub(crate) struct ProfileFrameContextGuard {
    frame: Option<FrameStackEntry>,
}

impl ProfileFrameContext {
    // 工作线程可携带当前帧关联；attach 会再次校验 epoch，避免落入后续录制。
    pub(crate) fn capture() -> Self {
        let capture_epoch = super::capture_epoch();
        Self {
            capture_epoch,
            frame_index: capture_epoch.and_then(|capture_epoch| {
                FRAME_STACK.with(|stack| {
                    stack
                        .borrow()
                        .iter()
                        .rev()
                        .find(|frame| frame.capture_epoch == capture_epoch)
                        .map(|frame| frame.frame_index)
                })
            }),
        }
    }

    pub(crate) fn is_active() -> bool {
        let Some(capture_epoch) = super::capture_epoch() else {
            return false;
        };
        FRAME_STACK.with(|stack| {
            stack
                .borrow()
                .iter()
                .any(|frame| frame.capture_epoch == capture_epoch)
        })
    }

    pub(crate) fn attach(self) -> ProfileFrameContextGuard {
        let frame =
            self.capture_epoch
                .zip(self.frame_index)
                .and_then(|(capture_epoch, frame_index)| {
                    (super::capture_epoch() == Some(capture_epoch)).then_some(FrameStackEntry {
                        capture_epoch,
                        frame_index,
                    })
                });
        if let Some(frame) = frame {
            FRAME_STACK.with(|stack| stack.borrow_mut().push(frame));
        }
        ProfileFrameContextGuard { frame }
    }
}

impl Drop for ProfileFrameContextGuard {
    fn drop(&mut self) {
        if let Some(frame) = self.frame.take() {
            detach_frame(frame);
        }
    }
}

impl ProfileFrameScope {
    pub fn enter(stream: &'static str, name: &'static str) -> Self {
        Self {
            token: super::begin_frame(stream, name),
        }
    }
}

impl Drop for ProfileFrameScope {
    fn drop(&mut self) {
        if let Some(token) = self.token.take() {
            super::finish_frame(token);
        }
    }
}

pub(crate) fn begin_scope_named(
    capture_epoch: u64,
    stream: &'static str,
    category: &'static str,
    name: String,
) -> Option<ProfileScopeToken> {
    let (parent_id, parent_path, depth) = SPAN_STACK.with(|stack| {
        let stack = stack.borrow();
        let parent = stack
            .iter()
            .rev()
            .find(|entry| entry.capture_epoch == capture_epoch);
        (
            parent.map(|entry| entry.id),
            parent.map(|entry| entry.path.clone()),
            stack
                .iter()
                .filter(|entry| entry.capture_epoch == capture_epoch)
                .count()
                .min(u16::MAX as usize) as u16,
        )
    });
    let frame_index = FRAME_STACK.with(|stack| {
        stack
            .borrow()
            .iter()
            .rev()
            .find(|frame| frame.capture_epoch == capture_epoch)
            .map(|frame| frame.frame_index)
    });
    let path = match parent_path {
        Some(parent_path) => format!("{parent_path}/{category}:{name}"),
        None => format!("{stream}/{category}:{name}"),
    };
    let token = with_recorder(|recorder| {
        if !recorder.is_active() || super::capture_epoch() != Some(capture_epoch) {
            return None;
        }
        Some(ProfileScopeToken {
            capture_epoch,
            id: recorder.next_span_id(),
            parent_id,
            frame_index,
            stream,
            category,
            name,
            path,
            start_us: recorder.now_us(),
            depth,
        })
    })?;
    SPAN_STACK.with(|stack| {
        stack.borrow_mut().push(SpanStackEntry {
            capture_epoch: token.capture_epoch,
            id: token.id,
            path: token.path.clone(),
        });
    });
    Some(token)
}

pub(crate) fn finish_scope(token: ProfileScopeToken) {
    SPAN_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let is_token = |entry: &SpanStackEntry| {
            entry.capture_epoch == token.capture_epoch && entry.id == token.id
        };
        if stack.last().is_some_and(is_token) {
            stack.pop();
        } else if let Some(index) = stack.iter().rposition(is_token) {
            stack.remove(index);
        }
    });
    with_recorder(|recorder| {
        if super::capture_epoch_for_completion() != Some(token.capture_epoch) {
            return;
        }
        let duration_us = recorder.now_us().saturating_sub(token.start_us);
        recorder.record_span(ProfileSpanSnapshot {
            id: token.id,
            parent_id: token.parent_id,
            frame_index: token.frame_index,
            stream: token.stream.to_string(),
            category: token.category.to_string(),
            name: token.name,
            path: token.path,
            start_us: token.start_us,
            duration_us,
            depth: token.depth,
        });
    });
}

pub(crate) fn begin_frame(
    capture_epoch: u64,
    stream: &'static str,
    name: &'static str,
) -> Option<ProfileFrameToken> {
    let token = with_recorder(|recorder| {
        if !recorder.is_active() || super::capture_epoch() != Some(capture_epoch) {
            return None;
        }
        Some(ProfileFrameToken {
            capture_epoch,
            stream,
            name,
            frame_index: recorder.next_frame_index(stream),
            start_us: recorder.now_us(),
            budget_ms: recorder.config().frame_budget_ms,
        })
    })?;
    FRAME_STACK.with(|stack| {
        stack.borrow_mut().push(FrameStackEntry {
            capture_epoch: token.capture_epoch,
            frame_index: token.frame_index,
        });
    });
    Some(token)
}

pub(crate) fn finish_frame(token: ProfileFrameToken) {
    detach_frame(FrameStackEntry {
        capture_epoch: token.capture_epoch,
        frame_index: token.frame_index,
    });
    with_recorder(|recorder| {
        if super::capture_epoch_for_completion() != Some(token.capture_epoch) {
            return;
        }
        let duration_us = recorder.now_us().saturating_sub(token.start_us);
        recorder.record_frame(ProfileFrameSnapshot {
            stream: token.stream.to_string(),
            name: token.name.to_string(),
            frame_index: token.frame_index,
            start_us: token.start_us,
            duration_us,
            budget_ms: token.budget_ms,
            over_budget: (duration_us as f64 / 1_000.0) > token.budget_ms,
        });
    });
}

fn detach_frame(frame: FrameStackEntry) {
    FRAME_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        if stack.last().copied() == Some(frame) {
            stack.pop();
        } else if let Some(index) = stack.iter().rposition(|entry| *entry == frame) {
            stack.remove(index);
        }
    });
}

pub(crate) fn record_counter(
    capture_epoch: u64,
    stream: &'static str,
    name: &'static str,
    value: f64,
) {
    let frame_index = current_frame_index(capture_epoch);
    with_recorder(|recorder| {
        if !recorder.is_active() || super::capture_epoch() != Some(capture_epoch) {
            return;
        }
        recorder.record_counter(ProfileCounterSnapshot {
            stream: stream.to_string(),
            name: name.to_string(),
            value,
            timestamp_us: recorder.now_us(),
            frame_index,
        });
    });
}

pub(crate) fn record_counter_batch(
    capture_epoch: u64,
    stream: &'static str,
    counters: &[(&'static str, f64)],
) {
    if counters.is_empty() {
        return;
    }
    let frame_index = current_frame_index(capture_epoch);
    with_recorder(|recorder| {
        if !recorder.is_active() || super::capture_epoch() != Some(capture_epoch) {
            return;
        }
        let timestamp_us = recorder.now_us();
        for &(name, value) in counters {
            recorder.record_counter(ProfileCounterSnapshot {
                stream: stream.to_string(),
                name: name.to_string(),
                value,
                timestamp_us,
                frame_index,
            });
        }
    });
}

fn current_frame_index(capture_epoch: u64) -> Option<u64> {
    FRAME_STACK.with(|stack| {
        stack
            .borrow()
            .iter()
            .rev()
            .find(|frame| frame.capture_epoch == capture_epoch)
            .map(|frame| frame.frame_index)
    })
}
