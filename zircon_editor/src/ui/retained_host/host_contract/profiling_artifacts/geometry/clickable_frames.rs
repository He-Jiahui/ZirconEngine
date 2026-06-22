use super::super::{UiProfileNamedFrame, UiProfileTabFrame};

pub(in crate::ui::retained_host::host_contract) fn collect_clickable_frames(
    resize_splitters: &[UiProfileNamedFrame],
    document_tabs: &[UiProfileTabFrame],
    drawer_tabs: &[UiProfileTabFrame],
    host_page_tabs: &[UiProfileTabFrame],
    activity_rail_buttons: &[UiProfileNamedFrame],
    viewport_toolbar_controls: &[UiProfileNamedFrame],
    template_controls: &[UiProfileNamedFrame],
) -> Vec<UiProfileNamedFrame> {
    let mut clickable_frames = Vec::new();
    clickable_frames.extend(resize_splitters.iter().cloned());
    clickable_frames.extend(document_tabs.iter().map(UiProfileNamedFrame::from_tab));
    clickable_frames.extend(drawer_tabs.iter().map(UiProfileNamedFrame::from_tab));
    clickable_frames.extend(host_page_tabs.iter().map(UiProfileNamedFrame::from_tab));
    clickable_frames.extend(activity_rail_buttons.iter().cloned());
    clickable_frames.extend(viewport_toolbar_controls.iter().cloned());
    clickable_frames.extend(template_controls.iter().cloned());
    clickable_frames
}
