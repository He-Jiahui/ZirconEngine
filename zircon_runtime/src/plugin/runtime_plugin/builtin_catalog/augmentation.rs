mod capabilities;
mod categories;

use capabilities::attach_extra_capabilities;
use categories::assign_category;

use super::IdentifiedBuiltinCatalogDescriptorBuilder;

pub(super) fn augment_descriptor(
    (package_id, descriptor): IdentifiedBuiltinCatalogDescriptorBuilder,
) -> IdentifiedBuiltinCatalogDescriptorBuilder {
    let descriptor = assign_category(package_id, descriptor);
    (
        package_id,
        attach_extra_capabilities(package_id, descriptor),
    )
}
