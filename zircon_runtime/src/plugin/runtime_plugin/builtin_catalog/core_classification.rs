mod content;
mod runtime;

use super::BuiltinCatalogDescriptorBuilder;
use content::{classify_content_tool_descriptor, is_content_tool_descriptor};
use runtime::{classify_runtime_foundation_descriptor, is_runtime_foundation_descriptor};

pub(super) fn is_core_descriptor(package_id: &str) -> bool {
    is_runtime_foundation_descriptor(package_id) || is_content_tool_descriptor(package_id)
}

pub(super) fn classify_core_descriptor(
    package_id: &str,
    descriptor: BuiltinCatalogDescriptorBuilder,
) -> BuiltinCatalogDescriptorBuilder {
    if is_runtime_foundation_descriptor(package_id) {
        return classify_runtime_foundation_descriptor(package_id, descriptor);
    }
    if is_content_tool_descriptor(package_id) {
        return classify_content_tool_descriptor(package_id, descriptor);
    }
    descriptor
}
