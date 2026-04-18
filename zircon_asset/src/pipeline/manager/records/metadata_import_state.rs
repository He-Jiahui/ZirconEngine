use zircon_resource::ResourceState;

use crate::AssetMetadata;

pub(super) fn metadata_import_state(metadata: &AssetMetadata) -> ResourceState {
    if metadata.artifact_locator().is_some() {
        ResourceState::Ready
    } else {
        ResourceState::Pending
    }
}
