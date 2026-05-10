use super::inspector_constants::{
    INSPECTOR_ACTIONS_HEIGHT, INSPECTOR_DIVIDER_HEIGHT, INSPECTOR_FIELD_HEIGHT,
    INSPECTOR_HEADER_HEIGHT, INSPECTOR_POSITION_HEIGHT, INSPECTOR_ROW_GAP,
    INSPECTOR_VIEWPORT_EXTRA,
};

pub(crate) fn inspector_content_extent() -> f32 {
    let sections = [
        INSPECTOR_HEADER_HEIGHT,
        INSPECTOR_FIELD_HEIGHT,
        INSPECTOR_FIELD_HEIGHT,
        INSPECTOR_POSITION_HEIGHT,
        INSPECTOR_DIVIDER_HEIGHT,
        INSPECTOR_ACTIONS_HEIGHT,
    ];
    let content = sections.into_iter().sum::<f32>();
    let gaps = (sections.len().saturating_sub(1) as f32) * INSPECTOR_ROW_GAP;
    content + gaps + INSPECTOR_VIEWPORT_EXTRA
}
