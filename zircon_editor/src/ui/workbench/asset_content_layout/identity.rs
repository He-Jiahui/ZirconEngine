use super::controls::{
    activity_reference_row_index, browser_reference_row_index, browser_source_tree_row_index,
    ActivityAssetReferenceListKind, BrowserAssetReferenceListKind,
    ACTIVITY_CONTENT_EMPTY_CONTROL_ID, ACTIVITY_CONTENT_FOLDER_PREFIX,
    ACTIVITY_CONTENT_ITEM_PREFIX, ACTIVITY_CONTENT_PANEL_CONTROL_ID, ACTIVITY_TREE_ROW_CONTROL_ID,
    BROWSER_CONTENT_ITEM_PREFIX, BROWSER_CONTENT_PREVIEW_CONTROL_ID,
    BROWSER_CONTENT_TABLE_CONTROL_ID, BROWSER_CONTENT_TABLE_HEADER_CONTROL_ID,
    BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AssetContentSurface {
    Activity,
    Browser,
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

    pub(crate) fn scrolls(self) -> bool {
        matches!(self, Self::Folder { .. } | Self::Item { .. })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BrowserContentNodeIdentity {
    TablePanel,
    Header,
    Preview,
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
    SelectionMarker,
    TypeBadge,
    Visual,
    NameContinuation,
    Name,
    Type,
    Meta,
}

impl BrowserThumbnailNodeRole {
    pub(crate) fn paints_hover(self) -> bool {
        matches!(self, Self::Card | Self::InfoBand)
    }
}

/// Generation-owned classification for one template row. Paint projection must
/// use this value, never reconstruct semantics from a control identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AssetContentRowDescriptor {
    Fixed,
    ActivityTreeRow,
    ActivityContent(ActivityContentNodeIdentity),
    ActivityReferenceViewport(ActivityAssetReferenceListKind),
    ActivityReference {
        list_kind: ActivityAssetReferenceListKind,
        index: usize,
        paints_hover: bool,
    },
    BrowserContent(BrowserContentNodeIdentity),
    BrowserSourceTreeViewport,
    BrowserSourceTree {
        index: usize,
    },
    BrowserReferenceViewport(BrowserAssetReferenceListKind),
    BrowserReference {
        list_kind: BrowserAssetReferenceListKind,
        index: usize,
        paints_hover: bool,
    },
}

pub(crate) fn describe_asset_content_row(
    surface: AssetContentSurface,
    control_id: &str,
) -> AssetContentRowDescriptor {
    match surface {
        AssetContentSurface::Activity => describe_activity_row(control_id),
        AssetContentSurface::Browser => describe_browser_row(control_id),
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

pub(crate) fn parse_browser_content_identity(
    control_id: &str,
) -> Option<BrowserContentNodeIdentity> {
    let leaf = control_id.rsplit('/').next()?;
    match leaf {
        BROWSER_CONTENT_TABLE_CONTROL_ID => Some(BrowserContentNodeIdentity::TablePanel),
        BROWSER_CONTENT_TABLE_HEADER_CONTROL_ID => Some(BrowserContentNodeIdentity::Header),
        BROWSER_CONTENT_PREVIEW_CONTROL_ID => Some(BrowserContentNodeIdentity::Preview),
        BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID => {
            Some(BrowserContentNodeIdentity::ThumbnailGrid)
        }
        _ => parse_browser_row_identity(leaf).or_else(|| parse_thumbnail_identity(leaf)),
    }
}

fn describe_activity_row(control_id: &str) -> AssetContentRowDescriptor {
    if let Some(identity) = parse_activity_content_identity(control_id) {
        return AssetContentRowDescriptor::ActivityContent(identity);
    }
    if control_id
        .rsplit('/')
        .next()
        .is_some_and(|leaf| leaf == ACTIVITY_TREE_ROW_CONTROL_ID)
    {
        return AssetContentRowDescriptor::ActivityTreeRow;
    }
    if control_id == "AssetsActivityReferenceLeftScrollBody" {
        return AssetContentRowDescriptor::ActivityReferenceViewport(
            ActivityAssetReferenceListKind::References,
        );
    }
    if control_id == "AssetsActivityReferenceRightScrollBody" {
        return AssetContentRowDescriptor::ActivityReferenceViewport(
            ActivityAssetReferenceListKind::UsedBy,
        );
    }
    activity_reference_row_index(control_id)
        .map(
            |(list_kind, index)| AssetContentRowDescriptor::ActivityReference {
                list_kind,
                index,
                paints_hover: activity_reference_row_paints_hover(control_id, list_kind),
            },
        )
        .unwrap_or(AssetContentRowDescriptor::Fixed)
}

fn describe_browser_row(control_id: &str) -> AssetContentRowDescriptor {
    if let Some(identity) = parse_browser_content_identity(control_id) {
        return AssetContentRowDescriptor::BrowserContent(identity);
    }
    if control_id == "AssetBrowserSourcesScrollBody" {
        return AssetContentRowDescriptor::BrowserSourceTreeViewport;
    }
    if let Some(index) = browser_source_tree_row_index(control_id) {
        return AssetContentRowDescriptor::BrowserSourceTree { index };
    }
    if control_id == "AssetBrowserReferenceLeftScrollBody" {
        return AssetContentRowDescriptor::BrowserReferenceViewport(
            BrowserAssetReferenceListKind::References,
        );
    }
    if control_id == "AssetBrowserReferenceRightScrollBody" {
        return AssetContentRowDescriptor::BrowserReferenceViewport(
            BrowserAssetReferenceListKind::UsedBy,
        );
    }
    browser_reference_row_index(control_id)
        .map(
            |(list_kind, index)| AssetContentRowDescriptor::BrowserReference {
                list_kind,
                index,
                paints_hover: browser_reference_row_paints_hover(control_id, list_kind),
            },
        )
        .unwrap_or(AssetContentRowDescriptor::Fixed)
}

fn activity_reference_row_paints_hover(
    control_id: &str,
    list_kind: ActivityAssetReferenceListKind,
) -> bool {
    match list_kind {
        ActivityAssetReferenceListKind::References => control_id
            .strip_prefix("AssetsActivityReferenceLeftRowPanel")
            .is_some(),
        ActivityAssetReferenceListKind::UsedBy => control_id
            .strip_prefix("AssetsActivityReferenceRightRowPanel")
            .is_some(),
    }
}

fn browser_reference_row_paints_hover(
    control_id: &str,
    list_kind: BrowserAssetReferenceListKind,
) -> bool {
    match list_kind {
        BrowserAssetReferenceListKind::References => control_id
            .strip_prefix("AssetBrowserReferenceLeftRowPanel")
            .is_some(),
        BrowserAssetReferenceListKind::UsedBy => control_id
            .strip_prefix("AssetBrowserReferenceRightRowPanel")
            .is_some(),
    }
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
        (
            "NameContinuation",
            BrowserThumbnailNodeRole::NameContinuation,
        ),
        ("SelectionMarker", BrowserThumbnailNodeRole::SelectionMarker),
        ("InfoBand", BrowserThumbnailNodeRole::InfoBand),
        ("TypeBadge", BrowserThumbnailNodeRole::TypeBadge),
        ("Visual", BrowserThumbnailNodeRole::Visual),
        ("Card", BrowserThumbnailNodeRole::Card),
        ("Name", BrowserThumbnailNodeRole::Name),
        ("Type", BrowserThumbnailNodeRole::Type),
        ("Meta", BrowserThumbnailNodeRole::Meta),
    ] {
        let Some(number) = suffix.strip_prefix(kind) else {
            continue;
        };
        let index = number.parse::<usize>().ok()?.checked_sub(1)?;
        return Some(BrowserContentNodeIdentity::Thumbnail { index, role });
    }
    None
}
