use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use crate::ui::surface::{
    UiArrangedVisibilityIndex, UiSurfaceControlIndex, arranged_node_indexed, arranged_node_indices,
    component_state::UiSurfaceComponentStateStore,
};
use zircon_runtime_interface::ui::surface::UiArrangedTree;
use zircon_runtime_interface::ui::surface::{UiRenderCommand, UiRenderExtract, UiRenderList};
use zircon_runtime_interface::ui::tree::UiTree;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiPoint},
};

use super::buttons::{button_render_commands, button_suppresses_owner_image};
use super::chrome::{
    chrome_render_commands, chrome_suppresses_owner_image, chrome_suppresses_owner_surface,
};
use super::collection_rows::{
    collection_row_render_commands, collection_row_suppresses_owner_image,
    collection_row_suppresses_owner_surface,
};
use super::command_palette::{
    command_palette_render_commands, command_palette_suppresses_owner_image,
    command_palette_suppresses_owner_surface,
};
use super::dialog::{
    dialog_render_commands, dialog_suppresses_owner_image, dialog_suppresses_owner_surface,
    dialog_suppresses_owner_text,
};
use super::divider::{
    divider_render_commands, divider_suppresses_owner_image, divider_suppresses_owner_surface,
};
use super::drag_overlay::{
    drag_overlay_render_commands, drag_overlay_suppresses_owner_image,
    drag_overlay_suppresses_owner_surface,
};
use super::dropdowns::dropdown_render_commands;
use super::feedback::{
    feedback_render_commands, feedback_suppresses_owner_image, feedback_suppresses_owner_surface,
};
use super::node_visual_data::UiNodeVisualData;
use super::notification_center::{
    notification_center_render_commands, notification_center_suppresses_owner_image,
    notification_center_suppresses_owner_surface,
};
use super::popup_menu::popup_menu_render_commands;
use super::popup_options::popup_option_render_commands;
use super::progress::{
    progress_render_commands, progress_suppresses_owner_image, progress_suppresses_owner_surface,
};
use super::resolve::resolve_command_kind;
use super::segmented_controls::segmented_control_render_commands;
use super::selection_controls::selection_control_render_commands;
use super::skeleton::{
    skeleton_render_commands, skeleton_suppresses_owner_image, skeleton_suppresses_owner_surface,
};
use super::sliders::slider_render_commands;
use super::text_fields::{text_field_render_commands, text_field_suppresses_owner_text};
use super::text_prewarm::{
    PendingOwnerTextLayouts, UI_TEXT_OWNER_PREWARM_OVERLAP_MIN_REQUESTS,
    prewarm_owner_text_requests, prewarm_render_command_text_after_owner_overlap,
    resolve_missing_render_command_text_layouts, ui_text_shape_prewarm_pool,
};
#[cfg(feature = "profiling")]
use super::text_prewarm::{
    TextFontHandleFrameProfile, record_compiled_rich_text_cache_profile,
    record_text_extract_profile,
};
use crate::text::TextDocumentKey;
use crate::ui::text::{UiTextMeasureCache, UiTextViewport};

mod one_shot;
mod owner_text_prewarm;
mod pixel_snapping;
mod popup_anchor;
mod text_layout_route;

pub use one_shot::{extract_ui_render_tree, extract_ui_render_tree_from_arranged};
use owner_text_prewarm::{collect_owner_text_prewarm_requests, owner_text_is_suppressed};
use pixel_snapping::apply_resolved_pixel_snapping_policies;
use popup_anchor::{popup_runtime_anchor_is_open, resolve_popup_anchor_frame};
pub(crate) use text_layout_route::resolve_text_layout_with_cache;

pub(crate) fn extract_ui_render_tree_from_arranged_with_component_states_and_text_measure_cache(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    component_states: Option<&UiSurfaceComponentStateStore>,
    text_measure_cache: &mut UiTextMeasureCache,
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
    text_measure_cache: &mut UiTextMeasureCache,
) -> UiRenderExtract {
    extract_ui_render_tree_from_arranged_indexed_with_component_states_and_text_measure_cache_and_control_index(
        tree,
        arranged_tree,
        node_indices,
        None,
        component_states,
        text_measure_cache,
        None,
        None,
    )
}

