use crate::ui::retained_host::console_output::ConsoleOutputPaintMetadata;
use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData};

pub(super) fn console_output_route_frame(pane: &PaneData, body: &FrameRect) -> Option<FrameRect> {
    let metadata = pane
        .console
        .nodes
        .metadata_rc::<ConsoleOutputPaintMetadata>()?;
    let viewport = metadata.viewport();
    (viewport.width > 0.0 && viewport.height > 0.0).then_some(FrameRect {
        x: body.x + viewport.x,
        y: body.y + viewport.y,
        width: viewport.width,
        height: viewport.height,
    })
}
