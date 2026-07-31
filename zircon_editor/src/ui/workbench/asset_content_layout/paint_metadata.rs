use std::collections::BTreeMap;

use super::controls::{
    ACTIVITY_CONTENT_EMPTY_CONTROL_ID, ACTIVITY_CONTENT_FOLDER_PREFIX,
    ACTIVITY_CONTENT_ITEM_PREFIX, ACTIVITY_CONTENT_PANEL_CONTROL_ID, BROWSER_CONTENT_ITEM_PREFIX,
    BROWSER_CONTENT_PREVIEW_CONTROL_ID, BROWSER_CONTENT_TABLE_CONTROL_ID,
    BROWSER_CONTENT_TABLE_HEADER_CONTROL_ID, BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AssetContentSurface {
    Activity,
    Browser,
}

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

    fn bottom(self) -> f32 {
        self.y + self.height
    }

    fn intersect(self, other: Self) -> Option<Self> {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ActivityContentNodeRole {
    Row,
    Badge,
    Type,
    Name,
    Meta,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ActivityContentNodeIdentity {
    ContentPanel,
    Empty,
    Folder {
        index: usize,
        role: ActivityContentNodeRole,
    },
    Item {
        index: usize,
        role: ActivityContentNodeRole,
    },
}

impl ActivityContentNodeIdentity {
    pub(crate) fn is_row(self) -> bool {
        matches!(
            self,
            Self::Folder {
                role: ActivityContentNodeRole::Row,
                ..
            } | Self::Item {
                role: ActivityContentNodeRole::Row,
                ..
            }
        )
    }

    pub(crate) fn shared_row_index(self, folder_row_count: usize) -> Option<i32> {
        match self {
            Self::Folder { index, .. } => i32::try_from(index).ok(),
            Self::Item { index, .. } => folder_row_count
                .checked_add(index)
                .and_then(|index| i32::try_from(index).ok()),
            Self::ContentPanel | Self::Empty => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BrowserContentNodeIdentity {
    TablePanel,
    Header,
    Row {
        index: usize,
    },
    ThumbnailGrid,
    Thumbnail {
        index: usize,
        role: BrowserThumbnailNodeRole,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BrowserThumbnailNodeRole {
    Card,
    InfoBand,
    Child,
}

impl BrowserThumbnailNodeRole {
    pub(crate) fn paints_hover(self) -> bool {
        matches!(self, Self::Card | Self::InfoBand)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AssetContentNodeIdentity {
    Activity(ActivityContentNodeIdentity),
    Browser(BrowserContentNodeIdentity),
}

#[derive(Clone, Debug)]
struct AssetContentRowGroup {
    top: f32,
    bottom: f32,
    node_rows: Vec<usize>,
}

#[derive(Clone, Debug)]
pub(crate) struct AssetContentPaintMetadata {
    surface: AssetContentSurface,
    viewport: Option<AssetContentRect>,
    content_extent: f32,
    folder_row_count: usize,
    browser_uses_thumbnails: bool,
    identities: BTreeMap<String, AssetContentNodeIdentity>,
    fixed_node_rows: Vec<usize>,
    scroll_groups: Vec<AssetContentRowGroup>,
}

impl AssetContentPaintMetadata {
    fn build(nodes: &[AssetContentPaintNodeInput<'_>], surface: AssetContentSurface) -> Self {
        let identities = nodes
            .iter()
            .map(|node| match surface {
                AssetContentSurface::Activity => parse_activity_content_identity(node.control_id)
                    .map(AssetContentNodeIdentity::Activity),
                AssetContentSurface::Browser => parse_browser_content_identity(node.control_id)
                    .map(AssetContentNodeIdentity::Browser),
            })
            .collect::<Vec<_>>();
        let folder_row_count = identities
            .iter()
            .filter(|identity| {
                matches!(
                    identity,
                    Some(AssetContentNodeIdentity::Activity(
                        ActivityContentNodeIdentity::Folder {
                            role: ActivityContentNodeRole::Row,
                            ..
                        }
                    ))
                )
            })
            .count();
        let browser_uses_thumbnails = identities.iter().any(|identity| {
            matches!(
                identity,
                Some(AssetContentNodeIdentity::Browser(
                    BrowserContentNodeIdentity::ThumbnailGrid
                ))
            )
        });

        let mut identity_index = BTreeMap::new();
        let mut fixed_node_rows = Vec::new();
        let mut groups = BTreeMap::<usize, AssetContentRowGroup>::new();
        for (row, (node, identity)) in nodes.iter().zip(identities).enumerate() {
            if let Some(identity) = identity {
                identity_index.insert(node.control_id.to_owned(), identity);
            }
            let group =
                match identity {
                    Some(AssetContentNodeIdentity::Activity(
                        ActivityContentNodeIdentity::Folder { index, .. },
                    )) => Some(index),
                    Some(AssetContentNodeIdentity::Activity(
                        ActivityContentNodeIdentity::Item { index, .. },
                    )) => folder_row_count.checked_add(index),
                    Some(AssetContentNodeIdentity::Browser(BrowserContentNodeIdentity::Row {
                        index,
                    })) if !browser_uses_thumbnails => Some(index),
                    Some(AssetContentNodeIdentity::Browser(
                        BrowserContentNodeIdentity::Thumbnail { index, .. },
                    )) if browser_uses_thumbnails => Some(index),
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

        let (viewport, content_extent) = match surface {
            AssetContentSurface::Activity => activity_geometry(nodes, &identity_index),
            AssetContentSurface::Browser => browser_geometry(nodes, &identity_index),
        };

        Self {
            surface,
            viewport,
            content_extent,
            folder_row_count,
            browser_uses_thumbnails,
            identities: identity_index,
            fixed_node_rows,
            scroll_groups,
        }
    }

    pub(crate) fn surface(&self) -> AssetContentSurface {
        self.surface
    }

    pub(crate) fn viewport(&self) -> Option<AssetContentRect> {
        self.viewport
    }

    pub(crate) fn content_extent(&self) -> f32 {
        self.content_extent
    }

    pub(crate) fn folder_row_count(&self) -> usize {
        self.folder_row_count
    }

    pub(crate) fn browser_uses_thumbnails(&self) -> bool {
        self.browser_uses_thumbnails
    }

    pub(crate) fn identity(&self, control_id: &str) -> Option<AssetContentNodeIdentity> {
        self.identities.get(control_id).copied()
    }

    pub(crate) fn is_scroll_node(&self, control_id: &str) -> bool {
        match self.identity(control_id) {
            Some(AssetContentNodeIdentity::Activity(
                ActivityContentNodeIdentity::Folder { .. }
                | ActivityContentNodeIdentity::Item { .. },
            )) => true,
            Some(AssetContentNodeIdentity::Browser(BrowserContentNodeIdentity::Row { .. })) => {
                !self.browser_uses_thumbnails
            }
            Some(AssetContentNodeIdentity::Browser(BrowserContentNodeIdentity::Thumbnail {
                ..
            })) => self.browser_uses_thumbnails,
            _ => false,
        }
    }

    pub(crate) fn visible_node_rows(
        &self,
        scroll_px: f32,
        origin_x: f32,
        origin_y: f32,
        damage_clip: AssetContentRect,
    ) -> Vec<usize> {
        let mut rows = self.fixed_node_rows.clone();
        let Some(viewport) = self
            .viewport
            .map(|viewport| viewport.translated(origin_x, origin_y))
        else {
            return rows;
        };
        let Some(visible) = viewport.intersect(damage_clip) else {
            return rows;
        };
        let visible_top = visible.y - origin_y + scroll_px.max(0.0);
        let visible_bottom = visible.bottom() - origin_y + scroll_px.max(0.0);
        let first = self
            .scroll_groups
            .partition_point(|group| group.bottom <= visible_top);
        let last = self
            .scroll_groups
            .partition_point(|group| group.top < visible_bottom);
        for group in &self.scroll_groups[first.min(last)..last] {
            rows.extend_from_slice(&group.node_rows);
        }
        rows.sort_unstable();
        rows
    }
}

pub(crate) fn asset_content_paint_metadata<'a>(
    nodes: impl IntoIterator<Item = AssetContentPaintNodeInput<'a>>,
    surface: AssetContentSurface,
) -> AssetContentPaintMetadata {
    AssetContentPaintMetadata::build(&nodes.into_iter().collect::<Vec<_>>(), surface)
}

fn activity_geometry(
    nodes: &[AssetContentPaintNodeInput<'_>],
    identities: &BTreeMap<String, AssetContentNodeIdentity>,
) -> (Option<AssetContentRect>, f32) {
    let panel = nodes.iter().find(|node| {
        identities.get(node.control_id)
            == Some(&AssetContentNodeIdentity::Activity(
                ActivityContentNodeIdentity::ContentPanel,
            ))
    });
    match panel {
        Some(panel) => (Some(panel.frame), finite_content_extent(panel.value_number)),
        None => (None, 0.0),
    }
}

fn browser_geometry(
    nodes: &[AssetContentPaintNodeInput<'_>],
    identities: &BTreeMap<String, AssetContentNodeIdentity>,
) -> (Option<AssetContentRect>, f32) {
    let find = |target| {
        nodes.iter().find(|node| {
            identities.get(node.control_id) == Some(&AssetContentNodeIdentity::Browser(target))
        })
    };
    if let Some(grid) = find(BrowserContentNodeIdentity::ThumbnailGrid) {
        return (Some(grid.frame), finite_content_extent(grid.value_number));
    }
    let Some(table) = find(BrowserContentNodeIdentity::TablePanel) else {
        return (None, 0.0);
    };
    let Some(header) = find(BrowserContentNodeIdentity::Header) else {
        return (None, 0.0);
    };
    let table_frame = table.frame;
    let header_bottom = header.frame.bottom();
    let preview_top = nodes
        .iter()
        .find(|node| node.control_id.rsplit('/').next() == Some(BROWSER_CONTENT_PREVIEW_CONTROL_ID))
        .map(|node| node.frame.y);
    let rows_bottom = preview_top
        .unwrap_or(table_frame.bottom())
        .min(table_frame.bottom());
    (
        Some(AssetContentRect {
            x: table_frame.x,
            y: header_bottom,
            width: table_frame.width,
            height: (rows_bottom - header_bottom).max(0.0),
        }),
        finite_content_extent(table.value_number),
    )
}

fn finite_content_extent(extent: f32) -> f32 {
    if extent.is_finite() {
        extent.max(0.0)
    } else {
        0.0
    }
}

pub(crate) fn parse_activity_content_identity(
    control_id: &str,
) -> Option<ActivityContentNodeIdentity> {
    let leaf = control_id.rsplit('/').next()?;
    match leaf {
        ACTIVITY_CONTENT_PANEL_CONTROL_ID => {
            return Some(ActivityContentNodeIdentity::ContentPanel);
        }
        ACTIVITY_CONTENT_EMPTY_CONTROL_ID => return Some(ActivityContentNodeIdentity::Empty),
        _ => {}
    }
    parse_indexed_activity_identity(leaf, ACTIVITY_CONTENT_FOLDER_PREFIX)
        .map(|(index, role)| ActivityContentNodeIdentity::Folder { index, role })
        .or_else(|| {
            parse_indexed_activity_identity(leaf, ACTIVITY_CONTENT_ITEM_PREFIX)
                .map(|(index, role)| ActivityContentNodeIdentity::Item { index, role })
        })
}

fn parse_indexed_activity_identity(
    control_id: &str,
    prefix: &str,
) -> Option<(usize, ActivityContentNodeRole)> {
    let suffix = control_id.strip_prefix(prefix)?;
    for (role_name, role) in [
        ("Row", ActivityContentNodeRole::Row),
        ("Badge", ActivityContentNodeRole::Badge),
        ("Type", ActivityContentNodeRole::Type),
        ("Name", ActivityContentNodeRole::Name),
        ("Meta", ActivityContentNodeRole::Meta),
    ] {
        if let Some(index) = suffix.strip_prefix(role_name) {
            if !index.is_empty() && index.bytes().all(|byte| byte.is_ascii_digit()) {
                return index.parse().ok().map(|index| (index, role));
            }
        }
    }
    None
}

pub(crate) fn parse_browser_content_identity(
    control_id: &str,
) -> Option<BrowserContentNodeIdentity> {
    let leaf = control_id.rsplit('/').next()?;
    match leaf {
        BROWSER_CONTENT_TABLE_CONTROL_ID => Some(BrowserContentNodeIdentity::TablePanel),
        BROWSER_CONTENT_TABLE_HEADER_CONTROL_ID => Some(BrowserContentNodeIdentity::Header),
        BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID => {
            Some(BrowserContentNodeIdentity::ThumbnailGrid)
        }
        _ => parse_browser_row_identity(leaf).or_else(|| parse_thumbnail_identity(leaf)),
    }
}

fn parse_browser_row_identity(leaf: &str) -> Option<BrowserContentNodeIdentity> {
    leaf.strip_prefix(BROWSER_CONTENT_ITEM_PREFIX)?
        .parse::<usize>()
        .ok()?
        .checked_sub(1)
        .map(|index| BrowserContentNodeIdentity::Row { index })
}

fn parse_thumbnail_identity(leaf: &str) -> Option<BrowserContentNodeIdentity> {
    let suffix = leaf.strip_prefix("AssetBrowserThumb")?;
    for (kind, role) in [
        ("NameContinuation", BrowserThumbnailNodeRole::Child),
        ("SelectionMarker", BrowserThumbnailNodeRole::Child),
        ("InfoBand", BrowserThumbnailNodeRole::InfoBand),
        ("TypeBadge", BrowserThumbnailNodeRole::Child),
        ("Visual", BrowserThumbnailNodeRole::Child),
        ("Card", BrowserThumbnailNodeRole::Card),
        ("Name", BrowserThumbnailNodeRole::Child),
        ("Type", BrowserThumbnailNodeRole::Child),
        ("Meta", BrowserThumbnailNodeRole::Child),
    ] {
        let Some(number) = suffix.strip_prefix(kind) else {
            continue;
        };
        let index = number.parse::<usize>().ok()?.checked_sub(1)?;
        return Some(BrowserContentNodeIdentity::Thumbnail { index, role });
    }
    None
}
