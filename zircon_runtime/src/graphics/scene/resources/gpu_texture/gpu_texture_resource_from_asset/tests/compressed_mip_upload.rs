use super::*;
use crate::asset::{TextureUploadCompressionFamily, TextureUploadPlan};

#[test]
fn dds_compressed_mip_uploads_use_layer_major_source_offsets() {
    let plan = TextureUploadPlan {
        format: "dds/ati2".to_string(),
        compression: TextureUploadCompressionFamily::Bc,
        data_offset: 128,
        data_length: None,
        block_width: 4,
        block_height: 4,
        block_depth: 1,
        bytes_per_block: 16,
        subresources: Vec::new(),
    };

    let uploads = dds_compressed_mip_uploads(8, 4, 3, 2, &plan)
        .expect("valid BC5 DDS mip layout should fit in address space");

    assert_eq!(
        uploads,
        vec![
            CompressedMipUpload::new(0, 0, 8, 4, 128, 32, 32, 1),
            CompressedMipUpload::new(1, 0, 4, 2, 160, 16, 16, 1),
            CompressedMipUpload::new(2, 0, 2, 1, 176, 16, 16, 1),
            CompressedMipUpload::new(0, 1, 8, 4, 192, 32, 32, 1),
            CompressedMipUpload::new(1, 1, 4, 2, 224, 16, 16, 1),
            CompressedMipUpload::new(2, 1, 2, 1, 240, 16, 16, 1),
        ]
    );
}
