mod metrics;
mod model;
mod palette_projection;
mod tokens;
mod typography;

pub(crate) use metrics::apply_host_metrics_from_tokens;
pub(in crate::ui::retained_host::host_contract) use metrics::{
    current_host_metrics, HostControlMetrics, METRICS,
};
pub(in crate::ui::retained_host::host_contract) use model::HostMaterialPalette;
pub(crate) use palette_projection::apply_host_palette_from_tokens;
pub(in crate::ui::retained_host::host_contract) use palette_projection::current_host_palette;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use palette_projection::project_host_palette;
pub(in crate::ui::retained_host::host_contract) use tokens::PALETTE;
pub(crate) use typography::{
    apply_host_text_preferences, current_host_text_preferences, project_host_text_preferences,
    HostTextPreferences, HostTextSmoothing, HostUtilityTabTextRole,
};
