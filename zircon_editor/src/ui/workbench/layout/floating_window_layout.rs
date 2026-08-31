use serde::{Deserialize, Serialize};

use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::view::ViewInstanceId;

use super::{DocumentNode, MainPageId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FloatingWindowLayout {
    pub window_id: MainPageId,
    pub title: String,
    pub workspace: DocumentNode,
    pub focused_view: Option<ViewInstanceId>,
    #[serde(with = "strict_shell_frame")]
    pub frame: ShellFrame,
}

mod strict_shell_frame {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use crate::ui::workbench::autolayout::ShellFrame;

    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ShellFrameWire {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    }

    pub(super) fn serialize<S>(frame: &ShellFrame, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ShellFrameWire {
            x: frame.x,
            y: frame.y,
            width: frame.width,
            height: frame.height,
        }
        .serialize(serializer)
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<ShellFrame, D::Error>
    where
        D: Deserializer<'de>,
    {
        let frame = ShellFrameWire::deserialize(deserializer)?;
        Ok(ShellFrame::new(frame.x, frame.y, frame.width, frame.height))
    }
}
