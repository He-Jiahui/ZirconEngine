use super::route::AssetPointerReferenceRoute;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum AssetReferenceListPointerTarget {
    Item {
        row_index: usize,
        asset_uuid: String,
    },
    ListSurface,
}

pub(super) fn to_public_route(
    target: AssetReferenceListPointerTarget,
) -> AssetPointerReferenceRoute {
    match target {
        AssetReferenceListPointerTarget::Item {
            row_index,
            asset_uuid,
        } => AssetPointerReferenceRoute::Item {
            row_index,
            asset_uuid,
        },
        AssetReferenceListPointerTarget::ListSurface => AssetPointerReferenceRoute::ListSurface,
    }
}

pub(super) fn hovered_row_from_target(
    target: Option<&AssetReferenceListPointerTarget>,
) -> Option<usize> {
    match target {
        Some(AssetReferenceListPointerTarget::Item { row_index, .. }) => Some(*row_index),
        Some(AssetReferenceListPointerTarget::ListSurface) | None => None,
    }
}
