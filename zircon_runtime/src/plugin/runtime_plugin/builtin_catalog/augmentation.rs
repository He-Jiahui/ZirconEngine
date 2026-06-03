mod capabilities;
mod categories;

use super::super::RuntimePluginDescriptor;
use capabilities::attach_extra_capabilities;
use categories::assign_category;

pub(super) fn augment_descriptor(descriptor: RuntimePluginDescriptor) -> RuntimePluginDescriptor {
    attach_extra_capabilities(assign_category(descriptor))
}
