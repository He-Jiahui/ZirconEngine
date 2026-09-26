use std::sync::Arc;

use super::terminal::PreferenceMutationTerminal;

/// 读取视图的持久化进度；可见值不等于已落盘，失败后可能仍保留本地可见值。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreferenceDurabilityState {
    Durable,
    Pending,
    VisibleNotDurable,
}

/// 某一键的可见代际视图；首次读取可能先返回 Pending，值为 None 不必然表示不存在。
/// 使用方应检查 durability 与 last_terminal，再决定是否等待或重试。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreferenceReadSnapshot {
    generation: u64,
    value: Option<Arc<[u8]>>,
    durability: PreferenceDurabilityState,
    last_terminal: Option<PreferenceMutationTerminal>,
}

/// 显式丢弃失败且尚未持久化的可见代际后返回的审计事实。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreferenceEviction {
    generation: u64,
    durability: PreferenceDurabilityState,
    last_terminal: Option<PreferenceMutationTerminal>,
}

impl PreferenceEviction {
    pub(crate) fn new(
        generation: u64,
        durability: PreferenceDurabilityState,
        last_terminal: Option<PreferenceMutationTerminal>,
    ) -> Self {
        Self {
            generation,
            durability,
            last_terminal,
        }
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn durability(&self) -> PreferenceDurabilityState {
        self.durability
    }

    pub fn last_terminal(&self) -> Option<&PreferenceMutationTerminal> {
        self.last_terminal.as_ref()
    }
}

impl PreferenceReadSnapshot {
    pub(crate) fn new(
        generation: u64,
        value: Option<Arc<[u8]>>,
        durability: PreferenceDurabilityState,
        last_terminal: Option<PreferenceMutationTerminal>,
    ) -> Self {
        Self {
            generation,
            value,
            durability,
            last_terminal,
        }
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn value(&self) -> Option<&[u8]> {
        self.value.as_deref()
    }

    pub const fn durability(&self) -> PreferenceDurabilityState {
        self.durability
    }

    pub fn last_terminal(&self) -> Option<&PreferenceMutationTerminal> {
        self.last_terminal.as_ref()
    }
}
