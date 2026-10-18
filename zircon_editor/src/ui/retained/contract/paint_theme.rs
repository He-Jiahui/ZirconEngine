#[path = "paint_theme/metrics.rs"]
mod metrics;
#[path = "paint_theme/model.rs"]
mod model;
#[path = "paint_theme/normal_tokens.rs"]
mod normal_tokens;
#[path = "paint_theme/palette_projection.rs"]
mod palette_projection;
#[path = "paint_theme/tokens.rs"]
mod tokens;
#[path = "paint_theme/typography.rs"]
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
    token_input: Option<normal_tokens::HostPaintTokenInput>,
}

impl HostPaintThemeSnapshot {
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    fn from_tokens(
        tokens: Arc<EditorDesignTokens>,
        settings_generation: Option<u64>,
        generation: u64,
        scale_factor: f32,
    ) -> Self {
        let base_metrics = metrics::project_host_metrics(&tokens);
        Self {
            generation,
            base_metrics,
            metrics: base_metrics.at_scale(scale_factor),
            scale_factor,
            palette: palette_projection::project_host_palette(&tokens),
            text_preferences: Arc::new(typography::project_host_text_preferences(&tokens)),
            token_input: Some(normal_tokens::HostPaintTokenInput {
                tokens,
                settings_generation,
            }),
        }
    }

    fn with_metrics(&self, base_metrics: metrics::HostControlMetrics) -> Self {
        Self {
            generation: self.generation.saturating_add(1),
            base_metrics,
            metrics: base_metrics.at_scale(self.scale_factor),
            token_input: None,
            ..self.clone()
        }
    }

    fn with_palette(&self, palette: model::HostMaterialPalette) -> Self {
        Self {
            generation: self.generation.saturating_add(1),
            palette,
            token_input: None,
            ..self.clone()
        }
    }

    fn with_text_preferences(
        &self,
        text_preferences: Arc<typography::HostTextPreferences>,
    ) -> Self {
        Self {
            generation: self.generation.saturating_add(1),
            text_preferences,
            token_input: None,
            ..self.clone()
        }
    }

    fn at_scale(&self, scale_factor: f32) -> Self {
        Self {
            generation: self.generation.saturating_add(1),
            metrics: self.base_metrics.at_scale(scale_factor),
            scale_factor,
            ..self.clone()
        }
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

pub(crate) fn apply_host_appearance_from_tokens(
    tokens: Arc<EditorDesignTokens>,
    settings_generation: Option<u64>,
) {
    host_paint_theme_authority().rcu(move |current| {
        Arc::new(HostPaintThemeSnapshot::from_tokens(
            Arc::clone(&tokens),
            settings_generation,
            current.generation.saturating_add(1),
            current.scale_factor,
        ))
    });
}

pub(crate) fn capture_host_paint_theme_snapshot() -> Arc<HostPaintThemeSnapshot> {
    host_paint_theme_authority().load_full()
}

#[cfg(test)]
pub(crate) fn host_paint_theme_snapshot_from_tokens_for_test(
    tokens: &EditorDesignTokens,
) -> Arc<HostPaintThemeSnapshot> {
    Arc::new(HostPaintThemeSnapshot::from_tokens(
        Arc::new(tokens.clone()),
        None,
        0,
        1.0,
    ))
}

pub(crate) fn enter_host_paint_theme_scope(
    snapshot: Arc<HostPaintThemeSnapshot>,
) -> HostPaintThemeScope {
    let previous = ACTIVE_HOST_PAINT_THEME.with(|active| active.replace(Some(snapshot)));
    HostPaintThemeScope { previous }
}

fn replace_host_metrics(base_metrics: metrics::HostControlMetrics) {
    host_paint_theme_authority().rcu(move |current| Arc::new(current.with_metrics(base_metrics)));
}

fn replace_host_palette(palette: model::HostMaterialPalette) {
    host_paint_theme_authority().rcu(move |current| Arc::new(current.with_palette(palette)));
}

fn replace_host_text_preferences(text_preferences: typography::HostTextPreferences) {
    let text_preferences = Arc::new(text_preferences);
    host_paint_theme_authority()
        .rcu(move |current| Arc::new(current.with_text_preferences(Arc::clone(&text_preferences))));
}

fn host_metrics_for_read() -> metrics::HostControlMetrics {
    ACTIVE_HOST_PAINT_THEME
        .with(|active| active.borrow().as_ref().map(|snapshot| snapshot.metrics))
        .unwrap_or_else(|| host_paint_theme_authority().load().metrics)
}

pub(crate) fn current_host_paint_scale_factor() -> f32 {
    ACTIVE_HOST_PAINT_THEME
        .with(|active| {
            active
                .borrow()
                .as_ref()
                .map(|snapshot| snapshot.scale_factor)
        })
        .unwrap_or_else(|| host_paint_theme_authority().load().scale_factor)
}

#[cfg(test)]
pub(crate) fn host_paint_theme_snapshot_at_scale_for_test(
    scale_factor: f32,
) -> Arc<HostPaintThemeSnapshot> {
    let snapshot =
        host_paint_theme_snapshot_from_tokens_for_test(&EditorDesignTokens::workbench_dark());
    let mut snapshot = (*snapshot).clone();
    snapshot.scale_factor = scale_factor;
    snapshot.metrics = snapshot.base_metrics.at_scale(scale_factor);
    Arc::new(snapshot)
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
        Arc::new(current.at_scale(scale_factor))
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
            token_input: None,
        })
    })
}

pub(crate) use normal_tokens::active_host_paint_token_input;

pub(crate) use metrics::apply_host_metrics_from_tokens;
// Pointer hit testing consumes the same retained-host density metrics as the
// painter so themed row geometry stays aligned with its interactive surface.
pub(in crate::ui::retained_host::host_contract) use metrics::logical_font_size_to_physical;
pub(in crate::ui::retained_host) use metrics::METRICS;
pub(crate) use metrics::{current_host_metrics, HostControlMetrics};
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
