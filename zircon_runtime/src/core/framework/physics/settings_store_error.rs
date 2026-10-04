use thiserror::Error;

/// 区分只读实现与持久化失败，供调用方报告配置未写入的原因。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum PhysicsSettingsStoreError {
    #[error("physics settings are read-only for backend {backend}")]
    ReadOnlyBackend { backend: String },
    #[error("physics settings persistence failed: {message}")]
    Persistence { message: String },
}

impl PhysicsSettingsStoreError {
    pub fn read_only_backend(backend: impl Into<String>) -> Self {
        Self::ReadOnlyBackend {
            backend: backend.into(),
        }
    }

    pub fn persistence(message: impl Into<String>) -> Self {
        Self::Persistence {
            message: message.into(),
        }
    }
}
