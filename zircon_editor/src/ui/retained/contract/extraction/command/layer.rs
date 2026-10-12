use super::super::super::command::ChromeCommandLayer;
use crate::ui::retained_host::host_contract::paint_frame::HostRecordedPaintKind;

pub(super) fn chrome_command_layer_from_recorded(
    kind: &HostRecordedPaintKind,
    full_rebuild: bool,
) -> ChromeCommandLayer {
    match kind {
        HostRecordedPaintKind::Text { .. } => ChromeCommandLayer::Text,
        HostRecordedPaintKind::Image { .. } => ChromeCommandLayer::Viewport,
        HostRecordedPaintKind::Quad { .. } | HostRecordedPaintKind::Border { .. } => {
            if full_rebuild {
                ChromeCommandLayer::Static
            } else {
                ChromeCommandLayer::Dynamic
            }
        }
    }
}
