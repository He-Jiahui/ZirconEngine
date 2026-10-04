use zircon_runtime::ui::surface::layout_text;
use zircon_runtime_interface::ui::{
    layout::UiFrame, surface::UiRenderExtract, window::UiWindowMetrics,
};

pub(super) fn physical_extract(
    logical: &UiRenderExtract,
    metrics: UiWindowMetrics,
) -> Result<UiRenderExtract, String> {
    let scale = metrics.scale_factor as f32;
    if !scale.is_finite() || scale <= 0.0 {
        return Err("DPI must be positive and finite".into());
    }
    let mut extract = logical.clone();
    if scale == 1.0 {
        return Ok(extract);
    }
    // WGPU consumes physical command coordinates. The surface retains logical
    // layout and its raster density; this submission already includes that density.
    extract.raster_scale = (logical.normalized_raster_scale() / scale).max(1.0);
    for command in &mut extract.list.commands {
        command.frame = scaled_frame(command.frame, scale);
        command.clip_frame = command.clip_frame.map(|frame| scaled_frame(frame, scale));
        let style = &mut command.style;
        style.border_width *= scale;
        style.corner_radius *= scale;
        style.font_size *= scale;
        style.line_height *= scale;
        if let Some(outline) = &mut style.text_effects.outline {
            outline.width_px *= scale;
        }
        if let Some(shadow) = &mut style.text_effects.shadow {
            shadow.offset_x_px *= scale;
            shadow.offset_y_px *= scale;
        }
        if let Some(glow) = &mut style.text_effects.glow {
            glow.radius_px *= scale;
        }
        if let Some(previous) = command.text_layout.take() {
            let text = command
                .text
                .as_deref()
                .ok_or("text layout has no source text")?;
            if previous
                .editable
                .as_ref()
                .is_some_and(|editable| editable.composition.is_some())
            {
                return Err(
                    "native DPI adapter does not yet project active IME composition".into(),
                );
            }
            // Opaque font-generation artifacts cannot be scaled in place. Re-enter
            // the actual text owner at physical size to publish matching glyphs.
            let mut layout = layout_text(text, style, command.frame, command.clip_frame);
            layout.editable = previous.editable;
            command.text_layout = Some(layout);
        }
    }
    Ok(extract)
}

fn scaled_frame(frame: UiFrame, scale: f32) -> UiFrame {
    UiFrame::new(
        frame.x * scale,
        frame.y * scale,
        frame.width * scale,
        frame.height * scale,
    )
}
