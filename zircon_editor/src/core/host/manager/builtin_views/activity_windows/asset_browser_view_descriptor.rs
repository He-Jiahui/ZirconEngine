use crate::default_constraints_for_content;
use crate::view::{PreferredHost, ViewDescriptor, ViewDescriptorId, ViewKind};
use crate::ViewContentKind;

pub(super) fn asset_browser_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.asset_browser"),
        ViewKind::ActivityWindow,
        "Asset Browser",
    )
    .with_preferred_host(PreferredHost::ExclusiveMainPage)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::AssetBrowser,
    ))
    .with_icon_key("asset-browser")
}
