use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use crate::ui::surface::{
    arranged_node_indexed, arranged_node_indices, build_arranged_tree,
    component_state::UiSurfaceComponentStateStore, is_arranged_render_visible_indexed,
    UiSurfaceControlIndex,
};
use zircon_runtime_interface::ui::surface::UiArrangedTree;
use zircon_runtime_interface::ui::surface::{UiRenderCommand, UiRenderExtract, UiRenderList};
use zircon_runtime_interface::ui::tree::UiTree;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    tree::UiTemplateNodeMetadata,
    widget::{UiPopupAnchor, UiWidgetBehavior},
};

use super::buttons::{
    button_render_commands, button_suppresses_owner_image, button_suppresses_owner_text,
};
use super::chrome::{
    chrome_render_commands, chrome_suppresses_owner_image, chrome_suppresses_owner_surface,
    chrome_suppresses_owner_text,
};
use super::collection_rows::{
    collection_row_render_commands, collection_row_suppresses_owner_image,
    collection_row_suppresses_owner_surface, collection_row_suppresses_owner_text,
};
use super::command_palette::{
    command_palette_render_commands, command_palette_suppresses_owner_image,
    command_palette_suppresses_owner_surface, command_palette_suppresses_owner_text,
};
use super::dialog::{
    dialog_render_commands, dialog_suppresses_owner_image, dialog_suppresses_owner_surface,
    dialog_suppresses_owner_text,
};
use super::divider::{
    divider_render_commands, divider_suppresses_owner_image, divider_suppresses_owner_surface,
    divider_suppresses_owner_text,
};
use super::drag_overlay::{
    drag_overlay_render_commands, drag_overlay_suppresses_owner_image,
    drag_overlay_suppresses_owner_surface, drag_overlay_suppresses_owner_text,
};
use super::dropdowns::{dropdown_render_commands, dropdown_suppresses_owner_text};
use super::feedback::{
    feedback_render_commands, feedback_suppresses_owner_image, feedback_suppresses_owner_surface,
    feedback_suppresses_owner_text,
};
use super::node_visual_data::UiNodeVisualData;
use super::notification_center::{
    notification_center_render_commands, notification_center_suppresses_owner_image,
    notification_center_suppresses_owner_surface, notification_center_suppresses_owner_text,
};
use super::popup_menu::{popup_menu_may_emit_text, popup_menu_render_commands};
use super::popup_options::{popup_option_may_emit_text, popup_option_render_commands};
use super::progress::{
    progress_render_commands, progress_suppresses_owner_image, progress_suppresses_owner_surface,
    progress_suppresses_owner_text,
};
use super::resolve::resolve_command_kind;
use super::segmented_controls::{
    segmented_control_render_commands, segmented_control_suppresses_owner_text,
};
use super::selection_controls::{
    selection_control_render_commands, selection_control_suppresses_owner_text,
};
use super::skeleton::{
    skeleton_render_commands, skeleton_suppresses_owner_image, skeleton_suppresses_owner_surface,
    skeleton_suppresses_owner_text,
};
use super::sliders::{slider_render_commands, slider_suppresses_owner_text};
use super::text_fields::{text_field_render_commands, text_field_suppresses_owner_text};
use super::text_prewarm::{
    prewarm_owner_text_requests, prewarm_render_command_text_after_owner_overlap,
    resolve_missing_render_command_text_layouts, ui_text_shape_prewarm_pool,
    PendingOwnerTextLayouts, UI_TEXT_OWNER_PREWARM_OVERLAP_MIN_REQUESTS,
};
#[cfg(feature = "profiling")]
use super::text_prewarm::{
    record_compiled_rich_text_cache_profile, record_text_extract_profile,
    TextFontHandleFrameProfile,
};
use crate::text::TextDocumentKey;
use crate::ui::text::{
    prepare_render_command_text_artifacts, resolve_text_layout, UiTextLayoutRequest,
    UiTextLayoutResolution, UiTextMeasureCache, UiTextShapePrewarmRequest, UiTextViewport,
};

