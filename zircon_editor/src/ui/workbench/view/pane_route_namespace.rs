//! pane控件事件的路由域身份，与载荷种类分离；注册表按此决定可接收的交互。
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaneRouteNamespace {
    Template,
    Dock,
    Draft,
    Selection,
    Animation,
    Diagnostics,
    UiComponentShowcase,
}

impl PaneRouteNamespace {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Template => "Template",
            Self::Dock => "Dock",
            Self::Draft => "Draft",
            Self::Selection => "Selection",
            Self::Animation => "Animation",
            Self::Diagnostics => "Diagnostics",
            Self::UiComponentShowcase => "UiComponentShowcase",
        }
    }
}

impl fmt::Display for PaneRouteNamespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
