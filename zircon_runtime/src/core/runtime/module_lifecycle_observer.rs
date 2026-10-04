use std::fmt;
use std::time::Instant;

/// Neutral callback installed by an upper runtime domain that needs to follow
/// successful core module activation and pre-unload deactivation.
/// 运行时模块变为 Running 后收到通知；卸载前回调可返回错误以阻止停用。
pub trait RuntimeModuleLifecycleObserver: fmt::Debug + Send + Sync {
    fn runtime_module_activated(&self, module_name: &str);

    fn runtime_module_deactivating(
        &self,
        module_name: &str,
    ) -> Result<(), RuntimeModuleLifecycleBlock>;

    fn runtime_module_deactivating_until(
        &self,
        module_name: &str,
        deadline: Instant,
    ) -> Result<(), RuntimeModuleLifecycleBlock> {
        let _ = deadline;
        self.runtime_module_deactivating(module_name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeModuleLifecycleBlock {
    diagnostic: String,
}

impl RuntimeModuleLifecycleBlock {
    pub fn new(diagnostic: impl Into<String>) -> Self {
        Self {
            diagnostic: diagnostic.into(),
        }
    }

    pub fn diagnostic(&self) -> &str {
        &self.diagnostic
    }
}

impl fmt::Display for RuntimeModuleLifecycleBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.diagnostic)
    }
}

impl std::error::Error for RuntimeModuleLifecycleBlock {}
