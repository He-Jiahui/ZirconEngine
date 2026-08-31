use std::collections::BTreeMap;

mod scrollbar_descriptors;

use scrollbar_descriptors::{build_scrollbar_descriptors, AssetContentScrollbarDescriptors};
pub(crate) use scrollbar_descriptors::{
    AssetContentScrollbarDescriptor, AssetContentScrollbarExtent, AssetContentScrollbarKind,
    AssetContentScrollbarViewport,
};

use super::browser_virtualization::{
    append_visible_virtual_group_rows, AssetBrowserLogicalPaintGeneration, AssetBrowserSlotBinding,
    AssetBrowserVirtualization,
};
use super::controls::{ActivityAssetReferenceListKind, BrowserAssetReferenceListKind};
use super::identity::{
    describe_asset_content_row, ActivityContentNodeIdentity, ActivityContentNodeRole,
    AssetContentRowDescriptor, AssetContentSurface, BrowserContentNodeIdentity,
    BrowserThumbnailNodeRole,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AssetContentRect {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl AssetContentRect {
    pub(crate) fn translated(self, x: f32, y: f32) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
            ..self
        }
    }

    pub(super) fn bottom(self) -> f32 {
        self.y + self.height
    }

    pub(super) fn intersect(self, other: Self) -> Option<Self> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = (self.x + self.width).min(other.x + other.width);
        let bottom = self.bottom().min(other.bottom());
        (right > x && bottom > y).then_some(Self {
            x,
            y,
            width: right - x,
            height: bottom - y,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AssetContentPaintNodeInput<'a> {
    control_id: &'a str,
    frame: AssetContentRect,
    value_number: f32,
}

impl<'a> AssetContentPaintNodeInput<'a> {
    pub(crate) fn new(
        control_id: &'a str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        value_number: f32,
    ) -> Self {
        Self {
            control_id,
            frame: AssetContentRect {
                x,
                y,
                width: width.max(0.0),
                height: height.max(0.0),
            },
            value_number,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct AssetContentRowGroup {
    pub(super) top: f32,
    pub(super) bottom: f32,
    pub(super) node_rows: Vec<usize>,
}

#[derive(Default)]
struct AssetContentGeometry {
    content_panel: Option<AssetContentRect>,
    viewport: Option<AssetContentRect>,
    activity_references_viewport: Option<AssetContentRect>,
    activity_used_by_viewport: Option<AssetContentRect>,
    browser_source_tree_viewport: Option<AssetContentRect>,
    browser_references_viewport: Option<AssetContentRect>,
    browser_used_by_viewport: Option<AssetContentRect>,
    content_extent: f32,
    browser_table: Option<(AssetContentRect, f32)>,
    browser_header_bottom: Option<f32>,
    browser_preview_top: Option<f32>,
}

impl AssetContentGeometry {
    fn observe(
        &mut self,
        surface: AssetContentSurface,
        descriptor: AssetContentRowDescriptor,
        node: AssetContentPaintNodeInput<'_>,
    ) {
        match descriptor {
            AssetContentRowDescriptor::ActivityContent(
                ActivityContentNodeIdentity::ContentPanel,
            ) if surface == AssetContentSurface::Activity => {
                self.content_panel = Some(node.frame);
                self.viewport = Some(node.frame);
                self.content_extent = finite_content_extent(node.value_number);
            }
            AssetContentRowDescriptor::ActivityReferenceViewport(
                ActivityAssetReferenceListKind::References,
            ) if surface == AssetContentSurface::Activity => {
                self.activity_references_viewport = Some(node.frame);
            }
            AssetContentRowDescriptor::ActivityReferenceViewport(
                ActivityAssetReferenceListKind::UsedBy,
            ) if surface == AssetContentSurface::Activity => {
                self.activity_used_by_viewport = Some(node.frame);
            }
            AssetContentRowDescriptor::BrowserContent(
                BrowserContentNodeIdentity::ThumbnailGrid,
            ) if surface == AssetContentSurface::Browser => {
                self.content_panel = Some(node.frame);
                self.viewport = Some(node.frame);
                self.content_extent = finite_content_extent(node.value_number);
            }
            AssetContentRowDescriptor::BrowserContent(BrowserContentNodeIdentity::TablePanel)
                if surface == AssetContentSurface::Browser =>
            {
                self.content_panel = Some(node.frame);
                self.browser_table = Some((node.frame, node.value_number));
            }
            AssetContentRowDescriptor::BrowserContent(BrowserContentNodeIdentity::Header)
                if surface == AssetContentSurface::Browser =>
            {
                self.browser_header_bottom = Some(node.frame.bottom());
            }
            AssetContentRowDescriptor::BrowserContent(BrowserContentNodeIdentity::Preview)
                if surface == AssetContentSurface::Browser =>
            {
                self.browser_preview_top = Some(node.frame.y);
            }
            AssetContentRowDescriptor::BrowserSourceTreeViewport
                if surface == AssetContentSurface::Browser =>
            {
                self.browser_source_tree_viewport = Some(node.frame);
            }
            AssetContentRowDescriptor::BrowserReferenceViewport(
                BrowserAssetReferenceListKind::References,
            ) if surface == AssetContentSurface::Browser => {
                self.browser_references_viewport = Some(node.frame);
            }
            AssetContentRowDescriptor::BrowserReferenceViewport(
                BrowserAssetReferenceListKind::UsedBy,
            ) if surface == AssetContentSurface::Browser => {
                self.browser_used_by_viewport = Some(node.frame);
            }
            _ => {}
        }
    }

    fn finish(mut self, surface: AssetContentSurface) -> Self {
        if surface != AssetContentSurface::Browser || self.viewport.is_some() {
            return self;
        }
        let (table, extent) = match self.browser_table {
            Some(table) => table,
            None => return self,
        };
        let header_bottom = match self.browser_header_bottom {
            Some(header_bottom) => header_bottom,
            None => return self,
        };
        let rows_bottom = self
            .browser_preview_top
            .unwrap_or(table.bottom())
            .min(table.bottom());
        self.viewport = Some(AssetContentRect {
            x: table.x,
            y: header_bottom,
            width: table.width,
            height: (rows_bottom - header_bottom).max(0.0),
        });
        self.content_extent = finite_content_extent(extent);
        self
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AssetContentPaintMetadata {
    surface: AssetContentSurface,
    content_panel: Option<AssetContentRect>,
    viewport: Option<AssetContentRect>,
    activity_references_viewport: Option<AssetContentRect>,
    activity_used_by_viewport: Option<AssetContentRect>,
    activity_tree_rows: Vec<usize>,
    activity_references_groups: Vec<AssetContentRowGroup>,
    activity_used_by_groups: Vec<AssetContentRowGroup>,
    browser_source_tree_viewport: Option<AssetContentRect>,
    browser_source_tree_groups: Vec<AssetContentRowGroup>,
    browser_references_viewport: Option<AssetContentRect>,
    browser_used_by_viewport: Option<AssetContentRect>,
    browser_references_groups: Vec<AssetContentRowGroup>,
    browser_used_by_groups: Vec<AssetContentRowGroup>,
    scrollbar_descriptors: AssetContentScrollbarDescriptors,
    content_extent: f32,
    folder_row_count: usize,
    browser_uses_thumbnails: bool,
    browser_materialized_item_count: usize,
    browser_materialized_node_count: usize,
    browser_virtualization: Option<AssetBrowserVirtualization>,
    #[cfg(any(feature = "profiling", test))]
    identity_parse_count: usize,
    row_descriptors: Vec<AssetContentRowDescriptor>,
    thumbnail_slot_cards: Vec<Option<AssetContentRect>>,
    fixed_node_rows: Vec<usize>,
    scroll_groups: Vec<AssetContentRowGroup>,
}

impl AssetContentPaintMetadata {
    fn build<'a, I>(nodes: I, surface: AssetContentSurface) -> Self
    where
        I: Iterator<Item = AssetContentPaintNodeInput<'a>> + Clone,
    {
        let row_descriptors = nodes
            .clone()
            .map(|node| describe_asset_content_row(surface, node.control_id))
            .collect::<Vec<_>>();
        let folder_row_count = row_descriptors
            .iter()
            .filter(|descriptor| {
                matches!(
                    descriptor,
                    AssetContentRowDescriptor::ActivityContent(
                        ActivityContentNodeIdentity::Folder {
                            role: ActivityContentNodeRole::Row,
                            ..
                        }
                    )
                )
            })
            .count();
        let browser_uses_thumbnails = row_descriptors.iter().any(|descriptor| {
            matches!(
                descriptor,
                AssetContentRowDescriptor::BrowserContent(
                    BrowserContentNodeIdentity::ThumbnailGrid
                )
            )
        });

        let mut fixed_node_rows = Vec::new();
        let mut activity_tree_rows = Vec::new();
        let mut groups = BTreeMap::<usize, AssetContentRowGroup>::new();
        let mut activity_reference_groups =
            BTreeMap::<(ActivityAssetReferenceListKind, usize), AssetContentRowGroup>::new();
        let mut browser_source_tree_groups = BTreeMap::<usize, AssetContentRowGroup>::new();
        let mut browser_reference_groups =
            BTreeMap::<(BrowserAssetReferenceListKind, usize), AssetContentRowGroup>::new();
        let mut geometry = AssetContentGeometry::default();
        let mut thumbnail_slot_cards = Vec::new();
        for (row, node) in nodes.clone().enumerate() {
            let descriptor = row_descriptors[row];
            geometry.observe(surface, descriptor, node);
            if descriptor == AssetContentRowDescriptor::ActivityTreeRow {
                activity_tree_rows.push(row);
            }
            if let AssetContentRowDescriptor::BrowserContent(
                BrowserContentNodeIdentity::Thumbnail {
                    index,
                    role: BrowserThumbnailNodeRole::Card,
                },
            ) = descriptor
            {
                if thumbnail_slot_cards.len() <= index {
                    thumbnail_slot_cards.resize(index + 1, None);
                }
                thumbnail_slot_cards[index] = Some(node.frame);
            }
            if let AssetContentRowDescriptor::BrowserSourceTree { index } = descriptor {
                let frame = node.frame;
                let group = browser_source_tree_groups.entry(index).or_insert_with(|| {
                    AssetContentRowGroup {
                        top: frame.y,
                        bottom: frame.bottom(),
                        node_rows: Vec::new(),
                    }
                });
                group.top = group.top.min(frame.y);
                group.bottom = group.bottom.max(frame.bottom());
                group.node_rows.push(row);
                continue;
            }
            if let AssetContentRowDescriptor::ActivityReference {
                list_kind, index, ..
            } = descriptor
            {
                let frame = node.frame;
                let group = activity_reference_groups
                    .entry((list_kind, index))
                    .or_insert_with(|| AssetContentRowGroup {
                        top: frame.y,
                        bottom: frame.bottom(),
                        node_rows: Vec::new(),
                    });
                group.top = group.top.min(frame.y);
                group.bottom = group.bottom.max(frame.bottom());
                group.node_rows.push(row);
                continue;
            }
            if let AssetContentRowDescriptor::BrowserReference {
                list_kind, index, ..
            } = descriptor
            {
                let frame = node.frame;
                let group = browser_reference_groups
                    .entry((list_kind, index))
                    .or_insert_with(|| AssetContentRowGroup {
                        top: frame.y,
                        bottom: frame.bottom(),
                        node_rows: Vec::new(),
                    });
                group.top = group.top.min(frame.y);
                group.bottom = group.bottom.max(frame.bottom());
                group.node_rows.push(row);
                continue;
            }
            let group = match descriptor {
                AssetContentRowDescriptor::ActivityContent(
                    ActivityContentNodeIdentity::Folder { index, .. },
                ) => Some(index),
                AssetContentRowDescriptor::ActivityContent(ActivityContentNodeIdentity::Item {
                    index,
                    ..
                }) => folder_row_count.checked_add(index),
                AssetContentRowDescriptor::BrowserContent(BrowserContentNodeIdentity::Row {
                    index,
                }) if !browser_uses_thumbnails => Some(index),
                AssetContentRowDescriptor::BrowserContent(
                    BrowserContentNodeIdentity::Thumbnail { index, .. },
                ) if browser_uses_thumbnails => Some(index),
                _ => None,
            };
            if let Some(group) = group {
                let frame = node.frame;
                let group = groups.entry(group).or_insert_with(|| AssetContentRowGroup {
                    top: frame.y,
                    bottom: frame.bottom(),
                    node_rows: Vec::new(),
                });
                group.top = group.top.min(frame.y);
                group.bottom = group.bottom.max(frame.bottom());
                group.node_rows.push(row);
            } else {
                fixed_node_rows.push(row);
            }
        }
        let mut scroll_groups = groups.into_values().collect::<Vec<_>>();
        scroll_groups.sort_by(|left, right| left.top.total_cmp(&right.top));
        let (browser_materialized_item_count, browser_materialized_node_count) =
            if surface == AssetContentSurface::Browser {
                (
                    scroll_groups.len(),
                    scroll_groups
                        .iter()
                        .map(|group| group.node_rows.len())
                        .sum(),
                )
            } else {
                (0, 0)
            };
        let mut browser_source_tree_groups =
            browser_source_tree_groups.into_values().collect::<Vec<_>>();
        browser_source_tree_groups.sort_by(|left, right| left.top.total_cmp(&right.top));
        let mut activity_references_groups = Vec::new();
        let mut activity_used_by_groups = Vec::new();
        for ((list_kind, _), group) in activity_reference_groups {
            match list_kind {
                ActivityAssetReferenceListKind::References => {
                    activity_references_groups.push(group)
                }
                ActivityAssetReferenceListKind::UsedBy => activity_used_by_groups.push(group),
            }
        }
        activity_references_groups.sort_by(|left, right| left.top.total_cmp(&right.top));
        activity_used_by_groups.sort_by(|left, right| left.top.total_cmp(&right.top));
        let mut browser_references_groups = Vec::new();
        let mut browser_used_by_groups = Vec::new();
        for ((list_kind, _), group) in browser_reference_groups {
            match list_kind {
                BrowserAssetReferenceListKind::References => browser_references_groups.push(group),
                BrowserAssetReferenceListKind::UsedBy => browser_used_by_groups.push(group),
            }
        }
        browser_references_groups.sort_by(|left, right| left.top.total_cmp(&right.top));
        browser_used_by_groups.sort_by(|left, right| left.top.total_cmp(&right.top));
        let geometry = geometry.finish(surface);
        let scrollbar_descriptors = build_scrollbar_descriptors(surface, &geometry);

        Self {
            surface,
            content_panel: geometry.content_panel,
            viewport: geometry.viewport,
            activity_references_viewport: geometry.activity_references_viewport,
            activity_used_by_viewport: geometry.activity_used_by_viewport,
            activity_tree_rows,
            activity_references_groups,
            activity_used_by_groups,
            browser_source_tree_viewport: geometry.browser_source_tree_viewport,
            browser_source_tree_groups,
            browser_references_viewport: geometry.browser_references_viewport,
            browser_used_by_viewport: geometry.browser_used_by_viewport,
            browser_references_groups,
            browser_used_by_groups,
            scrollbar_descriptors,
            content_extent: geometry.content_extent,
            folder_row_count,
            browser_uses_thumbnails,
            browser_materialized_item_count,
            browser_materialized_node_count,
            browser_virtualization: None,
            #[cfg(any(feature = "profiling", test))]
            identity_parse_count: row_descriptors.len(),
            row_descriptors,
            thumbnail_slot_cards,
            fixed_node_rows,
            scroll_groups,
        }
    }

    pub(crate) fn with_browser_virtual_items(
        mut self,
        items: AssetBrowserLogicalPaintGeneration,
        selected_item_indices: Vec<usize>,
        overscan_rows: usize,
    ) -> Self {
        if self.surface != AssetContentSurface::Browser {
            return self;
        }
        if self.scroll_groups.is_empty() {
            let viewport_top = self.viewport.map_or(0.0, |viewport| viewport.y);
            self.browser_virtualization = Some(AssetBrowserVirtualization::new(
                items,
                selected_item_indices,
                0,
                1,
                viewport_top,
                0.0,
                viewport_top,
                overscan_rows,
            ));
            return self;
        }
        let base_top = self.scroll_groups[0].top;
        let columns = self
            .scroll_groups
            .iter()
            .take_while(|group| (group.top - base_top).abs() <= 0.01)
            .count()
            .max(1);
        let row_stride = self
            .scroll_groups
            .iter()
            .find(|group| group.top > base_top + 0.01)
            .map(|group| group.top - base_top)
            .unwrap_or_else(|| self.scroll_groups[0].bottom - base_top);
        self.browser_virtualization = Some(AssetBrowserVirtualization::new(
            items,
            selected_item_indices,
            self.scroll_groups.len(),
            columns,
            base_top,
            row_stride,
            self.viewport.map_or(base_top, |viewport| viewport.y),
            overscan_rows,
        ));
        self
    }

    pub(crate) fn surface(&self) -> AssetContentSurface {
        self.surface
    }

    pub(crate) fn content_panel(&self) -> Option<AssetContentRect> {
        self.content_panel
    }

    pub(crate) fn viewport(&self) -> Option<AssetContentRect> {
        self.viewport
    }

    pub(crate) fn content_extent(&self) -> f32 {
        self.content_extent
    }

    pub(crate) fn scrollbar_descriptors(&self) -> &[AssetContentScrollbarDescriptor] {
        self.scrollbar_descriptors.as_slice()
    }

    pub(crate) fn browser_source_tree_viewport(&self) -> Option<AssetContentRect> {
        self.browser_source_tree_viewport
    }

    pub(crate) fn asset_tree_row_count(&self) -> usize {
        match self.surface {
            AssetContentSurface::Activity => self.activity_tree_rows.len(),
            AssetContentSurface::Browser => self.browser_source_tree_groups.len(),
        }
    }

    pub(crate) fn activity_tree_node_row(&self, index: usize) -> Option<usize> {
        if self.surface != AssetContentSurface::Activity {
            return None;
        }
        self.activity_tree_rows.get(index).copied()
    }

    pub(crate) fn activity_reference_viewport(
        &self,
        list_kind: ActivityAssetReferenceListKind,
    ) -> Option<AssetContentRect> {
        match list_kind {
            ActivityAssetReferenceListKind::References => self.activity_references_viewport,
            ActivityAssetReferenceListKind::UsedBy => self.activity_used_by_viewport,
        }
    }

    pub(crate) fn browser_reference_viewport(
        &self,
        list_kind: BrowserAssetReferenceListKind,
    ) -> Option<AssetContentRect> {
        match list_kind {
            BrowserAssetReferenceListKind::References => self.browser_references_viewport,
            BrowserAssetReferenceListKind::UsedBy => self.browser_used_by_viewport,
        }
    }

    pub(crate) fn browser_reference_row_count(
        &self,
        list_kind: BrowserAssetReferenceListKind,
    ) -> usize {
        match list_kind {
            BrowserAssetReferenceListKind::References => self.browser_references_groups.len(),
            BrowserAssetReferenceListKind::UsedBy => self.browser_used_by_groups.len(),
        }
    }

    pub(crate) fn activity_reference_row_count(
        &self,
        list_kind: ActivityAssetReferenceListKind,
    ) -> usize {
        match list_kind {
            ActivityAssetReferenceListKind::References => self.activity_references_groups.len(),
            ActivityAssetReferenceListKind::UsedBy => self.activity_used_by_groups.len(),
        }
    }

    pub(crate) fn folder_row_count(&self) -> usize {
        self.folder_row_count
    }

    pub(crate) fn browser_uses_thumbnails(&self) -> bool {
        self.browser_uses_thumbnails
    }

    pub(crate) fn browser_materialized_item_count(&self) -> usize {
        self.browser_materialized_item_count
    }

    pub(crate) fn browser_materialized_node_count(&self) -> usize {
        self.browser_materialized_node_count
    }

    pub(crate) fn browser_logical_item_count(&self) -> usize {
        self.browser_virtualization
            .as_ref()
            .map_or(self.browser_materialized_item_count, |virtualization| {
                virtualization.logical_item_count()
            })
    }

    pub(crate) fn browser_has_virtual_items(&self) -> bool {
        self.browser_virtualization.is_some()
    }

    pub(crate) fn browser_slot_binding(
        &self,
        scroll_px: f32,
        slot_index: usize,
    ) -> Option<AssetBrowserSlotBinding<'_>> {
        self.browser_virtualization
            .as_ref()?
            .binding(scroll_px, slot_index)
    }

    #[cfg(test)]
    pub(crate) fn shares_browser_logical_items_with(&self, other: &Self) -> bool {
        match (
            self.browser_virtualization.as_ref(),
            other.browser_virtualization.as_ref(),
        ) {
            (Some(current), Some(other)) => current.shares_items_with(other),
            _ => false,
        }
    }

    #[cfg(test)]
    pub(crate) fn shares_browser_logical_item_chunk_with(
        &self,
        index: usize,
        other: &Self,
    ) -> bool {
        match (
            self.browser_virtualization.as_ref(),
            other.browser_virtualization.as_ref(),
        ) {
            (Some(current), Some(other)) => current.shares_item_chunk_with(index, other),
            _ => false,
        }
    }

    pub(crate) fn row_descriptor(&self, row: usize) -> AssetContentRowDescriptor {
        self.row_descriptors
            .get(row)
            .copied()
            .unwrap_or(AssetContentRowDescriptor::Fixed)
    }

    /// Every generation input is classified once before the descriptor vector is published.
    #[cfg(any(feature = "profiling", test))]
    pub(crate) fn identity_parse_count(&self) -> usize {
        self.identity_parse_count
    }

    pub(crate) fn browser_thumbnail_slot_card_frame(
        &self,
        slot_index: usize,
    ) -> Option<AssetContentRect> {
        self.thumbnail_slot_cards.get(slot_index).copied().flatten()
    }

    pub(crate) fn visible_node_rows(
        &self,
        scroll_px: f32,
        origin_x: f32,
        origin_y: f32,
        damage_clip: AssetContentRect,
    ) -> Vec<usize> {
        let mut rows = self.fixed_node_rows.clone();
        append_visible_group_rows(
            &mut rows,
            &self.scroll_groups,
            self.viewport,
            scroll_px,
            origin_x,
            origin_y,
            damage_clip,
        );
        rows.sort_unstable();
        rows
    }

    pub(crate) fn visible_activity_node_rows(
        &self,
        content_scroll_px: f32,
        references_scroll_px: f32,
        used_by_scroll_px: f32,
        origin_x: f32,
        origin_y: f32,
        damage_clip: AssetContentRect,
    ) -> Vec<usize> {
        let mut rows = self.fixed_node_rows.clone();
        append_visible_group_rows(
            &mut rows,
            &self.scroll_groups,
            self.viewport,
            content_scroll_px,
            origin_x,
            origin_y,
            damage_clip,
        );
        append_visible_group_rows(
            &mut rows,
            &self.activity_references_groups,
            self.activity_references_viewport,
            references_scroll_px,
            origin_x,
            origin_y,
            damage_clip,
        );
        append_visible_group_rows(
            &mut rows,
            &self.activity_used_by_groups,
            self.activity_used_by_viewport,
            used_by_scroll_px,
            origin_x,
            origin_y,
            damage_clip,
        );
        rows.sort_unstable();
        rows
    }

    pub(crate) fn visible_browser_node_rows(
        &self,
        content_scroll_px: f32,
        source_tree_scroll_px: f32,
        references_scroll_px: f32,
        used_by_scroll_px: f32,
        origin_x: f32,
        origin_y: f32,
        damage_clip: AssetContentRect,
    ) -> (Vec<usize>, usize) {
        let mut rows = self.fixed_node_rows.clone();
        let visible_browser_item_count =
            if let Some(virtualization) = self.browser_virtualization.as_ref() {
                append_visible_virtual_group_rows(
                    &mut rows,
                    &self.scroll_groups,
                    virtualization,
                    self.viewport,
                    content_scroll_px,
                    origin_x,
                    origin_y,
                    damage_clip,
                )
            } else {
                append_visible_group_rows(
                    &mut rows,
                    &self.scroll_groups,
                    self.viewport,
                    content_scroll_px,
                    origin_x,
                    origin_y,
                    damage_clip,
                )
            };
        append_visible_group_rows(
            &mut rows,
            &self.browser_source_tree_groups,
            self.browser_source_tree_viewport,
            source_tree_scroll_px,
            origin_x,
            origin_y,
            damage_clip,
        );
        append_visible_group_rows(
            &mut rows,
            &self.browser_references_groups,
            self.browser_references_viewport,
            references_scroll_px,
            origin_x,
            origin_y,
            damage_clip,
        );
        append_visible_group_rows(
            &mut rows,
            &self.browser_used_by_groups,
            self.browser_used_by_viewport,
            used_by_scroll_px,
            origin_x,
            origin_y,
            damage_clip,
        );
        rows.sort_unstable();
        (rows, visible_browser_item_count)
    }
}

fn append_visible_group_rows(
    rows: &mut Vec<usize>,
    groups: &[AssetContentRowGroup],
    viewport: Option<AssetContentRect>,
    scroll_px: f32,
    origin_x: f32,
    origin_y: f32,
    damage_clip: AssetContentRect,
) -> usize {
    let Some(viewport) = viewport.map(|viewport| viewport.translated(origin_x, origin_y)) else {
        return 0;
    };
    let Some(visible) = viewport.intersect(damage_clip) else {
        return 0;
    };
    let visible_top = visible.y - origin_y + scroll_px.max(0.0);
    let visible_bottom = visible.bottom() - origin_y + scroll_px.max(0.0);
    let first = groups.partition_point(|group| group.bottom <= visible_top);
    let last = groups.partition_point(|group| group.top < visible_bottom);
    for group in &groups[first.min(last)..last] {
        rows.extend_from_slice(&group.node_rows);
    }
    last.saturating_sub(first.min(last))
}

pub(crate) fn asset_content_paint_metadata<'a, I>(
    nodes: I,
    surface: AssetContentSurface,
) -> AssetContentPaintMetadata
where
    I: Iterator<Item = AssetContentPaintNodeInput<'a>> + Clone,
{
    AssetContentPaintMetadata::build(nodes, surface)
}

fn finite_content_extent(extent: f32) -> f32 {
    if extent.is_finite() {
        extent.max(0.0)
    } else {
        0.0
    }
}
