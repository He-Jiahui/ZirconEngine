mod capabilities;
mod media;
mod model;
mod pipeline;

use super::BuiltinCatalogDescriptorBuilder;
use media::{classify_media_importer_descriptor, is_media_importer_descriptor};
use model::{classify_model_importer_descriptor, is_model_importer_descriptor};
use pipeline::{classify_pipeline_importer_descriptor, is_pipeline_importer_descriptor};

pub(super) fn is_importer_descriptor(package_id: &str) -> bool {
    is_model_importer_descriptor(package_id)
        || is_media_importer_descriptor(package_id)
        || is_pipeline_importer_descriptor(package_id)
}

pub(super) fn classify_importer_descriptor(
    package_id: &str,
    descriptor: BuiltinCatalogDescriptorBuilder,
) -> BuiltinCatalogDescriptorBuilder {
    if is_model_importer_descriptor(package_id) {
        return classify_model_importer_descriptor(package_id, descriptor);
    }
    if is_media_importer_descriptor(package_id) {
        return classify_media_importer_descriptor(package_id, descriptor);
    }
    if is_pipeline_importer_descriptor(package_id) {
        return classify_pipeline_importer_descriptor(package_id, descriptor);
    }
    descriptor
}
