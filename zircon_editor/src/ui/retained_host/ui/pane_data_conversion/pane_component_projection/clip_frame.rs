use crate::ui::retained_host as host_contract;
use zircon_runtime_interface::ui::layout::UiFrame;

pub(super) struct ProjectedClipFrame {
    pub(super) has_clip_frame: bool,
    pub(super) frame: host_contract::TemplateNodeFrameData,
}

pub(super) fn projected_clip_frame(clip_frame: Option<&UiFrame>) -> ProjectedClipFrame {
    ProjectedClipFrame {
        has_clip_frame: clip_frame.is_some(),
        frame: clip_frame
            .map(|clip| host_contract::TemplateNodeFrameData {
                x: clip.x,
                y: clip.y,
                width: clip.width,
                height: clip.height,
            })
            .unwrap_or_default(),
    }
}
