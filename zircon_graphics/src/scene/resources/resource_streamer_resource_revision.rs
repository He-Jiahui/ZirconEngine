use zircon_resource::ResourceId;

use crate::types::GraphicsError;

use super::resource_streamer::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn resource_revision(&self, id: ResourceId) -> Result<u64, GraphicsError> {
        self.asset_manager
            .resource_manager()
            .registry()
            .get(id)
            .map(|record| record.revision)
            .ok_or_else(|| GraphicsError::Asset(format!("missing resource record {id}")))
    }
}
