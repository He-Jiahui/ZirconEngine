use super::route::AssetPointerTreeRoute;
use crate::ui::retained_host::asset_pointer::AssetListPointerState;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AssetFolderTreePointerDispatch {
    pub route: Option<AssetPointerTreeRoute>,
    pub state: AssetListPointerState,
}
