//! 让编辑上下文、宿主和发现投影共用同一个命令目录所有者；锁作用于当前目录，克隆handle仍指向同一实例。

use std::sync::{Arc, Mutex, MutexGuard};

use super::EditorCommandRegistry;

/// Shared owner used by every runtime command discovery and invocation surface.
#[derive(Clone, Debug)]
pub struct EditorCommandRegistryHandle {
    registry: Arc<Mutex<EditorCommandRegistry>>,
}

impl EditorCommandRegistryHandle {
    pub fn new(registry: EditorCommandRegistry) -> Self {
        Self {
            registry: Arc::new(Mutex::new(registry)),
        }
    }

    pub fn default_workbench() -> Self {
        Self::new(EditorCommandRegistry::default_workbench())
    }

    /// 取得目录同步访问权；持锁期间的调用是否会进入插件回调须由执行链审查，展示快照应在释放锁后消费。
    pub fn lock(&self) -> MutexGuard<'_, EditorCommandRegistry> {
        self.registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
