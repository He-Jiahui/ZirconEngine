use super::super::super::super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::super::super::super::diagnostics::HostRefreshDiagnostics;

pub(in crate::ui::retained_host::host_contract) struct PlannedPresent<'a> {
    pub(in crate::ui::retained_host::host_contract) presentation: &'a HostWindowPresentationData,
    pub(in crate::ui::retained_host::host_contract) damage: Option<FrameRect>,
    pub(in crate::ui::retained_host::host_contract) diagnostics: HostRefreshDiagnostics,
    pub(in crate::ui::retained_host::host_contract) overlay_text: String,
}

pub(super) fn planned_present<'a>(
    presentation: &'a HostWindowPresentationData,
    damage: Option<FrameRect>,
    diagnostics: HostRefreshDiagnostics,
    overlay_text: String,
) -> PlannedPresent<'a> {
    PlannedPresent {
        presentation,
        damage,
        diagnostics,
        overlay_text,
    }
}
