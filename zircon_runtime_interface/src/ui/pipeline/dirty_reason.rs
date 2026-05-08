use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPipelineDirtyReason {
    Input,
    Focus,
    Text,
    Style,
    Layout,
    LayoutMetrics,
    HitGrid,
    Render,
    Template,
    Window,
    HostRequest,
    Diagnostics,
}
