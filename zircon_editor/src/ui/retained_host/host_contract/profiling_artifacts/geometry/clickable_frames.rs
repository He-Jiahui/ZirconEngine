use super::super::{UiProfileNamedFrame, UiProfileTabFrame};

#[cfg(test)]
mod capacity_tests;

pub(in crate::ui::retained_host::host_contract) fn collect_clickable_frames(
    resize_splitters: &[UiProfileNamedFrame],
    document_tabs: &[UiProfileTabFrame],
    drawer_tabs: &[UiProfileTabFrame],
    host_page_tabs: &[UiProfileTabFrame],
    activity_rail_buttons: &[UiProfileNamedFrame],
    viewport_toolbar_controls: &[UiProfileNamedFrame],
    template_controls: &[UiProfileNamedFrame],
) -> Vec<UiProfileNamedFrame> {
    let mut clickable_frames = Vec::with_capacity(clickable_frame_capacity(
        resize_splitters,
        document_tabs,
        drawer_tabs,
        host_page_tabs,
        activity_rail_buttons,
        viewport_toolbar_controls,
        template_controls,
    ));
    clickable_frames.extend(resize_splitters.iter().cloned());
    clickable_frames.extend(document_tabs.iter().map(UiProfileNamedFrame::from_tab));
    clickable_frames.extend(drawer_tabs.iter().map(UiProfileNamedFrame::from_tab));
    clickable_frames.extend(host_page_tabs.iter().map(UiProfileNamedFrame::from_tab));
    clickable_frames.extend(activity_rail_buttons.iter().cloned());
    clickable_frames.extend(viewport_toolbar_controls.iter().cloned());
    clickable_frames.extend(template_controls.iter().cloned());
    clickable_frames
}

fn clickable_frame_capacity(
    resize_splitters: &[UiProfileNamedFrame],
    document_tabs: &[UiProfileTabFrame],
    drawer_tabs: &[UiProfileTabFrame],
    host_page_tabs: &[UiProfileTabFrame],
    activity_rail_buttons: &[UiProfileNamedFrame],
    viewport_toolbar_controls: &[UiProfileNamedFrame],
    template_controls: &[UiProfileNamedFrame],
) -> usize {
    resize_splitters
        .len()
        .saturating_add(document_tabs.len())
        .saturating_add(drawer_tabs.len())
        .saturating_add(host_page_tabs.len())
        .saturating_add(activity_rail_buttons.len())
        .saturating_add(viewport_toolbar_controls.len())
        .saturating_add(template_controls.len())
}
