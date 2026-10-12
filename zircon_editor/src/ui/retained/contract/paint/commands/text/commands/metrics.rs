const MIN_TEXT_METRIC_PX: f32 = 1.0;

pub(super) fn resolved_font_size(font_size: f32) -> f32 {
    font_size.max(MIN_TEXT_METRIC_PX)
}

pub(super) fn resolved_line_height(font_size: f32, line_height: f32) -> f32 {
    if crate::ui::retained_host::host_contract::paint_text::active_authored_font_request().is_some()
        && line_height.is_finite()
        && line_height > 0.0
    {
        return line_height;
    }
    line_height.max(font_size).max(MIN_TEXT_METRIC_PX)
}

#[cfg(test)]
#[path = "../../../../../../contract/paint/commands/text/commands/tests/metrics.rs"]
mod tests;

// Runs may already carry a distinct resolved face; preserve it exactly. The
// projection also emits inherited base fields for semantic Strong/Code runs,
// which must retain their existing central face semantics.
pub(super) fn rich_font_request(
    family: Option<&str>,
    weight: u16,
    kind: zircon_runtime_interface::ui::surface::UiTextRunKind,
    preserve_resolved_override: bool,
) -> crate::ui::retained_host::host_contract::paint_text::HostTextFontRequest {
    use crate::ui::retained_host::host_contract::paint_text::{
        active_authored_font_request, font_request_for_face, HostTextFontFace,
        HostTextFontRequestScope,
    };
    use zircon_runtime_interface::ui::surface::UiTextRunKind;
    let base = active_authored_font_request()
        .unwrap_or_else(|| font_request_for_face(HostTextFontFace::Ui));
    let family = family.filter(|family| !family.trim().is_empty());
    let weight = (1..=1000).contains(&weight).then_some(weight);
    let distinct = family.is_some_and(|family| family != base.family)
        || weight.is_some_and(|weight| weight != base.weight);
    let mut request = base;
    if let Some(family) = family {
        request.family = family.to_owned();
    }
    if let Some(weight) = weight {
        request.weight = weight;
    }
    if preserve_resolved_override && distinct {
        return request;
    }
    let face = match kind {
        UiTextRunKind::Strong => HostTextFontFace::UiStrong,
        UiTextRunKind::Code => HostTextFontFace::Mono,
        _ => return request,
    };
    let semantic = {
        let _legacy = HostTextFontRequestScope::enter(None);
        font_request_for_face(face)
    };
    request.face = face;
    if kind == UiTextRunKind::Code {
        request.family = semantic.family;
        request.weight = semantic.weight;
    } else {
        request.weight = request.weight.max(semantic.weight);
    }
    request
}
