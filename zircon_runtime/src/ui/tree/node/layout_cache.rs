use serde::{Deserialize, Serialize};

use crate::ui::{layout::DesiredSize, layout::UiFrame, layout::UiSize, layout::UiVirtualListWindow};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiLayoutCache {
    pub desired_size: DesiredSize,
    pub frame: UiFrame,
    pub clip_frame: Option<UiFrame>,
    pub content_size: UiSize,
    pub virtual_window: Option<UiVirtualListWindow>,
}
