use crate::ui::workbench::snapshot::{EditorChromeSnapshot, ViewContentKind, ViewTabSnapshot};

use super::empty_state::empty_state_for_tab;
use super::pane_tab_model::PaneTabModel;

/// 调用方提供本抽屉的选择态；构造不会把该标签变成全局焦点。
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

/// 显示关闭入口的内容策略；真正关闭仍须验证实例和宿主生命周期。
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
