#[cfg(feature = "profiling")]
use std::cell::Cell;
use std::rc::Rc;

use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostPaneInteractionStateData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::paint_geometry::{
    frame_from_template, intersect, translated,
};
use crate::ui::retained_host::host_contract::paint_template_nodes::TemplateNodePaintTransform;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter_batch, UiPerfCounter};
use crate::ui::workbench::asset_content_layout::{
    asset_thumbnail_card_geometry, compact_thumbnail_file_name_to_width,
    ActivityAssetReferenceListKind, ActivityContentNodeIdentity, AssetBrowserPaintItem,
    AssetBrowserSlotBinding, AssetContentPaintMetadata, AssetContentRect,
    AssetContentRowDescriptor, AssetContentSurface, BrowserAssetReferenceListKind,
    BrowserContentNodeIdentity, BrowserThumbnailNodeRole,
};

pub(in crate::ui::retained_host::host_contract::paint_workbench_renderer::docks::pane::template_nodes)
struct ActivityAssetContentProjector
{
    metadata: Rc<AssetContentPaintMetadata>,
    origin_x: f32,
    origin_y: f32,
    content_clip: Option<FrameRect>,
    references_clip: Option<FrameRect>,
    used_by_clip: Option<FrameRect>,
    scroll_px: f32,
    hovered_row_index: i32,
    references_scroll_px: f32,
    references_hovered_index: i32,
    used_by_scroll_px: f32,
    used_by_hovered_index: i32,
    #[cfg(feature = "profiling")]
    descriptor_lookup_count: Cell<usize>,
}

impl ActivityAssetContentProjector {
    pub(in crate::ui::retained_host::host_contract::paint_workbench_renderer::docks::pane::template_nodes) fn new(
        nodes: &ModelRc<TemplatePaneNodeData>,
        origin: &FrameRect,
        interaction: &HostPaneInteractionStateData,
    ) -> Option<Self> {
        let metadata = nodes.metadata_rc::<AssetContentPaintMetadata>()?;
        if metadata.surface() != AssetContentSurface::Activity {
            return None;
        }
        let content_clip = valid_asset_content_auxiliary_clip(metadata.viewport(), origin);
        let references_clip = metadata
            .activity_reference_viewport(ActivityAssetReferenceListKind::References)
            .and_then(|viewport| valid_asset_content_auxiliary_clip(Some(viewport), origin));
        let used_by_clip = metadata
            .activity_reference_viewport(ActivityAssetReferenceListKind::UsedBy)
            .and_then(|viewport| valid_asset_content_auxiliary_clip(Some(viewport), origin));
        if content_clip.is_none() && references_clip.is_none() && used_by_clip.is_none() {
            return None;
        }

        Some(Self {
            metadata,
            origin_x: origin.x,
            origin_y: origin.y,
            content_clip,
            references_clip,
            used_by_clip,
            scroll_px: interaction.activity_asset_content_scroll_px.max(0.0),
            hovered_row_index: interaction.activity_asset_content_hovered_index,
            references_scroll_px: interaction.activity_asset_references_scroll_px.max(0.0),
            references_hovered_index: interaction.activity_asset_references_hovered_index,
            used_by_scroll_px: interaction.activity_asset_used_by_scroll_px.max(0.0),
            used_by_hovered_index: interaction.activity_asset_used_by_hovered_index,
            #[cfg(feature = "profiling")]
            descriptor_lookup_count: Cell::new(0),
        })
    }

    #[inline]
    fn record_descriptor_lookup(&self) {
        #[cfg(feature = "profiling")]
        self.descriptor_lookup_count
            .set(self.descriptor_lookup_count.get().saturating_add(1));
    }
}

#[cfg(feature = "profiling")]
impl Drop for ActivityAssetContentProjector {
    fn drop(&mut self) {
        record_current_ui_perf_counter_batch(|counters| {
            counters.push((
                UiPerfCounter::AssetContentDescriptorLookupCount,
                self.descriptor_lookup_count.get() as f64,
            ));
        });
    }
}

impl TemplateNodePaintTransform for ActivityAssetContentProjector {
    fn row_visit_indices(&self, _row_count: usize, clip: &FrameRect) -> Option<Vec<usize>> {
        Some(self.metadata.visible_activity_node_rows(
            self.scroll_px,
            self.references_scroll_px,
            self.used_by_scroll_px,
            self.origin_x,
            self.origin_y,
            asset_content_rect(clip),
        ))
    }

