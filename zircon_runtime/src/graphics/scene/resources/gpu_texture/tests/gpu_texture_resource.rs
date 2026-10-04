use super::{rgba8_mip_streaming_upload_bytes, GpuTextureResource};

#[test]
fn mip_streaming_upload_bytes_excludes_mips_copied_from_prior_residency() {
    assert_eq!(
        rgba8_mip_streaming_upload_bytes(8, 4, 2, 4, 2..4, 0..4),
        320,
        "only mip zero and one are reuploaded while the resident tail is copied on-GPU"
    );
    assert_eq!(
        rgba8_mip_streaming_upload_bytes(8, 4, 2, 4, 0..4, 2..4),
        0,
        "eviction recreates the physical tail solely through GPU copies"
    );
}

#[test]
fn mip_streaming_resident_bytes_tracks_the_physical_tail_range() {
    assert_eq!(
        GpuTextureResource::rgba8_mip_chain_bytes(8, 4, 2, 2..4),
        40,
        "the physical tail contains only source levels two and three"
    );
    assert_eq!(
        GpuTextureResource::rgba8_mip_chain_bytes(8, 4, 2, 0..4),
        340,
        "a fully resident chain accounts for every source mip"
    );
}
