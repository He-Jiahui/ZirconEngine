use super::route::AssetPointerContentRoute;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum AssetContentListPointerTarget {
    Folder {
        row_index: usize,
        folder_index: usize,
        folder_id: String,
    },
    Item {
        row_index: usize,
        item_index: usize,
        asset_uuid: String,
    },
    ContentSurface,
}

pub(super) fn to_public_route(target: AssetContentListPointerTarget) -> AssetPointerContentRoute {
    match target {
        AssetContentListPointerTarget::Folder {
            row_index,
            folder_index,
            folder_id,
        } => AssetPointerContentRoute::Folder {
            row_index,
            folder_index,
            folder_id,
        },
        AssetContentListPointerTarget::Item {
            row_index,
            item_index,
            asset_uuid,
        } => AssetPointerContentRoute::Item {
            row_index,
            item_index,
            asset_uuid,
        },
        AssetContentListPointerTarget::ContentSurface => AssetPointerContentRoute::ContentSurface,
    }
}

pub(super) fn hovered_row_from_target(
    target: Option<&AssetContentListPointerTarget>,
) -> Option<usize> {
    match target {
        Some(AssetContentListPointerTarget::Folder { row_index, .. })
        | Some(AssetContentListPointerTarget::Item { row_index, .. }) => Some(*row_index),
        Some(AssetContentListPointerTarget::ContentSurface) | None => None,
    }
}
