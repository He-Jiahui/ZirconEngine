use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    ActivityWindowTemplateSpec, PreferredHost, ViewDescriptor, ViewDescriptorId, ViewKind,
};

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
    .with_activity_window_template(ActivityWindowTemplateSpec::new(
        "res://ui/editor/windows/asset_window.zui",
    ))
    .with_icon_key("asset-browser")
}