    fn transform_row(
        &self,
        row: usize,
        mut node: TemplatePaneNodeData,
        clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)> {
        self.record_descriptor_lookup();
        match self.metadata.row_descriptor(row) {
            AssetContentRowDescriptor::ActivityReference {
                list_kind,
                index,
                paints_hover,
            } => {
                let (reference_clip, scroll_px, hovered_index) = match list_kind {
                    ActivityAssetReferenceListKind::References => (
                        self.references_clip.as_ref()?,
                        self.references_scroll_px,
                        self.references_hovered_index,
                    ),
                    ActivityAssetReferenceListKind::UsedBy => (
                        self.used_by_clip.as_ref()?,
                        self.used_by_scroll_px,
                        self.used_by_hovered_index,
                    ),
                };
                node.frame.y -= scroll_px;
                if node.has_clip_frame {
                    node.clip_frame.y -= scroll_px;
                }
                node.hovered = paints_hover && i32::try_from(index).ok() == Some(hovered_index);

                let reference_clip = intersect(&clip, reference_clip)?;
                let node_frame = translated(
                    &frame_from_template(&node.frame),
                    self.origin_x,
                    self.origin_y,
                );
                intersect(&node_frame, &reference_clip)?;
                Some((node, reference_clip))
            }
            AssetContentRowDescriptor::ActivityContent(identity) => {
                if identity == ActivityContentNodeIdentity::ContentPanel {
                    return Some((node, clip));
                }
                if identity.scrolls() {
                    node.frame.y -= self.scroll_px;
                    if node.has_clip_frame {
                        node.clip_frame.y -= self.scroll_px;
                    }
                }
                node.hovered = identity.is_row()
                    && identity.shared_row_index(self.metadata.folder_row_count())
                        == Some(self.hovered_row_index);

                let content_clip = intersect(&clip, self.content_clip.as_ref()?)?;
                let node_frame = translated(
                    &frame_from_template(&node.frame),
                    self.origin_x,
                    self.origin_y,
                );
                intersect(&node_frame, &content_clip)?;
                Some((node, content_clip))
            }
            _ => Some((node, clip)),
        }
    }
}

pub(in crate::ui::retained_host::host_contract::paint_workbench_renderer::docks::pane::template_nodes)
struct BrowserAssetContentProjector
{
    metadata: Rc<AssetContentPaintMetadata>,
    origin_x: f32,
    origin_y: f32,
    row_clip: Option<FrameRect>,
    source_tree_clip: Option<FrameRect>,
    references_clip: Option<FrameRect>,
    used_by_clip: Option<FrameRect>,
    scroll_px: f32,
    hovered_row_index: i32,
    source_tree_scroll_px: f32,
    source_tree_hovered_index: i32,
    references_scroll_px: f32,
    references_hovered_index: i32,
    used_by_scroll_px: f32,
    used_by_hovered_index: i32,
    #[cfg(feature = "profiling")]
    descriptor_lookup_count: Cell<usize>,
}

impl BrowserAssetContentProjector {
    pub(in crate::ui::retained_host::host_contract::paint_workbench_renderer::docks::pane::template_nodes) fn new(
        nodes: &ModelRc<TemplatePaneNodeData>,
        origin: &FrameRect,
        interaction: &HostPaneInteractionStateData,
    ) -> Option<Self> {
        let metadata = nodes.metadata_rc::<AssetContentPaintMetadata>()?;
        if metadata.surface() != AssetContentSurface::Browser {
            return None;
        }
        let row_clip = valid_asset_content_auxiliary_clip(metadata.viewport(), origin);
        let source_tree_clip = metadata
            .browser_source_tree_viewport()
            .and_then(|viewport| valid_asset_content_auxiliary_clip(Some(viewport), origin));
        let references_clip = metadata
            .browser_reference_viewport(BrowserAssetReferenceListKind::References)
            .and_then(|viewport| valid_asset_content_auxiliary_clip(Some(viewport), origin));
        let used_by_clip = metadata
            .browser_reference_viewport(BrowserAssetReferenceListKind::UsedBy)
            .and_then(|viewport| valid_asset_content_auxiliary_clip(Some(viewport), origin));
        if row_clip.is_none()
            && source_tree_clip.is_none()
            && references_clip.is_none()
            && used_by_clip.is_none()
        {
            return None;
        }
        Some(Self {
            metadata,
            origin_x: origin.x,
            origin_y: origin.y,
            row_clip,
            source_tree_clip,
            references_clip,
            used_by_clip,
            scroll_px: interaction.browser_asset_content_scroll_px.max(0.0),
            hovered_row_index: interaction.browser_asset_content_hovered_index,
            source_tree_scroll_px: interaction.browser_asset_tree_scroll_px.max(0.0),
            source_tree_hovered_index: interaction.browser_asset_tree_hovered_index,
            references_scroll_px: interaction.browser_asset_references_scroll_px.max(0.0),
            references_hovered_index: interaction.browser_asset_references_hovered_index,
            used_by_scroll_px: interaction.browser_asset_used_by_scroll_px.max(0.0),
            used_by_hovered_index: interaction.browser_asset_used_by_hovered_index,
            #[cfg(feature = "profiling")]
            descriptor_lookup_count: Cell::new(0),
        })
    }

