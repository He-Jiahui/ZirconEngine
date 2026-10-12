use super::super::super::data::HostWindowLayoutData;
use super::super::UiProfileLayout;

pub(in crate::ui::retained_host::host_contract) fn profile_layout(
    layout: &HostWindowLayoutData,
) -> UiProfileLayout {
    UiProfileLayout {
        center_band: layout.center_band_frame.clone().into(),
        document_region: layout.document_region_frame.clone().into(),
        left_region: layout.left_region_frame.clone().into(),
        right_region: layout.right_region_frame.clone().into(),
        bottom_region: layout.bottom_region_frame.clone().into(),
        status_bar: layout.status_bar_frame.clone().into(),
    }
}
