use super::prompt_text_layout_with_metrics;
use crate::ui::retained_host::host_contract::{
    data::{FrameRect, HostClosePromptData},
    paint_theme::METRICS,
};

#[test]
fn prompt_text_layout_projects_host_density_metrics() {
    let prompt = HostClosePromptData {
        dialog_frame: FrameRect {
            x: 10.0,
            y: 20.0,
            width: 360.0,
            height: 180.0,
        },
        ..HostClosePromptData::default()
    };

    let layout = prompt_text_layout_with_metrics(&prompt, METRICS);

    assert_eq!(layout.title_x, 28.0);
    assert_eq!(layout.title_y, 38.0);
    assert_eq!(layout.message_y, 68.0);
    assert_eq!(layout.details_frame.y, 96.0);
    assert_eq!(layout.details_frame.width, 324.0);
    assert_eq!(layout.details_frame.height, 42.0);
    assert_eq!(layout.details_x, 34.0);
    assert_eq!(layout.details_y, 106.0);
}
