use crate::{DeviceGeneration, DeviceId, RenderResourceHandleAllocator, RenderResourceHandleError};

#[test]
fn released_handle_fails_closed_after_slot_reuse() {
    let allocator = RenderResourceHandleAllocator::new(DeviceId::new(7), DeviceGeneration::new(3));
    let released = allocator.allocate_buffer().expect("allocate first buffer");

    allocator.release_buffer(released).expect("release buffer");
    let replacement = allocator.allocate_buffer().expect("reuse buffer slot");

    assert_ne!(released, replacement);
    assert_eq!(
        allocator.validate_buffer(released),
        Err(RenderResourceHandleError::StaleHandle {
            diagnostic_id: released.diagnostic_id(),
        })
    );
    assert!(allocator.validate_buffer(replacement).is_ok());
}

#[test]
fn resource_handle_rejects_other_device_and_generation() {
    let source = RenderResourceHandleAllocator::new(DeviceId::new(7), DeviceGeneration::new(3));
    let other_device =
        RenderResourceHandleAllocator::new(DeviceId::new(8), DeviceGeneration::new(3));
    let next_generation =
        RenderResourceHandleAllocator::new(DeviceId::new(7), DeviceGeneration::new(4));
    let handle = source.allocate_texture().expect("allocate source texture");

    assert_eq!(
        other_device.validate_texture(handle),
        Err(RenderResourceHandleError::WrongDevice {
            expected: DeviceId::new(8),
            actual: DeviceId::new(7),
        })
    );
    assert_eq!(
        next_generation.validate_texture(handle),
        Err(RenderResourceHandleError::WrongGeneration {
            expected: DeviceGeneration::new(4),
            actual: DeviceGeneration::new(3),
        })
    );
}
