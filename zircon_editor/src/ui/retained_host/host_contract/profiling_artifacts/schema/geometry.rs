use serde::Serialize;

use super::frame::{UiProfileFrame, UiProfileLayout, UiProfileSize};
use super::hit::UiProfileHitSample;
use super::named::UiProfileNamedFrame;
use super::tab::UiProfileTabFrame;

#[derive(Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileGeometry {
    pub(in crate::ui::retained_host::host_contract) schema_version: u32,
    pub(in crate::ui::retained_host::host_contract) presenter_backend: &'static str,
    pub(in crate::ui::retained_host::host_contract) window_client_size: UiProfileSize,
    pub(in crate::ui::retained_host::host_contract) layout: UiProfileLayout,
    pub(in crate::ui::retained_host::host_contract) resize_splitters: Vec<UiProfileNamedFrame>,
    pub(in crate::ui::retained_host::host_contract) document_tabs: Vec<UiProfileTabFrame>,
    pub(in crate::ui::retained_host::host_contract) drawer_tabs: Vec<UiProfileTabFrame>,
    pub(in crate::ui::retained_host::host_contract) host_page_tabs: Vec<UiProfileTabFrame>,
    pub(in crate::ui::retained_host::host_contract) activity_rail_buttons: Vec<UiProfileNamedFrame>,
    pub(in crate::ui::retained_host::host_contract) viewport_frame: Option<UiProfileFrame>,
    pub(in crate::ui::retained_host::host_contract) viewport_toolbar_controls:
        Vec<UiProfileNamedFrame>,
    pub(in crate::ui::retained_host::host_contract) template_controls: Vec<UiProfileNamedFrame>,
    pub(in crate::ui::retained_host::host_contract) clickable_frames: Vec<UiProfileNamedFrame>,
    pub(in crate::ui::retained_host::host_contract) hit_samples: Vec<UiProfileHitSample>,
}
