mod manifest;
mod rows;

use super::BuiltinCatalogDescriptorBuilder;
use manifest::net_feature;
use rows::NET_FEATURE_ROWS;

pub(super) fn attach_net_features(
    descriptor: BuiltinCatalogDescriptorBuilder,
) -> BuiltinCatalogDescriptorBuilder {
    NET_FEATURE_ROWS.iter().fold(descriptor, |descriptor, row| {
        descriptor.with_optional_feature(net_feature(row))
    })
}
