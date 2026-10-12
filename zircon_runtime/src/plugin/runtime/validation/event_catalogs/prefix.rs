/// 判断目录是否落在包的点分子命名空间，避免短包名匹配更长包名。
/// 只建立归属关系，不代替包身份和命名空间格式校验；字节边界同时避免分配临时前缀。
pub(super) fn runtime_plugin_package_event_catalog_has_owner(
    package_id: &str,
    event_catalog_namespace: &str,
) -> bool {
    let namespace = event_catalog_namespace.as_bytes();
    let owner = package_id.as_bytes();
    namespace.len() > owner.len() && namespace.starts_with(owner) && namespace[owner.len()] == b'.'
}

#[cfg(test)]
#[path = "tests/prefix.rs"]
mod tests;
