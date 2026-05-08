use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskPoolKind {
    #[default]
    Compute,
    AsyncCompute,
    Io,
}

impl TaskPoolKind {
    pub const fn default_thread_name(self) -> &'static str {
        match self {
            Self::Compute => "zircon-compute-task",
            Self::AsyncCompute => "zircon-async-compute-task",
            Self::Io => "zircon-io-task",
        }
    }
}