pub fn extract_ui_render_tree(tree: &UiTree) -> UiRenderExtract {
    let arranged_tree = build_arranged_tree(tree);
    extract_ui_render_tree_from_arranged(tree, &arranged_tree)
}

pub fn extract_ui_render_tree_from_arranged(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
) -> UiRenderExtract {
    extract_ui_render_tree_from_arranged_with_component_states(tree, arranged_tree, None)
}

pub(crate) fn extract_ui_render_tree_from_arranged_with_component_states(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    component_states: Option<&UiSurfaceComponentStateStore>,
) -> UiRenderExtract {
    extract_ui_render_tree_from_arranged_with_component_states_and_text_measure_cache(
        tree,
        arranged_tree,
        component_states,
        None,
    )
}

pub(crate) fn extract_ui_render_tree_from_arranged_with_component_states_and_text_measure_cache(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    component_states: Option<&UiSurfaceComponentStateStore>,
    text_measure_cache: Option<&mut UiTextMeasureCache>,
) -> UiRenderExtract {
    let node_indices = arranged_node_indices(arranged_tree);
    extract_ui_render_tree_from_arranged_indexed_with_component_states_and_text_measure_cache(
        tree,
        arranged_tree,
        &node_indices,
        component_states,
        text_measure_cache,
    )
}

pub(crate) fn extract_ui_render_tree_from_arranged_indexed_with_component_states_and_text_measure_cache(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    node_indices: &BTreeMap<UiNodeId, usize>,
    component_states: Option<&UiSurfaceComponentStateStore>,
    text_measure_cache: Option<&mut UiTextMeasureCache>,
) -> UiRenderExtract {
    extract_ui_render_tree_from_arranged_indexed_with_component_states_and_text_measure_cache_and_control_index(
        tree,
        arranged_tree,
        node_indices,
        component_states,
        text_measure_cache,
        None,
    )
}

