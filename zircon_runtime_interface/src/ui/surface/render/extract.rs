use serde::{Deserialize, Serialize};

use super::{UiRenderCommandKind, UiRenderList};
use crate::ui::event_ui::UiTreeId;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiRenderExtract {
    pub tree_id: UiTreeId,
    pub list: UiRenderList,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiRenderExtractKind {
    #[default]
    LegacyCommandList,
    PaintElements,
    BatchedPaint,
    TextSelectionCursor,
    DebugOverlay,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiRenderStats {
    pub command_count: usize,
    pub quad_command_count: usize,
    pub text_command_count: usize,
    pub image_command_count: usize,
    pub group_command_count: usize,
    pub clipped_command_count: usize,
}

impl UiRenderStats {
    pub fn from_extract(extract: &UiRenderExtract) -> Self {
        let mut stats = Self {
            command_count: extract.list.commands.len(),
            ..Self::default()
        };

        for command in &extract.list.commands {
            match command.kind {
                UiRenderCommandKind::Group => stats.group_command_count += 1,
                UiRenderCommandKind::Quad => stats.quad_command_count += 1,
                UiRenderCommandKind::Text => stats.text_command_count += 1,
                UiRenderCommandKind::Image => stats.image_command_count += 1,
            }
            if command.clip_frame.is_some() {
                stats.clipped_command_count += 1;
            }
        }

        stats
    }
}
