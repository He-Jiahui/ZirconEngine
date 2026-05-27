use zircon_runtime_interface::ui::{
    accessibility::UiA11yTextSelection, component::UiValue, dispatch::UiInputDispatchResult,
    event_ui::UiNodeId, surface::UiTextRange, tree::UiTemplateNodeMetadata,
};

use crate::ui::surface::UiSurface;

use self::metadata::{
    clear_text_input_composition_metadata, mutate_text_input_accessibility_metadata,
};

mod metadata;

pub(super) fn text_input_is_read_only(surface: &UiSurface, target: UiNodeId) -> bool {
    surface
        .tree
        .nodes
        .get(&target)
        .and_then(|node| node.template_metadata.as_ref())
        .is_some_and(|metadata| {
            bool_metadata_attribute(metadata, "read_only")
                .or_else(|| bool_metadata_attribute(metadata, "input_read_only"))
                .unwrap_or(false)
        })
}

fn bool_metadata_attribute(metadata: &UiTemplateNodeMetadata, property: &str) -> Option<bool> {
    metadata
        .attributes
        .get(property)
        .and_then(toml::Value::as_bool)
}

pub(super) fn sync_text_input_set_value_edit_metadata(
    surface: &mut UiSurface,
    target: UiNodeId,
    value: &UiValue,
    result: &mut UiInputDispatchResult,
) {
    // A11y SetValue replaces the whole edit buffer; collapse selection to the new end
    // and clear active composition through the same retained properties consumed by
    // render, input, and snapshots.
    let UiValue::String(text) = value else {
        return;
    };
    sync_text_input_edit_metadata(surface, target, text.len(), result);
}

pub(super) fn sync_text_input_edit_metadata(
    surface: &mut UiSurface,
    target: UiNodeId,
    collapse_offset: usize,
    result: &mut UiInputDispatchResult,
) {
    sync_text_input_selection_metadata(
        surface,
        target,
        collapse_offset,
        collapse_offset,
        collapse_offset,
        result,
    );
}

pub(super) fn sync_text_input_selection_metadata(
    surface: &mut UiSurface,
    target: UiNodeId,
    caret: usize,
    anchor: usize,
    focus: usize,
    result: &mut UiInputDispatchResult,
) {
    for (property, offset) in [
        ("caret_offset", caret),
        ("selection_anchor", anchor),
        ("selection_focus", focus),
    ] {
        mutate_text_input_accessibility_metadata(
            surface,
            target,
            property,
            UiValue::Int(i64::try_from(offset).unwrap_or(i64::MAX)),
            "accessibility_text_selection",
            result,
        );
    }
    clear_text_input_composition_metadata(surface, target, caret, result);
}

pub(super) fn text_selection_range(
    text: &str,
    selection: Option<&UiA11yTextSelection>,
) -> UiTextRange {
    let selection = selection
        .cloned()
        .unwrap_or_else(|| UiA11yTextSelection::collapsed(text.len()));
    let anchor = clamp_text_boundary(text, selection.anchor);
    let focus = clamp_text_boundary(text, selection.focus);
    UiTextRange {
        start: anchor.min(focus),
        end: anchor.max(focus),
    }
}

pub(super) fn clamp_text_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}
