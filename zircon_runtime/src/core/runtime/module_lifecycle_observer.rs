use std::fmt;

/// Neutral callback installed by an upper runtime domain that needs to follow
/// successful core module activation and pre-unload deactivation.
pub trait RuntimeModuleLifecycleObserver: fmt::Debug + Send + Sync {
    fn runtime_module_activated(&self, module_name: &str);

    fn runtime_module_deactivating(
        &self,
        module_name: &str,
    ) -> Result<(), RuntimeModuleLifecycleBlock>;
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
