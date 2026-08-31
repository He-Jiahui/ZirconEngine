use zircon_runtime_interface::resource::{ResourceLocator, ResourceScheme};

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
