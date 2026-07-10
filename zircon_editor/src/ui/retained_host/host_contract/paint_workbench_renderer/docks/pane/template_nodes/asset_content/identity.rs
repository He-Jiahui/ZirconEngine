use crate::ui::workbench::asset_content_layout::{
    ACTIVITY_CONTENT_EMPTY_CONTROL_ID, ACTIVITY_CONTENT_FOLDER_PREFIX,
    ACTIVITY_CONTENT_ITEM_PREFIX, ACTIVITY_CONTENT_PANEL_CONTROL_ID,
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
            return Some(ActivityContentNodeIdentity::ContentPanel)
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