/// Surface-owned extraction passes the incremental control index so control-anchored
/// popups resolve their live trigger frame without an additional whole-tree scan.
pub(crate) fn extract_ui_render_tree_from_arranged_indexed_with_component_states_and_text_measure_cache_and_control_index(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    node_indices: &BTreeMap<UiNodeId, usize>,
    component_states: Option<&UiSurfaceComponentStateStore>,
    mut text_measure_cache: Option<&mut UiTextMeasureCache>,
    control_index: Option<&UiSurfaceControlIndex>,
) -> UiRenderExtract {
    #[cfg(feature = "profiling")]
    let font_handle_profile = TextFontHandleFrameProfile::begin();
    // These caller-thread scans precede command building, so they need their own p95 attribution.
    let owner_prewarm_candidates = {
        crate::profile_scope!(
            "runtime",
            "ui_text.extract",
            "owner_prewarm_request_collection"
        );
        text_measure_cache.as_deref_mut().map(|cache| {
            collect_owner_text_prewarm_requests(
                tree,
                arranged_tree,
                node_indices,
                component_states,
                cache,
                control_index,
            )
        })
    };
    let owner_text_prewarm_requests = {
        crate::profile_scope!(
            "runtime",
            "ui_text.extract",
            "owner_prewarm_overlap_admission"
        );
        owner_prewarm_candidates.filter(|requests| {
            requests.len() >= UI_TEXT_OWNER_PREWARM_OVERLAP_MIN_REQUESTS
                && render_command_build_can_overlap_owner_prewarm(tree, arranged_tree, node_indices)
        })
    };
    #[cfg(feature = "profiling")]
    let profile_frame_context =
        crate::core::runtime::diagnostics::profiling::ProfileFrameContext::capture();
    let collect_render_commands = |mut command_text_measure_cache: Option<
        &mut UiTextMeasureCache,
    >| {
        crate::profile_scope!("runtime", "ui_text.extract", "render_command_collection");
        let mut commands = Vec::new();
        let mut pending_owner_text_layouts = PendingOwnerTextLayouts::default();
        for node_id in arranged_tree.draw_order.iter().copied() {
            let Some(node) = tree.nodes.get(&node_id) else {
                continue;
            };
            let Ok(arranged_node) = arranged_node_indexed(arranged_tree, node_indices, node_id)
            else {
                continue;
            };
            if !is_arranged_render_visible_indexed(arranged_tree, node_indices, node_id)
                .unwrap_or(false)
            {
                continue;
            }
            let popup_anchor_frame = resolve_popup_anchor_frame(
                tree,
                arranged_tree,
                node_indices,
                node.template_metadata.as_ref(),
                arranged_node.frame,
                control_index,
            );
            if popup_control_anchor_is_open(node.template_metadata.as_ref())
                && popup_anchor_frame.is_none()
            {
                continue;
            }
            let component_state = component_states.and_then(|states| states.get(node_id));
            let visual = UiNodeVisualData::resolve(
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
            );
            let owner_text = if owner_text_is_suppressed(node.template_metadata.as_ref()) {
                None
            } else {
                visual.text.clone()
            };
            let owner_image = if button_suppresses_owner_image(node.template_metadata.as_ref())
                || collection_row_suppresses_owner_image(node.template_metadata.as_ref())
                || progress_suppresses_owner_image(node.template_metadata.as_ref())
                || divider_suppresses_owner_image(node.template_metadata.as_ref())
                || skeleton_suppresses_owner_image(node.template_metadata.as_ref())
                || feedback_suppresses_owner_image(node.template_metadata.as_ref())
                || dialog_suppresses_owner_image(node.template_metadata.as_ref())
                || command_palette_suppresses_owner_image(node.template_metadata.as_ref())
                || notification_center_suppresses_owner_image(node.template_metadata.as_ref())
                || drag_overlay_suppresses_owner_image(node.template_metadata.as_ref())
                || chrome_suppresses_owner_image(node.template_metadata.as_ref())
            {
                None
            } else {
                visual.image.clone()
            };
            let owner_style =
                if collection_row_suppresses_owner_surface(node.template_metadata.as_ref())
                    || progress_suppresses_owner_surface(node.template_metadata.as_ref())
                    || divider_suppresses_owner_surface(node.template_metadata.as_ref())
                    || skeleton_suppresses_owner_surface(node.template_metadata.as_ref())
                    || feedback_suppresses_owner_surface(node.template_metadata.as_ref())
                    || dialog_suppresses_owner_surface(node.template_metadata.as_ref())
                    || command_palette_suppresses_owner_surface(node.template_metadata.as_ref())
                    || notification_center_suppresses_owner_surface(node.template_metadata.as_ref())
                    || drag_overlay_suppresses_owner_surface(node.template_metadata.as_ref())
                    || chrome_suppresses_owner_surface(node.template_metadata.as_ref())
                {
                    let mut style = visual.style.clone();
                    style.background_color = None;
                    style.border_color = None;
                    style.border_width = 0.0;
                    style.corner_radius = 0.0;
                    style
                } else {
                    visual.style.clone()
                };

            let owner_text_layout_metadata = owner_text.as_ref().map(|_| {
                let viewport = visual
                    .editable
                    .is_none()
                    .then(|| {
                        UiTextViewport::from_document_and_clip(
                            arranged_node.frame,
                            arranged_node.clip_frame,
                        )
                    })
                    .flatten();
                (
                    TextDocumentKey::new(node_id.0, node.layout_cache.text_layout_revision),
                    viewport,
                    visual.editable.clone(),
                )
            });
            let command = UiRenderCommand {
                node_id,
                kind: resolve_command_kind(&owner_style, owner_text.as_ref(), owner_image.as_ref()),
                frame: arranged_node.frame,
                clip_frame: Some(arranged_node.clip_frame),
                z_index: arranged_node.z_index,
                style: owner_style,
                text_layout: None,
                text: owner_text,
                image: owner_image,
                opacity: visual.opacity,
            };
            let owner_command_index = commands.len();
            commands.push(command);
            if let Some((document_key, viewport, editable)) = owner_text_layout_metadata {
                pending_owner_text_layouts.push(
                    owner_command_index,
                    document_key,
                    viewport,
                    editable,
                );
            }
            commands.extend(button_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(selection_control_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(segmented_control_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(slider_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(dropdown_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(text_field_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
                &visual.style,
                visual.text.as_deref(),
                visual.editable.as_ref(),
                command_text_measure_cache.as_deref_mut(),
            ));
            commands.extend(collection_row_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(feedback_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(progress_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(divider_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(skeleton_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(dialog_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(command_palette_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(notification_center_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(drag_overlay_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(chrome_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(popup_menu_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                arranged_node.frame,
                popup_anchor_frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
            commands.extend(popup_option_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                arranged_node.frame,
                popup_anchor_frame,
                Some(arranged_node.clip_frame),
                arranged_node.z_index,
                visual.opacity,
            ));
        }
        (commands, pending_owner_text_layouts)
    };
    let (
        (mut commands, pending_owner_text_layouts),
        owner_text_already_prewarmed,
        owner_overlap_request_count,
        owner_overlap_join_wait_nanos,
    ) = match owner_text_prewarm_requests {
        Some(requests) => match text_measure_cache.as_deref_mut() {
            Some(cache) => {
                let request_count = requests.len();
                let pool = ui_text_shape_prewarm_pool();
                let mut collected = (Vec::new(), PendingOwnerTextLayouts::default());
                let mut command_build_finished_at = None;
                pool.in_place_scope(|scope| {
                    scope.spawn(move |_| {
                        #[cfg(feature = "profiling")]
                        let _profile_frame_context = profile_frame_context.attach();
                        prewarm_owner_text_requests(&requests, cache);
                    });
                    collected = collect_render_commands(None);
                    command_build_finished_at = Some(Instant::now());
                });
                let join_wait_nanos = command_build_finished_at
                    .map(|finished_at| {
                        finished_at.elapsed().as_nanos().min(u64::MAX as u128) as u64
                    })
                    .unwrap_or_default();
                (collected, true, request_count, join_wait_nanos)
            }
            None => (collect_render_commands(None), false, 0, 0),
        },
        None => (
            collect_render_commands(text_measure_cache.as_deref_mut()),
            false,
            0,
            0,
        ),
    };
    #[cfg(feature = "profiling")]
    record_text_extract_profile(commands.len(), pending_owner_text_layouts.len());
    #[cfg(feature = "profiling")]
    {
        crate::profile_counter!(
            "runtime",
            "ui_text.prewarm.owner_overlap_requests",
            owner_overlap_request_count
        );
        crate::profile_counter!(
            "runtime",
            "ui_text.prewarm.owner_overlap_joins",
            owner_text_already_prewarmed as usize
        );
        crate::profile_counter!(
            "runtime",
            "ui_text.prewarm.owner_overlap_join_wait_nanos",
            owner_overlap_join_wait_nanos
        );
    }
    #[cfg(not(feature = "profiling"))]
    let _ = (owner_overlap_request_count, owner_overlap_join_wait_nanos);

    if let Some(cache) = text_measure_cache.as_deref_mut() {
        prewarm_render_command_text_after_owner_overlap(
            &commands,
            &pending_owner_text_layouts,
            cache,
            owner_text_already_prewarmed,
        );
    }
    resolve_missing_render_command_text_layouts(
        &mut commands,
        &pending_owner_text_layouts,
        text_measure_cache.as_deref_mut(),
    );
    prepare_render_command_text_artifacts(&mut commands);
    #[cfg(feature = "profiling")]
    {
        if let Some(cache) = text_measure_cache.as_deref_mut() {
            record_compiled_rich_text_cache_profile(cache.sample_compiled_rich_text_cache());
        }
        font_handle_profile.finish();
    }

    UiRenderExtract {
        tree_id: tree.tree_id.clone(),
        list: UiRenderList { commands },
        raster_scale: 1.0,
    }
}

fn resolve_popup_anchor_frame(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    node_indices: &BTreeMap<UiNodeId, usize>,
    metadata: Option<&UiTemplateNodeMetadata>,
    owner_frame: UiFrame,
    control_index: Option<&UiSurfaceControlIndex>,
) -> Option<UiFrame> {
    let Some(metadata) = metadata else {
        return Some(owner_frame);
    };
    let UiPopupAnchor::Control { control_id } = &metadata.widget.popup_anchor else {
        return Some(owner_frame);
    };
    let trigger_node_id = match control_index {
        Some(control_index) => control_index.unique_node_id_for_surface(tree, control_id),
        None => unique_control_node_id(tree, control_id),
    }?;
    if !is_arranged_render_visible_indexed(arranged_tree, node_indices, trigger_node_id)
        .unwrap_or(false)
    {
        return None;
    }
    let trigger = tree.nodes.get(&trigger_node_id)?;
    if !trigger.state_flags.enabled {
        return None;
    }
    let trigger_frame = arranged_node_indexed(arranged_tree, node_indices, trigger_node_id)
        .ok()?
        .frame;
    (trigger_frame.x.is_finite()
        && trigger_frame.y.is_finite()
        && trigger_frame.width.is_finite()
        && trigger_frame.height.is_finite()
        && trigger_frame.width > 0.0
        && trigger_frame.height > 0.0)
        .then_some(trigger_frame)
}

fn unique_control_node_id(tree: &UiTree, control_id: &str) -> Option<UiNodeId> {
    let mut matches = tree.nodes.iter().filter_map(|(node_id, node)| {
        (node
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            == Some(control_id))
        .then_some(*node_id)
    });
    let node_id = matches.next()?;
    matches.next().is_none().then_some(node_id)
}

fn popup_control_anchor_is_open(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    let Some(metadata) = metadata else {
        return false;
    };
    matches!(&metadata.widget.popup_anchor, UiPopupAnchor::Control { .. })
        && (metadata.widget.resolved_behavior(&metadata.component) == UiWidgetBehavior::Popup
            || matches!(
                metadata.component.as_str(),
                "Dialog" | "ConfirmDialog" | "Modal" | "Popover"
            ))
        && ["popup_open", "open"]
            .iter()
            .any(|key| metadata.attributes.get(*key).and_then(toml::Value::as_bool) == Some(true))
}

fn collect_owner_text_prewarm_requests(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    node_indices: &BTreeMap<UiNodeId, usize>,
    component_states: Option<&UiSurfaceComponentStateStore>,
    text_measure_cache: &mut UiTextMeasureCache,
    control_index: Option<&UiSurfaceControlIndex>,
) -> Vec<UiTextShapePrewarmRequest> {
    let mut requests = Vec::new();
    for node_id in arranged_tree.draw_order.iter().copied() {
        let Some(node) = tree.nodes.get(&node_id) else {
            continue;
        };
        let Ok(arranged_node) = arranged_node_indexed(arranged_tree, node_indices, node_id) else {
            continue;
        };
        if !is_arranged_render_visible_indexed(arranged_tree, node_indices, node_id)
            .unwrap_or(false)
            || owner_text_is_suppressed(node.template_metadata.as_ref())
            || (popup_control_anchor_is_open(node.template_metadata.as_ref())
                && resolve_popup_anchor_frame(
                    tree,
                    arranged_tree,
                    node_indices,
                    node.template_metadata.as_ref(),
                    arranged_node.frame,
                    control_index,
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
        let Some(text) = visual
            .text
            .as_deref()
            .filter(|text| !text.trim().is_empty())
        else {
            continue;
        };
        if !arranged_node.frame.width.is_finite()
            || !arranged_node.frame.height.is_finite()
            || arranged_node.frame.width <= 0.0
            || arranged_node.frame.height <= 0.0
        {
            continue;
        }

        let document_key = TextDocumentKey::new(node_id.0, node.layout_cache.text_layout_revision);
        let viewport = visual.editable.is_none().then(|| {
            UiTextViewport::from_document_and_clip(arranged_node.frame, arranged_node.clip_frame)
        });
        let mut layout_request = UiTextLayoutRequest::new(
            text,
            &visual.style,
            arranged_node.frame,
            Some(arranged_node.clip_frame),
        )
        .with_document_key(document_key);
        if let Some(viewport) = viewport.flatten() {
            layout_request = layout_request.with_viewport(viewport);
        }
        if text_measure_cache.viewport_selects_partial_plain_text(&layout_request) {
            continue;
        }
        if let Some(request) =
            UiTextShapePrewarmRequest::from_layout_source(text, visual.style.clone())
        {
            requests.push(request);
        }
    }
    requests
}

fn render_command_build_can_overlap_owner_prewarm(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    node_indices: &BTreeMap<UiNodeId, usize>,
) -> bool {
    arranged_tree.draw_order.iter().copied().all(|node_id| {
        let Some(node) = tree.nodes.get(&node_id) else {
            return true;
        };
        !is_arranged_render_visible_indexed(arranged_tree, node_indices, node_id).unwrap_or(false)
            || !component_text_requires_shared_cache(node.template_metadata.as_ref())
    })
}

fn component_text_requires_shared_cache(
    metadata: Option<&zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata>,
) -> bool {
    owner_text_is_suppressed(metadata)
        || popup_menu_may_emit_text(metadata)
        || popup_option_may_emit_text(metadata)
}

fn owner_text_is_suppressed(
    metadata: Option<&zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata>,
) -> bool {
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

pub(crate) fn extract_ui_render_commands_for_nodes_with_component_states_and_text_measure_cache(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    changed_node_ids: &BTreeSet<UiNodeId>,
    component_states: Option<&UiSurfaceComponentStateStore>,
    text_measure_cache: Option<&mut UiTextMeasureCache>,
) -> Result<UiRenderExtract, ()> {
    let mut partial_nodes = Vec::new();
    let mut included = BTreeSet::new();
    for node_id in changed_node_ids {
        let mut current = Some(*node_id);
        while let Some(current_id) = current {
            let Some(index) = arranged_node_indices.get(&current_id).copied() else {
                return Err(());
            };
            let Some(node) = arranged_tree.nodes.get(index) else {
                return Err(());
            };
            if node.node_id != current_id || tree.node(current_id).is_none() {
                return Err(());
            }
            current = node.parent;
            if included.insert(current_id) {
                partial_nodes.push(node.clone());
            }
        }
    }
    let partial_tree = UiArrangedTree {
        tree_id: arranged_tree.tree_id.clone(),
        roots: Vec::new(),
        nodes: partial_nodes,
        draw_order: changed_node_ids.iter().copied().collect(),
        canvas_layers: Vec::new(),
    };
    Ok(
        extract_ui_render_tree_from_arranged_with_component_states_and_text_measure_cache(
            tree,
            &partial_tree,
            component_states,
            text_measure_cache,
        ),
    )
}

pub(super) fn resolve_text_layout_with_cache(
    request: &UiTextLayoutRequest<'_>,
    text_measure_cache: Option<&mut UiTextMeasureCache>,
) -> UiTextLayoutResolution {
    match text_measure_cache {
        Some(cache) => cache.resolve_or_shape(request),
        None => resolve_text_layout(request),
    }
}
