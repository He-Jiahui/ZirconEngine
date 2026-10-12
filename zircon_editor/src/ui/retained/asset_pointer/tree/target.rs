use super::route::AssetPointerTreeRoute;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum AssetFolderTreePointerTarget {
    Folder { row_index: usize, folder_id: String },
    TreeSurface,
}

pub(super) fn to_public_route(target: AssetFolderTreePointerTarget) -> AssetPointerTreeRoute {
    match target {
        AssetFolderTreePointerTarget::Folder {
            row_index,
            folder_id,
        } => AssetPointerTreeRoute::Folder {
            row_index,
            folder_id,
        },
        AssetFolderTreePointerTarget::TreeSurface => AssetPointerTreeRoute::TreeSurface,
    }
}
