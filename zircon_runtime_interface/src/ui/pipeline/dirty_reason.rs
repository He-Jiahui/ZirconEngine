use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPipelineDirtyReason {
    Input,
    Focus,
    WidgetBehavior,
    Text,
    Style,
    Layout,
    LayoutMetrics,
    Picking,
    HitGrid,
    A11y,
    Render,
    Template,
    Window,
    HostRequest,
    Diagnostics,
}