    #[inline]
    fn record_descriptor_lookup(&self) {
        #[cfg(feature = "profiling")]
        self.descriptor_lookup_count
            .set(self.descriptor_lookup_count.get().saturating_add(1));
    }
}

#[cfg(feature = "profiling")]
impl Drop for BrowserAssetContentProjector {
    fn drop(&mut self) {
        record_current_ui_perf_counter_batch(|counters| {
            counters.push((
                UiPerfCounter::AssetContentDescriptorLookupCount,
                self.descriptor_lookup_count.get() as f64,
            ));
        });
    }
}

impl TemplateNodePaintTransform for BrowserAssetContentProjector {
    fn row_visit_indices(&self, _row_count: usize, clip: &FrameRect) -> Option<Vec<usize>> {
        let (rows, visible_item_count) = self.metadata.visible_browser_node_rows(
            self.scroll_px,
            self.source_tree_scroll_px,
            self.references_scroll_px,
            self.used_by_scroll_px,
            self.origin_x,
            self.origin_y,
            asset_content_rect(clip),
        );
        record_current_ui_perf_counter_batch(|counters| {
            counters.extend_from_slice(&[
                (
                    UiPerfCounter::AssetBrowserLogicalItemCount,
                    self.metadata.browser_logical_item_count() as f64,
                ),
                (
                    UiPerfCounter::AssetBrowserMaterializedItemCount,
                    self.metadata.browser_materialized_item_count() as f64,
                ),
                (
                    UiPerfCounter::AssetBrowserMaterializedNodeCount,
                    self.metadata.browser_materialized_node_count() as f64,
                ),
                (
                    UiPerfCounter::AssetBrowserVisibleItemCount,
                    visible_item_count as f64,
                ),
                (
                    UiPerfCounter::AssetBrowserVisibleNodeCount,
                    rows.len() as f64,
                ),
            ]);
        });
        Some(rows)
    }

    fn transform_row(
        &self,
        row: usize,
        mut node: TemplatePaneNodeData,
        clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)> {
        self.record_descriptor_lookup();
        match self.metadata.row_descriptor(row) {
            AssetContentRowDescriptor::BrowserSourceTree { index } => {
                let source_tree_clip = self.source_tree_clip.as_ref()?;
                node.frame.y -= self.source_tree_scroll_px;
                if node.has_clip_frame {
                    node.clip_frame.y -= self.source_tree_scroll_px;
                }
                node.hovered = i32::try_from(index).ok() == Some(self.source_tree_hovered_index);

                let source_tree_clip = intersect(&clip, source_tree_clip)?;
                let node_frame = translated(
                    &frame_from_template(&node.frame),
                    self.origin_x,
                    self.origin_y,
                );
                intersect(&node_frame, &source_tree_clip)?;
                Some((node, source_tree_clip))
            }
            AssetContentRowDescriptor::BrowserReference {
                list_kind,
                index,
                paints_hover,
            } => {
                let (reference_clip, scroll_px, hovered_index) = match list_kind {
                    BrowserAssetReferenceListKind::References => (
                        self.references_clip.as_ref()?,
                        self.references_scroll_px,
                        self.references_hovered_index,
                    ),
                    BrowserAssetReferenceListKind::UsedBy => (
                        self.used_by_clip.as_ref()?,
                        self.used_by_scroll_px,
                        self.used_by_hovered_index,
                    ),
                };
                node.frame.y -= scroll_px;
                if node.has_clip_frame {
                    node.clip_frame.y -= scroll_px;
                }
                node.hovered = paints_hover && i32::try_from(index).ok() == Some(hovered_index);

                let reference_clip = intersect(&clip, reference_clip)?;
                let node_frame = translated(
                    &frame_from_template(&node.frame),
                    self.origin_x,
                    self.origin_y,
                );
                intersect(&node_frame, &reference_clip)?;
                Some((node, reference_clip))
            }
            AssetContentRowDescriptor::BrowserContent(identity) => {
                let (slot_index, paints_hover, thumbnail_role) = match identity {
                    BrowserContentNodeIdentity::Row { index } => (index, true, None),
                    BrowserContentNodeIdentity::Thumbnail { index, role } => {
                        (index, role.paints_hover(), Some(role))
                    }
                    _ => return Some((node, clip)),
                };
                let (logical_index, y_offset) = if self.metadata.browser_has_virtual_items() {
                    let binding = self
                        .metadata
                        .browser_slot_binding(self.scroll_px, slot_index)?;
                    let thumbnail_slot_card = thumbnail_role
                        .and_then(|_| self.metadata.browser_thumbnail_slot_card_frame(slot_index));
                    if !apply_browser_slot_item(
                        &mut node,
                        binding,
                        thumbnail_role,
                        thumbnail_slot_card,
                    ) {
                        return None;
                    }
                    (binding.logical_index, binding.y_offset)
                } else {
                    (slot_index, 0.0)
                };

                node.frame.y += y_offset - self.scroll_px;
                if node.has_clip_frame {
                    node.clip_frame.y += y_offset - self.scroll_px;
                }
                node.hovered = paints_hover
                    && i32::try_from(logical_index).ok() == Some(self.hovered_row_index);

                let row_clip = intersect(&clip, self.row_clip.as_ref()?)?;
                let node_frame = translated(
                    &frame_from_template(&node.frame),
                    self.origin_x,
                    self.origin_y,
                );
                intersect(&node_frame, &row_clip)?;
                Some((node, row_clip))
            }
            _ => Some((node, clip)),
        }
    }
}

