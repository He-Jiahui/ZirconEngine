use crate::ui::workbench::snapshot::{EditorChromeSnapshot, ViewContentKind, ViewTabSnapshot};

use super::empty_state::empty_state_for_tab;
use super::pane_tab_model::PaneTabModel;

pub(super) fn pane_tab_model(
    tab: &ViewTabSnapshot,
    active: bool,
    chrome: &EditorChromeSnapshot,
) -> PaneTabModel {
    PaneTabModel {
        instance_id: tab.instance_id.clone(),
        descriptor_id: tab.descriptor_id.clone(),
        title: tab.title.clone(),
        icon_key: tab.icon_key.clone(),
        content_kind: tab.content_kind,
        active,
        closeable: is_closeable_content_kind(tab.content_kind),
        empty_state: empty_state_for_tab(tab, chrome),
    }
}

pub(super) fn is_closeable_content_kind(kind: ViewContentKind) -> bool {
    matches!(
        kind,
        ViewContentKind::PrefabEditor
            | ViewContentKind::AssetBrowser
            | ViewContentKind::UiAssetEditor
            | ViewContentKind::AnimationSequenceEditor
            | ViewContentKind::AnimationGraphEditor
            | ViewContentKind::Placeholder
    )
}
