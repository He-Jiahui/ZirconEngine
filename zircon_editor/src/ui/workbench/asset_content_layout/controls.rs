//! 资产模板控件的稳定命名与行号协议；生成、分类和命中需共同维护。
pub(crate) const ACTIVITY_CONTENT_PANEL_CONTROL_ID: &str = "AssetsActivityContentPanel";
pub(crate) const ACTIVITY_CONTENT_EMPTY_CONTROL_ID: &str = "AssetsActivityContentEmptyText";
pub(crate) const ACTIVITY_CONTENT_FOLDER_PREFIX: &str = "AssetsActivityContentFolder";
pub(crate) const ACTIVITY_CONTENT_ITEM_PREFIX: &str = "AssetsActivityContentItem";
pub(crate) const ACTIVITY_TREE_ROW_CONTROL_ID: &str = "AssetsActivityTreeRowPanel";
pub(crate) const BROWSER_CONTENT_TABLE_CONTROL_ID: &str = "AssetBrowserAssetTablePanel";
pub(crate) const BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID: &str = "AssetBrowserThumbGridPanel";
pub(crate) const BROWSER_CONTENT_TABLE_HEADER_CONTROL_ID: &str = "WorkbenchAssetBrowserTableHeader";
pub(crate) const BROWSER_CONTENT_PREVIEW_CONTROL_ID: &str = "AssetBrowserContentPreviewCard";
pub(crate) const BROWSER_CONTENT_ITEM_PREFIX: &str = "WorkbenchAssetBrowserAssetRow";
pub(crate) const BROWSER_CONTENT_TABLE_HEADER_HEIGHT: f32 = 24.0;
pub(crate) const BROWSER_CONTENT_LIST_ROW_HEIGHT: f32 = 28.0;
const BROWSER_SOURCE_TREE_ROOT_CONTROL_ID: &str = "AssetBrowserSourcesRowPanel";
const BROWSER_SOURCE_TREE_DYNAMIC_ROW_PREFIX: &str = "AssetBrowserSourcesTreeRow";
const BROWSER_SOURCE_TREE_DYNAMIC_ROW_SUFFIX: &str = "/AssetBrowserSourcesRowPanel";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum BrowserAssetReferenceListKind {
    References,
    UsedBy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ActivityAssetReferenceListKind {
    References,
    UsedBy,
}

/// 把来源树根与动态控件编号转为零基行身份；未知或零号动态行不接受。
pub(crate) fn browser_source_tree_row_index(control_id: &str) -> Option<usize> {
    if control_id == BROWSER_SOURCE_TREE_ROOT_CONTROL_ID {
        return Some(0);
    }
    let row_number = control_id
        .strip_prefix(BROWSER_SOURCE_TREE_DYNAMIC_ROW_PREFIX)?
        .strip_suffix(BROWSER_SOURCE_TREE_DYNAMIC_ROW_SUFFIX)?;
    row_number.parse::<usize>().ok()?.checked_sub(1)
}

/// 引用/被引用列表的各文本子控件归并到同一个零基行。
pub(crate) fn browser_reference_row_index(
    control_id: &str,
) -> Option<(BrowserAssetReferenceListKind, usize)> {
    for (list_kind, prefixes) in [
        (
            BrowserAssetReferenceListKind::References,
            [
                "AssetBrowserReferenceLeftRowPanel",
                "AssetBrowserReferenceLeftRowNameText",
                "AssetBrowserReferenceLeftRowLocatorText",
                "AssetBrowserReferenceLeftRowKindText",
            ],
        ),
        (
            BrowserAssetReferenceListKind::UsedBy,
            [
                "AssetBrowserReferenceRightRowPanel",
                "AssetBrowserReferenceRightRowNameText",
                "AssetBrowserReferenceRightRowLocatorText",
                "AssetBrowserReferenceRightRowKindText",
            ],
        ),
    ] {
        for prefix in prefixes {
            if let Some(index) = control_id
                .strip_prefix(prefix)
                .and_then(|suffix| suffix.parse::<usize>().ok())
                .and_then(|index| index.checked_sub(1))
            {
                return Some((list_kind, index));
            }
        }
    }
    None
}

/// 活动区引用列表沿与生成端一致的一基控件编号还原行身份。
pub(crate) fn activity_reference_row_index(
    control_id: &str,
) -> Option<(ActivityAssetReferenceListKind, usize)> {
    for (list_kind, prefixes) in [
        (
            ActivityAssetReferenceListKind::References,
            [
                "AssetsActivityReferenceLeftRowPanel",
                "AssetsActivityReferenceLeftRowNameText",
                "AssetsActivityReferenceLeftRowLocatorText",
                "AssetsActivityReferenceLeftRowKindText",
            ],
        ),
        (
            ActivityAssetReferenceListKind::UsedBy,
            [
                "AssetsActivityReferenceRightRowPanel",
                "AssetsActivityReferenceRightRowNameText",
                "AssetsActivityReferenceRightRowLocatorText",
                "AssetsActivityReferenceRightRowKindText",
            ],
        ),
    ] {
        for prefix in prefixes {
            if let Some(index) = control_id
                .strip_prefix(prefix)
                .and_then(|suffix| suffix.parse::<usize>().ok())
                .and_then(|index| index.checked_sub(1))
            {
                return Some((list_kind, index));
            }
        }
    }
    None
}
