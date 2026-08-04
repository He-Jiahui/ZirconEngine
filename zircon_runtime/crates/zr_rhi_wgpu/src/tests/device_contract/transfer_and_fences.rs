use super::*;

#[test]
fn deterministic_rhi_contract_fence_queries_reject_unissued_fence_values() {
    let device = DeterministicRhiContractDevice::new_headless();

    assert_eq!(
        device.is_fence_complete(FenceValue(0)).unwrap_err(),
        zr_rhi::RhiError::UnknownFence(0)
    );
    assert_eq!(
        device.is_fence_complete(FenceValue(7)).unwrap_err(),
        zr_rhi::RhiError::UnknownFence(7)
    );

    let command_list = device
        .create_command_list(RenderQueueClass::Copy, "empty-copy")
        .unwrap();
    let fence = device.submit(command_list).unwrap();
    assert!(device.is_fence_complete(fence).unwrap());
    assert_eq!(
        device
            .is_fence_complete(FenceValue(fence.0 + 1))
            .unwrap_err(),
        zr_rhi::RhiError::UnknownFence(fence.0 + 1)
    );
}

#[test]
fn deterministic_rhi_contract_write_copy_and_read_buffer_preserves_bytes() {
    let device = DeterministicRhiContractDevice::new_headless();
    let upload = device
        .create_buffer(&BufferDesc::new(
            "upload",
            16,
            BufferUsage::STAGING_WRITE | BufferUsage::COPY_SRC,
        ))
        .unwrap();
    let gpu_buffer = device
        .create_buffer(&BufferDesc::new(
            "gpu-buffer",
            16,
            BufferUsage::COPY_DST | BufferUsage::STAGING_READ,
        ))
        .unwrap();

    device
        .write_buffer(upload, 4, &[10, 20, 30, 40, 50, 60])
        .unwrap();

    let mut command_list = device
        .create_command_list(RenderQueueClass::Copy, "upload-copy")
        .unwrap();
    command_list.copy_buffer_to_buffer(upload, gpu_buffer, 4, 2, 6);
    let fence = device.submit(command_list).unwrap();
    assert!(device.is_fence_complete(fence).unwrap());

    assert_eq!(
        device.read_buffer(gpu_buffer, 0, 10).unwrap(),
        vec![0, 0, 10, 20, 30, 40, 50, 60, 0, 0]
    );
}

#[test]
fn deterministic_rhi_contract_overlapping_self_copy_preserves_memmove_semantics() {
    let device = DeterministicRhiContractDevice::new_headless();
    let buffer = device
        .create_buffer(&BufferDesc::new(
            "self-copy",
            8,
            BufferUsage::STAGING_WRITE
                | BufferUsage::STAGING_READ
                | BufferUsage::COPY_SRC
                | BufferUsage::COPY_DST,
        ))
        .unwrap();
    device
        .write_buffer(buffer, 0, &[1, 2, 3, 4, 5, 6, 7, 8])
        .unwrap();

    let mut command_list = device
        .create_command_list(RenderQueueClass::Copy, "overlapping-self-copy")
        .unwrap();
    command_list.copy_buffer_to_buffer(buffer, buffer, 0, 2, 6);
    device.submit(command_list).unwrap();

    assert_eq!(
        device.read_buffer(buffer, 0, 8).unwrap(),
        vec![1, 2, 1, 2, 3, 4, 5, 6]
    );
}

#[test]
fn deterministic_rhi_contract_write_buffer_validates_usage_and_range() {
    let device = DeterministicRhiContractDevice::new_headless();
    let read_only = device
        .create_buffer(&BufferDesc::new("read-only", 8, BufferUsage::STAGING_READ))
        .unwrap();

    assert_eq!(
        device.write_buffer(read_only, 0, &[1, 2, 3]).unwrap_err(),
        zr_rhi::RhiError::InvalidBufferUsage {
            buffer: read_only.raw(),
            required: BufferUsage::STAGING_WRITE,
            actual: BufferUsage::STAGING_READ,
        }
    );

    let upload = device
        .create_buffer(&BufferDesc::new("upload", 8, BufferUsage::STAGING_WRITE))
        .unwrap();
    assert_eq!(
        device.write_buffer(upload, 6, &[1, 2, 3]).unwrap_err(),
        zr_rhi::RhiError::WriteOutOfRange {
            buffer: upload.raw(),
            offset: 6,
            size: 3,
        }
    );
}

#[test]
fn deterministic_rhi_contract_read_texture_validates_usage() {
    let device = DeterministicRhiContractDevice::new_headless();
    let write_only = device
        .create_texture(&TextureDesc::new(
            "write-only-texture",
            2,
            2,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::COPY_DST,
        ))
        .unwrap();

    assert_eq!(
        device.read_texture(write_only).unwrap_err(),
        zr_rhi::RhiError::InvalidTextureUsage {
            texture: write_only.raw(),
            required: TextureUsage::COPY_SRC,
            actual: TextureUsage::COPY_DST,
        }
    );
}

#[test]
fn deterministic_rhi_contract_read_buffer_validates_usage_and_range() {
    let device = DeterministicRhiContractDevice::new_headless();
    let non_readback = device
        .create_buffer(&BufferDesc::new("non-readback", 8, BufferUsage::COPY_DST))
        .unwrap();

    assert_eq!(
        device.read_buffer(non_readback, 0, 4).unwrap_err(),
        zr_rhi::RhiError::InvalidBufferUsage {
            buffer: non_readback.raw(),
            required: BufferUsage::STAGING_READ,
            actual: BufferUsage::COPY_DST,
        }
    );

    let readback = device
        .create_buffer(&BufferDesc::new("readback", 8, BufferUsage::STAGING_READ))
        .unwrap();
    assert_eq!(
        device.read_buffer(readback, 6, 3).unwrap_err(),
        zr_rhi::RhiError::ReadbackOutOfRange {
            buffer: readback.raw(),
            offset: 6,
            size: 3,
        }
    );
}
