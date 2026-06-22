mod capabilities;
mod categories;

use capabilities::attach_extra_capabilities;
use categories::assign_category;

use super::BuiltinCatalogDescriptorBuilder;

pub(super) fn augment_descriptor(
    descriptor: BuiltinCatalogDescriptorBuilder,
) -> BuiltinCatalogDescriptorBuilder {
    attach_extra_capabilities(assign_category(descriptor))
}
