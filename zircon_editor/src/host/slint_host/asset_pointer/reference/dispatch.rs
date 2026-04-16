use super::route::AssetPointerReferenceRoute;
use crate::host::slint_host::asset_pointer::AssetListPointerState;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AssetReferenceListPointerDispatch {
    pub route: Option<AssetPointerReferenceRoute>,
    pub state: AssetListPointerState,
}
