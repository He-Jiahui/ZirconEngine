use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaneRouteNamespace {
    Dock,
    Draft,
    Selection,
    Animation,
    Diagnostics,
}

impl PaneRouteNamespace {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dock => "Dock",
            Self::Draft => "Draft",
            Self::Selection => "Selection",
            Self::Animation => "Animation",
            Self::Diagnostics => "Diagnostics",
        }
    }
}

impl fmt::Display for PaneRouteNamespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
