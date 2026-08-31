use toml::Value;
use zircon_runtime_interface::ui::{
    component::{UiComponentFlags, UiComponentState},
    event_ui::UiStateFlags,
    style::UiPainterState,
    tree::UiTemplateNodeMetadata,
};

#[derive(Clone, Copy)]
pub(super) struct UiRenderPainterStateSource<'a> {
    metadata: Option<&'a UiTemplateNodeMetadata>,
    state_flags: &'a UiStateFlags,
    component_state: Option<&'a UiComponentState>,
}

impl<'a> UiRenderPainterStateSource<'a> {
    pub(super) fn new(
        metadata: Option<&'a UiTemplateNodeMetadata>,
        state_flags: &'a UiStateFlags,
        component_state: Option<&'a UiComponentState>,
    ) -> Self {
        Self {
            metadata,
            state_flags,
            component_state,
        }
    }

    pub(super) fn painter_state(self) -> UiPainterState {
        painter_state_from_source(self)
    }

    pub(super) fn painter_state_with_value_checked(self) -> UiPainterState {
        painter_state_with_value_checked_from_source(self)
    }
}

fn painter_state_from_source(source: UiRenderPainterStateSource<'_>) -> UiPainterState {
    let component_flags = source.component_state.map(|state| &state.flags);
    let metadata_focused = metadata_focused(source.metadata);
    UiPainterState {
        hovered: component_bool(component_flags, |flags| flags.hovered)
            || bool_attribute(source.metadata, "hovered").unwrap_or(false),
        pressed: component_bool(component_flags, |flags| flags.pressed)
            || source.state_flags.pressed
            || bool_attribute(source.metadata, "pressed").unwrap_or(false),
        focused: component_bool(component_flags, |flags| flags.focused) || metadata_focused,
        focus_visible: component_bool(component_flags, |flags| flags.focus_visible)
            || bool_attribute(source.metadata, "focus_visible")
                .or_else(|| bool_attribute(source.metadata, "focusVisible"))
                // Static painter fixtures predate a live focus owner. Preserve their focus
                // styling until runtime input establishes semantic focus for the component.
                .unwrap_or(
                    metadata_focused && !component_bool(component_flags, |flags| flags.focused),
                ),
        disabled: component_bool(component_flags, |flags| flags.disabled)
            || !source.state_flags.enabled
            || bool_attribute(source.metadata, "disabled").unwrap_or(false),
        checked: component_bool(component_flags, |flags| flags.checked)
            || source.state_flags.checked
            || bool_attribute(source.metadata, "checked").unwrap_or(false),
        selected: component_bool(component_flags, |flags| flags.selected)
            || bool_attribute(source.metadata, "selected").unwrap_or(false),
        open: component_bool(component_flags, |flags| flags.popup_open)
            || bool_attribute(source.metadata, "open")
                .or_else(|| bool_attribute(source.metadata, "popup_open"))
                .unwrap_or(false),
        dragging: component_bool(component_flags, |flags| flags.dragging)
            || bool_attribute(source.metadata, "dragging").unwrap_or(false),
        drop_hovered: component_bool(component_flags, |flags| {
            flags.drop_hovered || flags.active_drag_target
        }) || bool_attribute(source.metadata, "drop_hovered")
            .or_else(|| bool_attribute(source.metadata, "active_drag_target"))
            .unwrap_or(false),
        loading: component_bool(component_flags, |flags| flags.loading)
            || bool_attribute(source.metadata, "loading").unwrap_or(false),
    }
}

fn metadata_focused(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    bool_attribute(metadata, "focused").unwrap_or(false)
}

fn painter_state_with_value_checked_from_source(
    source: UiRenderPainterStateSource<'_>,
) -> UiPainterState {
    let metadata = source.metadata;
    let mut state = source.painter_state();
    state.checked = state.checked || bool_attribute(metadata, "value").unwrap_or(false);
    state
}

fn component_bool(
    component_flags: Option<&UiComponentFlags>,
    selector: impl FnOnce(&UiComponentFlags) -> bool,
) -> bool {
    component_flags.is_some_and(selector)
}

fn bool_attribute(metadata: Option<&UiTemplateNodeMetadata>, key: &str) -> Option<bool> {
    metadata
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(Value::as_bool)
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const RESOLUTIONS_PER_SAMPLE: usize = 262_144;

    #[test]
    fn static_focus_preview_yields_to_runtime_focus_visibility() {
        let mut metadata = UiTemplateNodeMetadata::default();
        metadata
            .attributes
            .insert("focused".to_string(), Value::Boolean(true));
        let state_flags = UiStateFlags {
            enabled: true,
            ..UiStateFlags::default()
        };

        let static_preview =
            UiRenderPainterStateSource::new(Some(&metadata), &state_flags, None).painter_state();
        assert!(static_preview.focused);
        assert!(static_preview.focus_visible);

        let mut pointer_focus = UiComponentState::default();
        pointer_focus.flags.focused = true;
        let pointer =
            UiRenderPainterStateSource::new(Some(&metadata), &state_flags, Some(&pointer_focus))
                .painter_state();
        assert!(pointer.focused);
        assert!(!pointer.focus_visible);

        pointer_focus.flags.focus_visible = true;
        let keyboard =
            UiRenderPainterStateSource::new(Some(&metadata), &state_flags, Some(&pointer_focus))
                .painter_state();
        assert!(keyboard.focus_visible);
    }

    #[test]
    fn optimization_batch_fr_runtime474_reads_static_focus_once() {
        let source = include_str!("painter_state.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("painter-state production source");

        assert_eq!(
            production
                .matches("metadata_focused(source.metadata)")
                .count(),
            1
        );
        assert_eq!(
            production
                .matches("bool_attribute(metadata, \"focused\")")
                .count(),
            1
        );
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fr_runtime474_single_static_focus_lookup_benchmark() {
        let mut metadata = UiTemplateNodeMetadata::default();
        for index in 0..16 {
            metadata.attributes.insert(
                format!("representative_attribute_{index:02}"),
                Value::Boolean(index == 7),
            );
        }
        metadata
            .attributes
            .insert("focused".to_owned(), Value::Boolean(true));

        for _ in 0..4 {
            black_box(measure_focus_resolution(&metadata, false));
            black_box(measure_focus_resolution(&metadata, true));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_focus_resolution(&metadata, false));
                optimized_samples.push(measure_focus_resolution(&metadata, true));
            } else {
                optimized_samples.push(measure_focus_resolution(&metadata, true));
                legacy_samples.push(measure_focus_resolution(&metadata, false));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME474_SINGLE_STATIC_FOCUS_LOOKUP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} resolutions_per_sample={RESOLUTIONS_PER_SAMPLE} metadata_attributes={} legacy_focused_lookups_per_resolution=2 optimized_focused_lookups_per_resolution=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=35",
            metadata.attributes.len(),
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 65 / 100);
    }

    fn measure_focus_resolution(metadata: &UiTemplateNodeMetadata, optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = false;
        for _ in 0..RESOLUTIONS_PER_SAMPLE {
            if optimized {
                let focused = metadata_focused(black_box(Some(metadata)));
                checksum ^= focused;
            } else {
                let focused = bool_attribute(black_box(Some(metadata)), "focused").unwrap_or(false);
                let fallback =
                    bool_attribute(black_box(Some(metadata)), "focused").unwrap_or(false);
                checksum ^= focused | fallback;
            }
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
