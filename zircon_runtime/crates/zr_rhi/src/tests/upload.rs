use std::sync::Arc;

use crate::{DeviceGeneration, DeviceId, RenderResourceHandleAllocator, TextureCopyRegion};

use super::{BufferUpload, BufferUploadBatch, TextureUpload, TextureUploadBatch};

#[test]
fn batches_share_payload_owners_and_count_only_selected_ranges() {
    let payload: Arc<[u8]> = Arc::from([0_u8; 16]);
    let handles = RenderResourceHandleAllocator::new(DeviceId::new(1), DeviceGeneration::initial());
    let buffer = handles
        .allocate_buffer()
        .expect("test buffer handle allocation must succeed");
    let texture = handles
        .allocate_texture()
        .expect("test texture handle allocation must succeed");
    let buffer_upload = BufferUpload::new(buffer, 4, Arc::clone(&payload), 2..8)
        .expect("test buffer source range must be valid");
    let texture_upload = TextureUpload::new(
        texture,
        TextureCopyRegion::new(1, 1),
        4,
        Arc::clone(&payload),
        8..12,
    )
    .expect("test texture source range must be valid");

    let buffer_batch = BufferUploadBatch::from(buffer_upload);
    let texture_batch = TextureUploadBatch::from(texture_upload);

    assert_eq!(buffer_batch.payload_byte_len(), Some(6));
    assert_eq!(texture_batch.payload_byte_len(), Some(4));
    assert_eq!(Arc::strong_count(&payload), 3);
}
