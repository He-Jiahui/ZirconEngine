use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;
use crate::ui::retained_host::primitives::SharedString;

pub(in crate::ui::retained_host::host_contract) enum PanePointerTarget {
    Hierarchy,
    Welcome,
    Console,
    Inspector,
    BrowserAssetDetails,
    AssetTree(SharedString),
    AssetContent(SharedString),
    AssetReference(SharedString, SharedString),
    TemplateNode(TemplateNodePointerHit),
    ViewportToolbar {
        surface_key: SharedString,
        control_id: Option<SharedString>,
    },
    Viewport(SharedString),
    UiAsset,
    Other,
}
