//! 把交互命令上下文作为一个版本化快照共享给发现与可用性投影；代次只随语义改变递增，供缓存判断是否需要重算。

use std::sync::{Arc, RwLock};

use super::CommandEvalCtx;

/// The single interactive command-context snapshot owned by `EditorContext`.
#[derive(Clone, Debug, Default)]
pub struct CommandEvalSnapshotHandle {
    snapshot: Arc<RwLock<CommandEvalSnapshot>>,
}

#[derive(Clone, Debug)]
struct CommandEvalSnapshot {
    generation: u64,
    context: Arc<CommandEvalCtx>,
}

impl Default for CommandEvalSnapshot {
    fn default() -> Self {
        Self {
            generation: 0,
            context: Arc::new(CommandEvalCtx::default()),
        }
    }
}

impl CommandEvalSnapshotHandle {
    pub fn snapshot(&self) -> CommandEvalCtx {
        self.shared_snapshot().as_ref().clone()
    }

    pub fn shared_snapshot(&self) -> Arc<CommandEvalCtx> {
        self.shared_snapshot_with_generation().1
    }

    /// Reads the generation and context under one lock so consumers cannot pair mismatched state.
    pub fn snapshot_with_generation(&self) -> (u64, CommandEvalCtx) {
        let (generation, context) = self.shared_snapshot_with_generation();
        (generation, context.as_ref().clone())
    }

    /// 在同一次读锁内返回代次和上下文；缓存键必须使用这一配对，不能分两次读取后拼接。
    pub fn shared_snapshot_with_generation(&self) -> (u64, Arc<CommandEvalCtx>) {
        let snapshot = self
            .snapshot
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        (snapshot.generation, Arc::clone(&snapshot.context))
    }

    pub fn generation(&self) -> u64 {
        self.snapshot
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .generation
    }

    /// Replaces the shared context only when its command semantics changed.
    /// 由上下文所有者发布完整快照；返回值表示可用性语义是否改变，可据此避免无效投影刷新。
    pub fn replace(&self, context: CommandEvalCtx) -> bool {
        let mut snapshot = self
            .snapshot
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if snapshot.context.as_ref() == &context {
            return false;
        }
        snapshot.context = Arc::new(context);
        snapshot.generation = snapshot.generation.wrapping_add(1);
        true
    }
}

#[cfg(test)]
#[path = "tests/eval_snapshot_handle.rs"]
mod tests;
