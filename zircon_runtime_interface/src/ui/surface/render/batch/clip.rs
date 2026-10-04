use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::super::{UiClipMode, UiClipState};

#[cfg(test)]
#[path = "clip/tests/performance_tests.rs"]
mod performance_tests;

/// Canonical clip states referenced by one ordered batch plan.
///
/// The interface keeps the structured state in `UiBatchKey` so consumers can
/// serialize and inspect a plan without backend-private numeric handles. The
/// table removes duplicate state storage and gives the GPU submission layer a
/// single place to assign backend handles later.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(super) struct UiBatchClipStates {
    states: Vec<UiClipState>,
    // 该索引只用于加速查找，反序列化后可重建，因此不属于序列化字段。
    #[serde(skip)]
    indices: HashMap<UiClipStateIdentity, usize>,
}

impl PartialEq for UiBatchClipStates {
    fn eq(&self, other: &Self) -> bool {
        self.states == other.states
    }
}

impl UiBatchClipStates {
    pub(super) fn intern(&mut self, state: UiClipState) -> UiClipState {
        self.rebuild_indices_if_needed();
        let identity = UiClipStateIdentity::from(&state);
        if let Some(&index) = self.indices.get(&identity) {
            return self.states[index].clone();
        }
        self.indices.insert(identity, self.states.len());
        self.states.push(state.clone());
        state
    }

    fn rebuild_indices_if_needed(&mut self) {
        if self.indices.len() == self.states.len() {
            return;
        }
        self.indices = self
            .states
            .iter()
            .enumerate()
            .map(|(index, state)| (UiClipStateIdentity::from(state), index))
            .collect();
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.states.len()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct UiClipStateIdentity {
    mode: u8,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl From<&UiClipState> for UiClipStateIdentity {
    fn from(state: &UiClipState) -> Self {
        Self {
            mode: clip_mode_identity(state.mode),
            x: normalized_float_bits(state.frame.x),
            y: normalized_float_bits(state.frame.y),
            width: normalized_float_bits(state.frame.width),
            height: normalized_float_bits(state.frame.height),
        }
    }
}

fn clip_mode_identity(mode: UiClipMode) -> u8 {
    match mode {
        UiClipMode::Scissor => 0,
        UiClipMode::Stencil => 1,
    }
}

fn normalized_float_bits(value: f32) -> u32 {
    // PartialEq 将正负零视为相等；哈希身份也必须合并这两种位模式。
    if value == 0.0 {
        0
    } else {
        value.to_bits()
    }
}

/// Stack semantics for nested clipping during render extraction.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct UiClipStack {
    states: UiBatchClipStates,
    stack: Vec<UiClipState>,
}

impl UiClipStack {
    pub(super) fn push(&mut self, requested: UiClipState) -> UiClipState {
        let effective = self
            .stack
            .last()
            .and_then(|parent| intersect_scissors(parent, &requested))
            .unwrap_or(requested);
        let effective = self.states.intern(effective);
        self.stack.push(effective.clone());
        effective
    }

    pub(super) fn pop(&mut self) -> Option<UiClipState> {
        self.stack.pop()
    }

    pub(super) fn resolve(&self, clip: &UiClipState) -> Option<&UiClipState> {
        let identity = UiClipStateIdentity::from(clip);
        if let Some(&index) = self.states.indices.get(&identity) {
            if let Some(state) = self.states.states.get(index).filter(|state| *state == clip) {
                return Some(state);
            }
        }
        // 反序列化或索引缺项时仍以规范状态表兜底，索引只影响查找速度。
        self.resolve_linear(clip)
    }

    fn resolve_linear(&self, clip: &UiClipState) -> Option<&UiClipState> {
        self.states.states.iter().find(|state| *state == clip)
    }
}

fn intersect_scissors(parent: &UiClipState, child: &UiClipState) -> Option<UiClipState> {
    // 只有矩形 Scissor 可在这里求交；其他裁剪模式继续保留请求状态。
    if parent.mode != UiClipMode::Scissor || child.mode != UiClipMode::Scissor {
        return None;
    }

    let left = parent.frame.x.max(child.frame.x);
    let top = parent.frame.y.max(child.frame.y);
    let right = parent.frame.right().min(child.frame.right());
    let bottom = parent.frame.bottom().min(child.frame.bottom());
    Some(UiClipState {
        mode: UiClipMode::Scissor,
        frame: crate::ui::layout::UiFrame::new(
            left,
            top,
            (right - left).max(0.0),
            (bottom - top).max(0.0),
        ),
    })
}
