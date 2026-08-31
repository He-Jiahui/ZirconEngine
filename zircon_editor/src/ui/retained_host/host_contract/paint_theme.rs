mod metrics;
mod model;
mod palette_projection;
mod tokens;
mod typography;

use std::cell::RefCell;
use std::sync::Arc;

use arc_swap::ArcSwap;
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

#[derive(Clone)]
pub(crate) struct HostPaintThemeSnapshot {
    generation: u64,
    base_metrics: metrics::HostControlMetrics,
    metrics: metrics::HostControlMetrics,
    scale_factor: f32,
    palette: model::HostMaterialPalette,
    text_preferences: Arc<typography::HostTextPreferences>,
}

impl HostPaintThemeSnapshot {
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
}

thread_local! {
    static ACTIVE_HOST_PAINT_THEME: RefCell<Option<Arc<HostPaintThemeSnapshot>>> =
        const { RefCell::new(None) };
}

pub(crate) struct HostPaintThemeScope {
    previous: Option<Arc<HostPaintThemeSnapshot>>,
}

impl Drop for HostPaintThemeScope {
    fn drop(&mut self) {
        let previous = self.previous.take();
        ACTIVE_HOST_PAINT_THEME.with(|active| *active.borrow_mut() = previous);
    }
}

pub(crate) fn apply_host_appearance_from_tokens(tokens: &EditorDesignTokens) {
    let base_metrics = metrics::project_host_metrics(tokens);
    let palette = palette_projection::project_host_palette(tokens);
    let text_preferences = Arc::new(typography::project_host_text_preferences(tokens));
    host_paint_theme_authority().rcu(move |current| {
        Arc::new(HostPaintThemeSnapshot {
            generation: current.generation.saturating_add(1),
            base_metrics,
            metrics: base_metrics.at_scale(current.scale_factor),
            scale_factor: current.scale_factor,
            palette,
            text_preferences: Arc::clone(&text_preferences),
        })
    });
}

pub(crate) fn capture_host_paint_theme_snapshot() -> Arc<HostPaintThemeSnapshot> {
    host_paint_theme_authority().load_full()
}

#[cfg(test)]
pub(crate) fn host_paint_theme_snapshot_from_tokens_for_test(
    tokens: &EditorDesignTokens,
) -> Arc<HostPaintThemeSnapshot> {
    let base_metrics = metrics::project_host_metrics(tokens);
    Arc::new(HostPaintThemeSnapshot {
        generation: 0,
        base_metrics,
        metrics: base_metrics,
        scale_factor: 1.0,
        palette: palette_projection::project_host_palette(tokens),
        text_preferences: Arc::new(typography::project_host_text_preferences(tokens)),
    })
}

pub(crate) fn enter_host_paint_theme_scope(
    snapshot: Arc<HostPaintThemeSnapshot>,
) -> HostPaintThemeScope {
    let previous = ACTIVE_HOST_PAINT_THEME.with(|active| active.replace(Some(snapshot)));
    HostPaintThemeScope { previous }
}

fn replace_host_metrics(base_metrics: metrics::HostControlMetrics) {
    host_paint_theme_authority().rcu(move |current| {
        Arc::new(HostPaintThemeSnapshot {
            generation: current.generation.saturating_add(1),
            base_metrics,
            metrics: base_metrics.at_scale(current.scale_factor),
            scale_factor: current.scale_factor,
            palette: current.palette,
            text_preferences: Arc::clone(&current.text_preferences),
        })
    });
}

fn replace_host_palette(palette: model::HostMaterialPalette) {
    host_paint_theme_authority().rcu(move |current| {
        Arc::new(HostPaintThemeSnapshot {
            generation: current.generation.saturating_add(1),
            base_metrics: current.base_metrics,
            metrics: current.metrics,
            scale_factor: current.scale_factor,
            palette,
            text_preferences: Arc::clone(&current.text_preferences),
        })
    });
}

fn replace_host_text_preferences(text_preferences: typography::HostTextPreferences) {
    let text_preferences = Arc::new(text_preferences);
    host_paint_theme_authority().rcu(move |current| {
        Arc::new(HostPaintThemeSnapshot {
            generation: current.generation.saturating_add(1),
            base_metrics: current.base_metrics,
            metrics: current.metrics,
            scale_factor: current.scale_factor,
            palette: current.palette,
            text_preferences: Arc::clone(&text_preferences),
        })
    });
}

fn host_metrics_for_read() -> metrics::HostControlMetrics {
    ACTIVE_HOST_PAINT_THEME
        .with(|active| active.borrow().as_ref().map(|snapshot| snapshot.metrics))
        .unwrap_or_else(|| host_paint_theme_authority().load().metrics)
}

pub(crate) fn apply_host_paint_scale_factor(scale_factor: f32) {
    let scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    };
    host_paint_theme_authority().rcu(move |current| {
        if (current.scale_factor - scale_factor).abs() <= f32::EPSILON {
            return Arc::clone(current);
        }
        Arc::new(HostPaintThemeSnapshot {
            generation: current.generation.saturating_add(1),
            base_metrics: current.base_metrics,
            metrics: current.base_metrics.at_scale(scale_factor),
            scale_factor,
            palette: current.palette,
            text_preferences: Arc::clone(&current.text_preferences),
        })
    });
}

fn host_palette_for_read() -> model::HostMaterialPalette {
    ACTIVE_HOST_PAINT_THEME
        .with(|active| active.borrow().as_ref().map(|snapshot| snapshot.palette))
        .unwrap_or_else(|| host_paint_theme_authority().load().palette)
}

fn host_text_preferences_for_read() -> Arc<typography::HostTextPreferences> {
    ACTIVE_HOST_PAINT_THEME
        .with(|active| {
            active
                .borrow()
                .as_ref()
                .map(|snapshot| Arc::clone(&snapshot.text_preferences))
        })
        .unwrap_or_else(|| Arc::clone(&host_paint_theme_authority().load().text_preferences))
}

fn host_paint_theme_authority() -> &'static ArcSwap<HostPaintThemeSnapshot> {
    static AUTHORITY: std::sync::OnceLock<ArcSwap<HostPaintThemeSnapshot>> =
        std::sync::OnceLock::new();
    AUTHORITY.get_or_init(|| {
        ArcSwap::from_pointee(HostPaintThemeSnapshot {
            generation: 0,
            base_metrics: metrics::METRICS,
            metrics: metrics::METRICS,
            scale_factor: 1.0,
            palette: palette_projection::DEFAULT_HOST_PALETTE,
            text_preferences: Arc::new(typography::HostTextPreferences::default()),
        })
    })
}

pub(crate) use metrics::apply_host_metrics_from_tokens;
// Pointer hit testing consumes the same retained-host density metrics as the
// painter so themed row geometry stays aligned with its interactive surface.
pub(in crate::ui::retained_host) use metrics::{current_host_metrics, HostControlMetrics, METRICS};
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
