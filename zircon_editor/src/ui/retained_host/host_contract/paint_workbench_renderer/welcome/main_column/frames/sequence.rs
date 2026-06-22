mod form;
mod top;

use super::super::super::super::super::data::{FrameRect, PaneData};

use super::fallback::welcome_main_column_frame_metrics;
use super::model::WelcomeMainColumnFrames;
use form::resolve_form_frames;
use top::resolve_top_frames;

pub(in super::super) fn welcome_main_column_frames(
    pane: &PaneData,
    body: &FrameRect,
    main_panel: &FrameRect,
) -> WelcomeMainColumnFrames {
    let metrics = welcome_main_column_frame_metrics(main_panel);
    let top = resolve_top_frames(pane, body, main_panel, &metrics);
    let form = resolve_form_frames(pane, body, &metrics, &top.header);
    WelcomeMainColumnFrames {
        hero: top.hero,
        status: top.status,
        header: top.header,
        name: form.name,
        location: form.location,
        preview: form.preview,
        validation: form.validation,
        actions: form.actions,
    }
}