pub(crate) fn extract_ui_render_tree_from_arranged_indexed_with_component_states_and_text_measure_cache_and_control_index(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    node_indices: &BTreeMap<UiNodeId, usize>,
    arranged_visibility: Option<&UiArrangedVisibilityIndex>,
    component_states: Option<&UiSurfaceComponentStateStore>,
    text_measure_cache: &mut UiTextMeasureCache,
    control_index: Option<&UiSurfaceControlIndex>,
    popup_anchor_points: Option<&BTreeMap<UiNodeId, UiPoint>>,
) -> UiRenderExtract {
    let fallback_arranged_visibility;
    let arranged_visibility = if let Some(index) = arranged_visibility {
        index
    } else {
        crate::profile_counter!(
            "runtime",
            "ui.render_extract.visibility_index_fallback_build_count",
            1
        );
        fallback_arranged_visibility =
            UiArrangedVisibilityIndex::from_arranged(arranged_tree, node_indices);
        &fallback_arranged_visibility
    };
    #[cfg(feature = "profiling")]
    let font_handle_profile = TextFontHandleFrameProfile::begin();
    let owner_prewarm_collection = {
        crate::profile_scope!(
            "runtime",
            "ui_text.extract",
            "owner_prewarm_request_collection"
        );
        collect_owner_text_prewarm_requests(
            tree,
            arranged_tree,
            node_indices,
            arranged_visibility,
            component_states,
            text_measure_cache,
            control_index,
            popup_anchor_points,
        )
    };
    let owner_text_prewarm_requests = {
        crate::profile_scope!(
            "runtime",
            "ui_text.extract",
            "owner_prewarm_overlap_admission"
        );
        (owner_prewarm_collection.requests.len() >= UI_TEXT_OWNER_PREWARM_OVERLAP_MIN_REQUESTS
            && owner_prewarm_collection.can_overlap_render_commands)
            .then_some(owner_prewarm_collection.requests)
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
            if !arranged_visibility.is_render_visible(node_id) {
                continue;
            }
            let popup_anchor_frame = resolve_popup_anchor_frame(
                tree,
                arranged_tree,
                node_indices,
                arranged_visibility,
                node_id,
                node.template_metadata.as_ref(),
                arranged_node.frame,
                control_index,
                popup_anchor_points,
            );
            if popup_runtime_anchor_is_open(node.template_metadata.as_ref())
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
                let document_key = node
                    .layout_cache
                    .retained_text_layout_revision()
                    .map(|revision| TextDocumentKey::new(node_id.0, revision));
                (document_key, viewport, visual.editable.clone())
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
            if let Some(text_measure_cache) = command_text_measure_cache.as_deref_mut() {
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
                    text_measure_cache,
                ));
            } else {
                debug_assert!(!text_field_suppresses_owner_text(
                    node.template_metadata.as_ref()
                ));
            }
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
                popup_anchor_frame,
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
            if let Some(text_measure_cache) = command_text_measure_cache.as_deref_mut() {
                commands.extend(dialog_render_commands(
                    node_id,
                    node.template_metadata.as_ref(),
                    &node.state_flags,
                    component_state,
                    arranged_node.frame,
                    popup_anchor_frame,
                    Some(arranged_node.clip_frame),
                    arranged_node.z_index,
                    visual.opacity,
                    text_measure_cache,
                ));
            } else {
                debug_assert!(!dialog_suppresses_owner_text(
                    node.template_metadata.as_ref()
                ));
            }
            commands.extend(command_palette_render_commands(
                node_id,
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
                arranged_node.frame,
                popup_anchor_frame,
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
                popup_anchor_frame,
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
        Some(requests) => {
            let request_count = requests.len();
            let pool = ui_text_shape_prewarm_pool();
            let mut collected = (Vec::new(), PendingOwnerTextLayouts::default());
            let mut command_build_finished_at = None;
            pool.in_place_scope(|scope| {
                let prewarm_cache = &mut *text_measure_cache;
                scope.spawn(move |_| {
                    #[cfg(feature = "profiling")]
                    let _profile_frame_context = profile_frame_context.attach();
                    prewarm_owner_text_requests(&requests, prewarm_cache);
                });
                collected = collect_render_commands(None);
                command_build_finished_at = Some(Instant::now());
            });
            let join_wait_nanos = command_build_finished_at
                .map(|finished_at| finished_at.elapsed().as_nanos().min(u64::MAX as u128) as u64)
                .unwrap_or_default();
            (collected, true, request_count, join_wait_nanos)
        }
        None => (
            collect_render_commands(Some(&mut *text_measure_cache)),
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

    prewarm_render_command_text_after_owner_overlap(
        &commands,
        &pending_owner_text_layouts,
        text_measure_cache,
        owner_text_already_prewarmed,
    );
    resolve_missing_render_command_text_layouts(
        &mut commands,
        &pending_owner_text_layouts,
        &mut *text_measure_cache,
    );
    text_measure_cache.prepare_render_command_text_artifacts(&mut commands);
    apply_resolved_pixel_snapping_policies(tree, &mut commands);
    #[cfg(feature = "profiling")]
    {
        record_compiled_rich_text_cache_profile(
            text_measure_cache.sample_compiled_rich_text_cache(),
        );
        font_handle_profile.finish();
    }

    UiRenderExtract {
        tree_id: tree.tree_id.clone(),
        list: UiRenderList { commands },
        raster_scale: 1.0,
    }
}

pub(crate) fn extract_ui_render_commands_for_nodes_with_component_states_and_text_measure_cache(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    node_indices: &BTreeMap<UiNodeId, usize>,
    arranged_visibility: &UiArrangedVisibilityIndex,
    changed_node_ids: &BTreeSet<UiNodeId>,
    component_states: Option<&UiSurfaceComponentStateStore>,
    text_measure_cache: &mut UiTextMeasureCache,
    control_index: Option<&UiSurfaceControlIndex>,
    popup_anchor_points: Option<&BTreeMap<UiNodeId, UiPoint>>,
) -> Result<UiRenderExtract, ()> {
    let mut partial_nodes = Vec::new();
    let mut included = BTreeSet::new();
    for node_id in changed_node_ids {
        let mut current = Some(*node_id);
        while let Some(current_id) = current {
            let Some(index) = node_indices.get(&current_id).copied() else {
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
        roots: Vec::new().into(),
        nodes: partial_nodes.into(),
        draw_order: changed_node_ids.iter().copied().collect(),
        canvas_layers: Vec::new().into(),
    };
    let partial_indices = arranged_node_indices(&partial_tree);
    Ok(extract_ui_render_tree_from_arranged_indexed_with_component_states_and_text_measure_cache_and_control_index(
        tree,
        &partial_tree,
        &partial_indices,
        Some(arranged_visibility),
        component_states,
        text_measure_cache,
        control_index,
        popup_anchor_points,
    ))
}
