mod services;
mod systems;

use super::super::BuiltinCatalogDescriptorBuilder;
use services::{classify_runtime_service_descriptor, is_runtime_service_descriptor};
use systems::{classify_runtime_system_descriptor, is_runtime_system_descriptor};

pub(super) fn is_runtime_foundation_descriptor(package_id: &str) -> bool {
    is_runtime_service_descriptor(package_id) || is_runtime_system_descriptor(package_id)
}

// 运行时基础包按服务或系统规则分类，避免普通包的兜底状态覆盖它们。
pub(super) fn classify_runtime_foundation_descriptor(
    package_id: &str,
    descriptor: BuiltinCatalogDescriptorBuilder,
) -> BuiltinCatalogDescriptorBuilder {
    if is_runtime_service_descriptor(package_id) {
        return classify_runtime_service_descriptor(package_id, descriptor);
    }
    if is_runtime_system_descriptor(package_id) {
        return classify_runtime_system_descriptor(package_id, descriptor);
    }
    descriptor
}
