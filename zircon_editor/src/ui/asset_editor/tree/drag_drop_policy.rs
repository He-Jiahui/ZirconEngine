use zircon_runtime::ui::template::UiAssetKind;
use zircon_runtime::ui::template::UiNodeDefinitionKind;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiAssetDragDropPolicy;

impl UiAssetDragDropPolicy {
    pub fn allows_tree_edit(asset_kind: UiAssetKind) -> bool {
        !matches!(asset_kind, UiAssetKind::Style)
    }

    pub fn allows_child_drop(parent_kind: UiNodeDefinitionKind) -> bool {
        !matches!(parent_kind, UiNodeDefinitionKind::Slot)
    }
}
