//! 导入加载、原型仓库验证和组件展开共用引用语法，避免各阶段对资源片段作不同解释。

use zircon_runtime_interface::ui::{template::parse_component_reference, v2::UiV2AssetError};

/// 导入整份资源时允许搜索其中的组件；具名导入仅开放指定组件，字符串借用原始声明。
pub(crate) enum UiV2WidgetImportReference<'a> {
    WholeAsset(&'a str),
    Component {
        asset_id: &'a str,
        component: &'a str,
    },
}

/// 为共享引用解析失败补上声明资源的身份，使导入链诊断指向发出引用的文档。
pub(crate) fn parse_v2_component_reference<'a>(
    owner_asset_id: &str,
    reference: &'a str,
) -> Result<(&'a str, &'a str), UiV2AssetError> {
    parse_component_reference(reference).map_err(|error| UiV2AssetError::InvalidDocument {
        asset_id: owner_asset_id.to_string(),
        detail: error.to_string(),
    })
}

/// V2 widget imports support both complete component assets and one named component.
/// A named component always passes through the shared reference parser so malformed
/// fragments fail while sources are being loaded instead of becoming a later miss.
pub(crate) fn parse_v2_widget_import_reference<'a>(
    owner_asset_id: &str,
    reference: &'a str,
) -> Result<UiV2WidgetImportReference<'a>, UiV2AssetError> {
    if reference.contains('#') {
        let (asset_id, component) = parse_v2_component_reference(owner_asset_id, reference)?;
        Ok(UiV2WidgetImportReference::Component {
            asset_id,
            component,
        })
    } else {
        Ok(UiV2WidgetImportReference::WholeAsset(reference))
    }
}
