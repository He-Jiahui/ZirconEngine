use crate::ui::workbench::asset_content_layout::{
    ACTIVITY_CONTENT_EMPTY_CONTROL_ID, ACTIVITY_CONTENT_FOLDER_PREFIX,
    ACTIVITY_CONTENT_ITEM_PREFIX, ACTIVITY_CONTENT_PANEL_CONTROL_ID, BROWSER_CONTENT_ITEM_PREFIX,
    BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ActivityContentNodeRole {
    Row,
    Badge,
    Type,
    Name,
    Meta,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ActivityContentNodeIdentity {
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
    pub(super) fn is_row(self) -> bool {
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

    pub(super) fn shared_row_index(self, folder_row_count: usize) -> Option<i32> {
        match self {
            Self::Folder { index, .. } => i32::try_from(index).ok(),
            Self::Item { index, .. } => folder_row_count
                .checked_add(index)
                .and_then(|index| i32::try_from(index).ok()),
            Self::ContentPanel | Self::Empty => None,
        }
    }
}

pub(super) fn activity_content_identity(control_id: &str) -> Option<ActivityContentNodeIdentity> {
    let leaf = control_id.rsplit('/').next()?;
    match leaf {
        ACTIVITY_CONTENT_PANEL_CONTROL_ID => {
            return Some(ActivityContentNodeIdentity::ContentPanel);
        }
        ACTIVITY_CONTENT_EMPTY_CONTROL_ID => return Some(ActivityContentNodeIdentity::Empty),
        _ => {}
    }

    parse_indexed_identity(leaf, ACTIVITY_CONTENT_FOLDER_PREFIX)
        .map(|(index, role)| ActivityContentNodeIdentity::Folder { index, role })
        .or_else(|| {
            parse_indexed_identity(leaf, ACTIVITY_CONTENT_ITEM_PREFIX)
                .map(|(index, role)| ActivityContentNodeIdentity::Item { index, role })
        })
}

fn parse_indexed_identity(
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BrowserContentNodeIdentity {
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
pub(super) enum BrowserThumbnailNodeRole {
    Card,
    InfoBand,
    Child,
}

impl BrowserThumbnailNodeRole {
    pub(super) fn paints_hover(self) -> bool {
        matches!(self, Self::Card | Self::InfoBand)
    }
}

pub(super) fn browser_content_identity(control_id: &str) -> Option<BrowserContentNodeIdentity> {
    let leaf = control_id.rsplit('/').next()?;
    match leaf {
        "AssetBrowserAssetTablePanel" => Some(BrowserContentNodeIdentity::TablePanel),
        "WorkbenchAssetBrowserTableHeader" => Some(BrowserContentNodeIdentity::Header),
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
