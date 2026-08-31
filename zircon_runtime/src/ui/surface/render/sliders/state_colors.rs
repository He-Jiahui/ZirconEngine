use zircon_runtime_interface::ui::{style::UiRgbaColor, tree::UiTemplateNodeMetadata};

use super::{SliderRenderState, SliderVisual, string_attribute};

pub(super) fn track_color(state: &SliderRenderState, visual: &SliderVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.track_disabled
    } else {
        visual.track
    }
}

pub(super) fn fill_color(
    metadata: &UiTemplateNodeMetadata,
    state: &SliderRenderState,
    visual: &SliderVisual,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else if string_attribute(metadata, "validation_level").is_some_and(|level| level == "warning")
    {
        visual.warning
    } else if string_attribute(metadata, "validation_level")
        .is_some_and(|level| matches!(level, "error" | "danger"))
    {
        visual.error
    } else {
        visual.fill
    }
}

pub(super) fn thumb_color(state: &SliderRenderState, visual: &SliderVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else {
        visual.thumb
    }
}

pub(super) fn thumb_outline_color(state: &SliderRenderState, visual: &SliderVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.border_disabled
    } else {
        visual.thumb_outline
    }
}

pub(super) fn halo_color(state: &SliderRenderState, visual: &SliderVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else {
        visual.halo
    }
}

pub(super) fn label_color(state: &SliderRenderState, visual: &SliderVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else {
        visual.label_text
    }
}

pub(super) fn text_color(state: &SliderRenderState, visual: &SliderVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else {
        visual.text
    }
}

pub(super) fn value_surface_color(state: &SliderRenderState, visual: &SliderVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.value_surface_disabled
    } else {
        visual.value_surface
    }
}

pub(super) fn value_border(
    state: &SliderRenderState,
    visual: &SliderVisual,
    metadata: &UiTemplateNodeMetadata,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.border_disabled
    } else if state.pressed() {
        fill_color(metadata, state, visual)
    } else {
        visual.value_border
    }
}

pub(super) fn range_value_border(state: &SliderRenderState, visual: &SliderVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.border_disabled
    } else {
        visual.value_border
    }
}

pub(super) fn tick_color(state: &SliderRenderState, visual: &SliderVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.border_disabled
    } else {
        visual.tick
    }
}
