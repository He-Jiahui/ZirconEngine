use zircon_runtime_interface::resource::{ResourceLocator, ResourceScheme};

/// 富文本图片和图标共用的定位符入口，只接纳引擎受控 scheme；此阶段不解析资源也不授予加载权限。
pub(super) fn controlled_resource_locator(value: &str) -> Option<ResourceLocator> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let locator = if value.contains("://") {
        ResourceLocator::parse(value).ok()?
    } else {
        ResourceLocator::new(ResourceScheme::Res, value, None).ok()?
    };
    matches!(
        locator.scheme(),
        ResourceScheme::Res
            | ResourceScheme::Library
            | ResourceScheme::Package
            | ResourceScheme::Builtin
    )
    .then_some(locator)
}
