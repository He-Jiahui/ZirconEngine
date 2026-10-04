mod capabilities;
mod categories;

use capabilities::attach_extra_capabilities;
use categories::assign_category;

use super::IdentifiedBuiltinCatalogDescriptorBuilder;

// 分类前只补齐包类别和额外能力，不在此决定能力的完成状态。
pub(super) fn augment_descriptor(
    (package_id, descriptor): IdentifiedBuiltinCatalogDescriptorBuilder,
) -> IdentifiedBuiltinCatalogDescriptorBuilder {
    let descriptor = assign_category(package_id, descriptor);
    (
        package_id,
        attach_extra_capabilities(package_id, descriptor),
    )
}
