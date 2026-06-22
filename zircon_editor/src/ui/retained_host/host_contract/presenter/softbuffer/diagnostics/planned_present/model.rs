use super::super::super::super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::super::super::super::diagnostics::HostRefreshDiagnostics;

pub(in crate::ui::retained_host::host_contract) struct PlannedPresent {
    pub(in crate::ui::retained_host::host_contract) presentation: HostWindowPresentationData,
    pub(in crate::ui::retained_host::host_contract) damage: Option<FrameRect>,
    pub(in crate::ui::retained_host::host_contract) diagnostics: HostRefreshDiagnostics,
    pub(in crate::ui::retained_host::host_contract) overlay_text: String,
}

pub(super) fn planned_present(
    presentation: &HostWindowPresentationData,
    damage: Option<FrameRect>,
    diagnostics: HostRefreshDiagnostics,
    overlay_text: String,
) -> PlannedPresent {
    let mut presentation = presentation.clone();
    presentation.host_shell.debug_refresh_rate = overlay_text.clone().into();
    PlannedPresent {
        presentation,
        damage,
        diagnostics,
        overlay_text,
    }
}
