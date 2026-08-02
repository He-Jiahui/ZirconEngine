use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsoleMessageFilter {
    #[default]
    All,
    Info,
    Warning,
    Error,
}

impl ConsoleMessageFilter {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}
