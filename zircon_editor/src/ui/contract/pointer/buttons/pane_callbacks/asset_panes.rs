#[path = "../../../../retained/contract/pointer/buttons/asset_panes/content.rs"]
mod content;
#[path = "../../../../retained/contract/pointer/buttons/asset_panes/reference.rs"]
mod reference;
#[path = "../../../../retained/contract/pointer/buttons/asset_panes/tree.rs"]
mod tree;

pub(in crate::ui::retained_host::host_contract) use self::content::dispatch_asset_content_button;
pub(in crate::ui::retained_host::host_contract) use self::reference::dispatch_asset_reference_button;
pub(in crate::ui::retained_host::host_contract) use self::tree::dispatch_asset_tree_button;
