use std::rc::Rc;

use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostPaneInteractionStateData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::paint_geometry::{
    frame_from_template, intersect, translated,
};
use crate::ui::retained_host::host_contract::paint_template_nodes::TemplateNodePaintTransform;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::asset_content_layout::{
    ActivityAssetReferenceListKind, ActivityContentNodeIdentity, AssetContentNodeIdentity,
    AssetContentPaintMetadata, AssetContentRect, AssetContentSurface,
    BrowserAssetReferenceListKind, BrowserContentNodeIdentity, activity_reference_row_index,
    browser_reference_row_index, browser_source_tree_row_index,
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
        })
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

    fn transform(
        &self,
        mut node: TemplatePaneNodeData,
        clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)> {
        if let Some((list_kind, index)) = activity_reference_row_index(node.control_id.as_str()) {
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
            node.hovered = node.control_id.as_str().contains("RowPanel")
                && i32::try_from(index).ok() == Some(hovered_index);

            let reference_clip = intersect(&clip, reference_clip)?;
            let node_frame = translated(
                &frame_from_template(&node.frame),
                self.origin_x,
                self.origin_y,
            );
            intersect(&node_frame, &reference_clip)?;
            return Some((node, reference_clip));
        }
        let Some(AssetContentNodeIdentity::Activity(identity)) =
            self.metadata.identity(node.control_id.as_str())
        else {
            return Some((node, clip));
        };
        if identity == ActivityContentNodeIdentity::ContentPanel {
            return Some((node, clip));
        }

        if self.metadata.is_scroll_node(node.control_id.as_str()) {
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
    mode: BrowserAssetContentProjectionMode,
}

#[derive(Clone, Copy)]
enum BrowserAssetContentProjectionMode {
    List,
    Thumbnail,
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
        let mode = if metadata.browser_uses_thumbnails() {
            BrowserAssetContentProjectionMode::Thumbnail
        } else {
            BrowserAssetContentProjectionMode::List
        };
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
            mode,
        })
    }
}

impl TemplateNodePaintTransform for BrowserAssetContentProjector {
    fn row_visit_indices(&self, _row_count: usize, clip: &FrameRect) -> Option<Vec<usize>> {
        Some(self.metadata.visible_browser_node_rows(
            self.scroll_px,
            self.source_tree_scroll_px,
            self.references_scroll_px,
            self.used_by_scroll_px,
            self.origin_x,
            self.origin_y,
            asset_content_rect(clip),
        ))
    }

    fn transform(
        &self,
        mut node: TemplatePaneNodeData,
        clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)> {
        if let Some(index) = browser_source_tree_row_index(node.control_id.as_str()) {
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
            return Some((node, source_tree_clip));
        }
        if let Some((list_kind, index)) = browser_reference_row_index(node.control_id.as_str()) {
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
            node.hovered = node.control_id.as_str().contains("RowPanel")
                && i32::try_from(index).ok() == Some(hovered_index);

            let reference_clip = intersect(&clip, reference_clip)?;
            let node_frame = translated(
                &frame_from_template(&node.frame),
                self.origin_x,
                self.origin_y,
            );
            intersect(&node_frame, &reference_clip)?;
            return Some((node, reference_clip));
        }
        let Some(AssetContentNodeIdentity::Browser(identity)) =
            self.metadata.identity(node.control_id.as_str())
        else {
            return Some((node, clip));
        };
        let (index, paints_hover) = match (self.mode, identity) {
            (
                BrowserAssetContentProjectionMode::List,
                BrowserContentNodeIdentity::Row { index },
            ) => (index, true),
            (
                BrowserAssetContentProjectionMode::Thumbnail,
                BrowserContentNodeIdentity::Thumbnail { index, role },
            ) => (index, role.paints_hover()),
            _ => return Some((node, clip)),
        };

        node.frame.y -= self.scroll_px;
        if node.has_clip_frame {
            node.clip_frame.y -= self.scroll_px;
        }
        node.hovered = paints_hover && i32::try_from(index).ok() == Some(self.hovered_row_index);

        let row_clip = intersect(&clip, self.row_clip.as_ref()?)?;
        let node_frame = translated(
            &frame_from_template(&node.frame),
            self.origin_x,
            self.origin_y,
        );
        intersect(&node_frame, &row_clip)?;
        Some((node, row_clip))
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
