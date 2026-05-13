use std::cell::RefCell;

use zircon_runtime_interface::{ProfileCounterSnapshot, ProfileFrameSnapshot, ProfileSpanSnapshot};

use super::with_recorder;

thread_local! {
    static SPAN_STACK: RefCell<Vec<SpanStackEntry>> = const { RefCell::new(Vec::new()) };
    static FRAME_STACK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}

#[derive(Clone, Debug)]
struct SpanStackEntry {
    id: u64,
    path: String,
}

#[derive(Clone, Debug)]
pub(crate) struct ProfileScopeToken {
    id: u64,
    parent_id: Option<u64>,
    frame_index: Option<u64>,
    stream: &'static str,
    category: &'static str,
    name: &'static str,
    path: String,
    start_us: u64,
    depth: u16,
}

#[derive(Clone, Debug)]
pub(crate) struct ProfileFrameToken {
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
}

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

pub(crate) fn begin_scope(
    stream: &'static str,
    category: &'static str,
    name: &'static str,
) -> Option<ProfileScopeToken> {
    let (parent_id, parent_path, depth) = SPAN_STACK.with(|stack| {
        let stack = stack.borrow();
        let parent = stack.last();
        (
            parent.map(|entry| entry.id),
            parent.map(|entry| entry.path.clone()),
            stack.len().min(u16::MAX as usize) as u16,
        )
    });
    let frame_index = FRAME_STACK.with(|stack| stack.borrow().last().copied());
    let path = match parent_path {
        Some(parent_path) => format!("{parent_path}/{category}:{name}"),
        None => format!("{stream}/{category}:{name}"),
    };
    let token = with_recorder(|recorder| {
        if !recorder.is_active() {
            return None;
        }
        Some(ProfileScopeToken {
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
            id: token.id,
            path: token.path.clone(),
        });
    });
    Some(token)
}

pub(crate) fn finish_scope(token: ProfileScopeToken) {
    let end_us = with_recorder(|recorder| recorder.now_us());
    let duration_us = end_us.saturating_sub(token.start_us);
    SPAN_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        if stack.last().is_some_and(|entry| entry.id == token.id) {
            stack.pop();
        } else if let Some(index) = stack.iter().rposition(|entry| entry.id == token.id) {
            stack.remove(index);
        }
    });
    with_recorder(|recorder| {
        recorder.record_span(ProfileSpanSnapshot {
            id: token.id,
            parent_id: token.parent_id,
            frame_index: token.frame_index,
            stream: token.stream.to_string(),
            category: token.category.to_string(),
            name: token.name.to_string(),
            path: token.path,
            start_us: token.start_us,
            duration_us,
            depth: token.depth,
        });
    });
}

pub(crate) fn begin_frame(stream: &'static str, name: &'static str) -> Option<ProfileFrameToken> {
    let token = with_recorder(|recorder| {
        if !recorder.is_active() {
            return None;
        }
        Some(ProfileFrameToken {
            stream,
            name,
            frame_index: recorder.next_frame_index(stream),
            start_us: recorder.now_us(),
            budget_ms: recorder.config().frame_budget_ms,
        })
    })?;
    FRAME_STACK.with(|stack| stack.borrow_mut().push(token.frame_index));
    Some(token)
}

pub(crate) fn finish_frame(token: ProfileFrameToken) {
    let end_us = with_recorder(|recorder| recorder.now_us());
    let duration_us = end_us.saturating_sub(token.start_us);
    FRAME_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        if stack.last().copied() == Some(token.frame_index) {
            stack.pop();
        } else if let Some(index) = stack.iter().rposition(|frame| *frame == token.frame_index) {
            stack.remove(index);
        }
    });
    with_recorder(|recorder| {
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

pub(crate) fn record_counter(stream: &'static str, name: &'static str, value: f64) {
    let frame_index = FRAME_STACK.with(|stack| stack.borrow().last().copied());
    with_recorder(|recorder| {
        if !recorder.is_active() {
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
