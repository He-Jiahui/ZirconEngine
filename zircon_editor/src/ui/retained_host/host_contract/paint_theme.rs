mod metrics;
mod model;
mod palette_projection;
mod tokens;
mod typography;

use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

pub(crate) fn apply_host_appearance_from_tokens(tokens: &EditorDesignTokens) {
    apply_host_metrics_from_tokens(tokens);
    apply_host_palette_from_tokens(tokens);
    apply_host_text_preferences(project_host_text_preferences(tokens));
}

pub(crate) use metrics::apply_host_metrics_from_tokens;
// Pointer hit testing consumes the same retained-host density metrics as the
// painter so themed row geometry stays aligned with its interactive surface.
pub(in crate::ui::retained_host) use metrics::{HostControlMetrics, METRICS, current_host_metrics};
pub(in crate::ui::retained_host::host_contract) use model::HostMaterialPalette;
pub(crate) use palette_projection::apply_host_palette_from_tokens;
pub(in crate::ui::retained_host::host_contract) use palette_projection::current_host_palette;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use palette_projection::project_host_palette;
pub(in crate::ui::retained_host::host_contract) use tokens::PALETTE;
pub(crate) use typography::{
    HostTextPreferences, HostTextSmoothing, HostUtilityTabTextRole, apply_host_text_preferences,
    current_host_text_preferences, project_host_text_preferences,
};
