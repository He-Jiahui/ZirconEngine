use super::route::AssetPointerContentRoute;
use crate::host::slint_host::asset_pointer::AssetListPointerState;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AssetContentListPointerDispatch {
    pub route: Option<AssetPointerContentRoute>,
    pub state: AssetListPointerState,
}
