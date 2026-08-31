use std::collections::BTreeMap;

use crate::text::TextDocumentKey;
use crate::ui::surface::{
    UiArrangedVisibilityIndex, UiSurfaceControlIndex, arranged_node_indexed,
    component_state::UiSurfaceComponentStateStore,
};
use crate::ui::text::{
    UiTextLayoutRequest, UiTextMeasureCache, UiTextShapePrewarmRequest, UiTextViewport,
};
use zircon_runtime_interface::ui::surface::UiArrangedTree;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiPoint,
    tree::{UiTemplateNodeMetadata, UiTree},
};

use super::super::buttons::button_suppresses_owner_text;
use super::super::chrome::chrome_suppresses_owner_text;
use super::super::collection_rows::collection_row_suppresses_owner_text;
use super::super::command_palette::command_palette_suppresses_owner_text;
use super::super::dialog::dialog_suppresses_owner_text;
use super::super::divider::divider_suppresses_owner_text;
use super::super::drag_overlay::drag_overlay_suppresses_owner_text;
use super::super::dropdowns::dropdown_suppresses_owner_text;
use super::super::feedback::feedback_suppresses_owner_text;
use super::super::node_visual_data::UiNodeVisualData;
use super::super::notification_center::notification_center_suppresses_owner_text;
use super::super::popup_menu::popup_menu_may_emit_text;
use super::super::popup_options::popup_option_may_emit_text;
use super::super::progress::progress_suppresses_owner_text;
use super::super::segmented_controls::segmented_control_suppresses_owner_text;
use super::super::selection_controls::selection_control_suppresses_owner_text;
use super::super::skeleton::skeleton_suppresses_owner_text;
use super::super::sliders::slider_suppresses_owner_text;
use super::super::text_fields::text_field_suppresses_owner_text;
use super::popup_anchor::{popup_runtime_anchor_is_open, resolve_popup_anchor_frame};

pub(super) struct OwnerTextPrewarmCollection {
    pub(super) requests: Vec<UiTextShapePrewarmRequest>,
    pub(super) can_overlap_render_commands: bool,
}

pub(super) fn collect_owner_text_prewarm_requests(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    node_indices: &BTreeMap<UiNodeId, usize>,
    arranged_visibility: &UiArrangedVisibilityIndex,
    component_states: Option<&UiSurfaceComponentStateStore>,
    text_measure_cache: &mut UiTextMeasureCache,
    control_index: Option<&UiSurfaceControlIndex>,
    popup_anchor_points: Option<&BTreeMap<UiNodeId, UiPoint>>,
) -> OwnerTextPrewarmCollection {
    let mut collection = OwnerTextPrewarmCollection {
        requests: Vec::new(),
        can_overlap_render_commands: true,
    };
    for node_id in arranged_tree.draw_order.iter().copied() {
        let Some(node) = tree.nodes.get(&node_id) else {
            continue;
        };
        if arranged_visibility.is_render_visible(node_id)
            && component_text_requires_shared_cache(node.template_metadata.as_ref())
        {
            collection.can_overlap_render_commands = false;
        }
        let Ok(arranged_node) = arranged_node_indexed(arranged_tree, node_indices, node_id) else {
            continue;
        };
        if !arranged_visibility.is_render_visible(node_id)
            || owner_text_is_suppressed(node.template_metadata.as_ref())
            || (popup_runtime_anchor_is_open(node.template_metadata.as_ref())
                && resolve_popup_anchor_frame(
                    tree,
                    arranged_tree,
                    node_indices,
                    arranged_visibility,
                    node_id,
                    node.template_metadata.as_ref(),
                    arranged_node.frame,
                    control_index,
                    popup_anchor_points,
                )
                .is_none())
        {
            continue;
        }
        let component_state = component_states.and_then(|states| states.get(node_id));
        let visual = UiNodeVisualData::resolve(
            node.template_metadata.as_ref(),
            &node.state_flags,
            component_state,
        );
        let Some(text) = visual.text.as_deref().filter(|text| !text.is_empty()) else {
            continue;
        };
        if !arranged_node.frame.width.is_finite()
            || !arranged_node.frame.height.is_finite()
            || arranged_node.frame.width <= 0.0
            || arranged_node.frame.height <= 0.0
        {
            continue;
        }

        let document_key = node
            .layout_cache
            .retained_text_layout_revision()
            .map(|revision| TextDocumentKey::new(node_id.0, revision));
        let viewport = visual.editable.is_none().then(|| {
            UiTextViewport::from_document_and_clip(arranged_node.frame, arranged_node.clip_frame)
        });
        let mut layout_request = UiTextLayoutRequest::new(
            text,
            &visual.style,
            arranged_node.frame,
            Some(arranged_node.clip_frame),
        );
        if let Some(document_key) = document_key {
            layout_request = layout_request.with_document_key(document_key);
        }
        if let Some(viewport) = viewport.flatten() {
            layout_request = layout_request.with_viewport(viewport);
        }
        if text_measure_cache.viewport_selects_partial_plain_text(&layout_request) {
            continue;
        }
        if let Some(request) = text_measure_cache.shape_prewarm_request(text, visual.style.clone())
        {
            collection.requests.push(request);
        }
    }
    collection
}

fn component_text_requires_shared_cache(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    owner_text_is_suppressed(metadata)
        || popup_menu_may_emit_text(metadata)
        || popup_option_may_emit_text(metadata)
}

pub(super) fn owner_text_is_suppressed(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    selection_control_suppresses_owner_text(metadata)
        || slider_suppresses_owner_text(metadata)
        || dropdown_suppresses_owner_text(metadata)
        || text_field_suppresses_owner_text(metadata)
        || button_suppresses_owner_text(metadata)
        || segmented_control_suppresses_owner_text(metadata)
        || progress_suppresses_owner_text(metadata)
        || divider_suppresses_owner_text(metadata)
        || skeleton_suppresses_owner_text(metadata)
        || collection_row_suppresses_owner_text(metadata)
        || feedback_suppresses_owner_text(metadata)
        || dialog_suppresses_owner_text(metadata)
        || command_palette_suppresses_owner_text(metadata)
        || notification_center_suppresses_owner_text(metadata)
        || drag_overlay_suppresses_owner_text(metadata)
        || chrome_suppresses_owner_text(metadata)
}