fn apply_browser_slot_item(
    node: &mut TemplatePaneNodeData,
    binding: AssetBrowserSlotBinding<'_>,
    thumbnail_role: Option<BrowserThumbnailNodeRole>,
    thumbnail_slot_card: Option<AssetContentRect>,
) -> bool {
    match binding.item {
        AssetBrowserPaintItem::List(item) => {
            node.text = item.text.clone().into();
            node.options = item.cells.clone();
            node.selected = binding.selected;
            true
        }
        AssetBrowserPaintItem::Thumbnail(item) => {
            let role = thumbnail_role?;
            let slot_card = thumbnail_slot_card?;
            if role == BrowserThumbnailNodeRole::SelectionMarker && !binding.selected {
                return false;
            }
            let geometry = asset_thumbnail_card_geometry(
                slot_card,
                !item.name_continuation.is_empty(),
                item.type_label_width,
            );
            apply_thumbnail_item_frame(node, geometry.for_role(role));
            node.selected = binding.selected;
            match role {
                BrowserThumbnailNodeRole::Card => {
                    node.border_width = if binding.selected { 1.0 } else { 0.0 };
                }
                BrowserThumbnailNodeRole::Visual => {
                    node.component_variant = item.visual_variant.clone().into();
                    node.media_source = item.preview_artifact_path.clone().into();
                    node.has_preview_image = !item.preview_artifact_path.trim().is_empty();
                    node.surface_variant = if binding.selected {
                        "asset-preview-visual".into()
                    } else {
                        "asset-placeholder-visual".into()
                    };
                }
                BrowserThumbnailNodeRole::NameContinuation => {
                    node.text = item.name_continuation.clone().into();
                }
                BrowserThumbnailNodeRole::Name => {
                    node.text = if item.source_file_name.is_empty() {
                        item.name.clone()
                    } else {
                        compact_thumbnail_file_name_to_width(
                            item.source_file_name.as_str(),
                            item.file_extension.as_str(),
                            node.frame.width,
                            node.font_size,
                        )
                    }
                    .into();
                    node.value_text = item.source_file_name.clone().into();
                }
                BrowserThumbnailNodeRole::Type => {
                    node.text = item.type_label.clone().into();
                }
                BrowserThumbnailNodeRole::Meta => {
                    node.text = item.state_label.clone().into();
                }
                BrowserThumbnailNodeRole::InfoBand
                | BrowserThumbnailNodeRole::SelectionMarker
                | BrowserThumbnailNodeRole::TypeBadge => {}
            }
            true
        }
    }
}

fn apply_thumbnail_item_frame(node: &mut TemplatePaneNodeData, frame: AssetContentRect) {
    let delta_x = frame.x - node.frame.x;
    let delta_y = frame.y - node.frame.y;
    node.frame.x = frame.x;
    node.frame.y = frame.y;
    node.frame.width = frame.width;
    node.frame.height = frame.height;
    if node.has_clip_frame {
        node.clip_frame.x += delta_x;
        node.clip_frame.y += delta_y;
        node.clip_frame.width = frame.width;
        node.clip_frame.height = frame.height;
    }
}

fn translated_asset_content_rect(rect: AssetContentRect, origin: &FrameRect) -> FrameRect {
    FrameRect {
        x: origin.x + rect.x,
        y: origin.y + rect.y,
        width: rect.width,
        height: rect.height,
    }
}

fn valid_asset_content_auxiliary_clip(
    rect: Option<AssetContentRect>,
    origin: &FrameRect,
) -> Option<FrameRect> {
    let rect = translated_asset_content_rect(rect?, origin);
    (rect.width > 0.0 && rect.height > 0.0).then_some(rect)
}

fn asset_content_rect(rect: &FrameRect) -> AssetContentRect {
    AssetContentRect {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
    }
}
