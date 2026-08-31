use super::layout::{
    texture_row_alignment, DiagnosticTextureMipChainReadbackLayout, DiagnosticTextureReadbackLayout,
};
use super::*;
use zr_rhi::{
    DeviceGeneration, DeviceId, DiagnosticReadbackAdmission, DiagnosticReadbackBudget,
    DiagnosticReadbackKind, DiagnosticReadbackReceipt, DiagnosticReadbackTerminal,
    DiagnosticReadbackTracker, RenderResourceHandleAllocator, TextureCopyRegion,
};

fn receipt(budget: DiagnosticReadbackBudget) -> DiagnosticReadbackReceipt {
    let mut tracker =
        DiagnosticReadbackTracker::new(DeviceId::new(17), DeviceGeneration::initial(), budget);
    tracker.begin_frame(1).unwrap();
    let request = tracker.admit(DiagnosticReadbackKind::Buffer, 4).unwrap();
    tracker
        .terminalize(request, DiagnosticReadbackTerminal::Succeeded)
        .unwrap()
}

fn distinct_receipts(
    budget: DiagnosticReadbackBudget,
) -> (DiagnosticReadbackReceipt, DiagnosticReadbackReceipt) {
    let mut tracker =
        DiagnosticReadbackTracker::new(DeviceId::new(17), DeviceGeneration::initial(), budget);
    tracker.begin_frame(1).unwrap();
    let first_request = tracker.admit(DiagnosticReadbackKind::Buffer, 4).unwrap();
    let first = tracker
        .terminalize(first_request, DiagnosticReadbackTerminal::Succeeded)
        .unwrap();
    tracker.terminalize_active_frame(DiagnosticReadbackTerminal::Cancelled);
    tracker.begin_frame(2).unwrap();
    let second_request = tracker.admit(DiagnosticReadbackKind::Buffer, 4).unwrap();
    let second = tracker
        .terminalize(second_request, DiagnosticReadbackTerminal::Succeeded)
        .unwrap();
    (first, second)
}

#[test]
fn delivery_ring_limits_retained_payload_bytes_as_well_as_receipt_count() {
    let budget = DiagnosticReadbackBudget::new(4, 4, 4, 4, 4, 2);
    let mut service =
        WgpuDiagnosticReadbackService::new(DeviceId::new(17), DeviceGeneration::initial(), budget);

    service.push_delivery(WgpuDiagnosticReadbackDelivery {
        receipt: receipt(budget),
        bytes: Some(vec![1; 4]),
    });
    service.push_delivery(WgpuDiagnosticReadbackDelivery {
        receipt: receipt(budget),
        bytes: Some(vec![2; 4]),
    });

    assert_eq!(service.retained_delivery_bytes(), 4);
    assert_eq!(service.dropped_delivery_count(), 1);
    assert_eq!(service.take_delivery().unwrap().bytes(), Some(&[2; 4][..]));
    assert_eq!(service.retained_delivery_bytes(), 0);
}

#[test]
fn request_qualified_delivery_does_not_consume_another_request_at_the_front() {
    let budget = DiagnosticReadbackBudget::new(4, 4, 4, 4, 8, 4);
    let mut service =
        WgpuDiagnosticReadbackService::new(DeviceId::new(17), DeviceGeneration::initial(), budget);
    let (first, second) = distinct_receipts(budget);
    service.push_delivery(WgpuDiagnosticReadbackDelivery {
        receipt: first,
        bytes: Some(vec![1; 4]),
    });
    service.push_delivery(WgpuDiagnosticReadbackDelivery {
        receipt: second,
        bytes: Some(vec![2; 4]),
    });

    assert!(service.take_delivery_for(second.request()).is_none());
    assert_eq!(service.retained_delivery_bytes(), 8);
    assert_eq!(
        service.take_delivery_for(first.request()).unwrap().bytes(),
        Some(&[1; 4][..])
    );
    assert_eq!(
        service.take_delivery_for(second.request()).unwrap().bytes(),
        Some(&[2; 4][..])
    );
}

#[test]
fn batch_delivery_drain_preserves_order_and_releases_retained_bytes() {
    let budget = DiagnosticReadbackBudget::new(4, 4, 4, 4, 8, 4);
    let mut service =
        WgpuDiagnosticReadbackService::new(DeviceId::new(17), DeviceGeneration::initial(), budget);
    let (first, second) = distinct_receipts(budget);
    service.push_delivery(WgpuDiagnosticReadbackDelivery {
        receipt: first,
        bytes: Some(vec![1; 4]),
    });
    service.push_delivery(WgpuDiagnosticReadbackDelivery {
        receipt: second,
        bytes: Some(vec![2; 4]),
    });
    let mut deliveries = Vec::new();

    assert_eq!(service.append_deliveries(&mut deliveries), 2);
    assert_eq!(
        deliveries
            .iter()
            .map(|delivery| delivery.receipt().request())
            .collect::<Vec<_>>(),
        vec![first.request(), second.request()]
    );
    assert_eq!(service.retained_delivery_bytes(), 0);
    assert_eq!(service.metrics_snapshot().drained_delivery_count(), 2);
    assert_eq!(service.metrics_snapshot().drained_delivery_bytes(), 8);
}

#[test]
fn abandoning_a_rejected_frame_allows_the_next_diagnostic_frame_to_begin() {
    let budget = DiagnosticReadbackBudget::new(4, 4, 4, 4, 4, 4);
    let mut service =
        WgpuDiagnosticReadbackService::new(DeviceId::new(17), DeviceGeneration::initial(), budget);
    let handles =
        RenderResourceHandleAllocator::new(DeviceId::new(17), DeviceGeneration::initial());
    let buffer = handles.allocate_buffer().unwrap();

    service.begin_frame(1).unwrap();
    assert!(matches!(
        service.admit_buffer(buffer, 0, 8),
        Ok(DiagnosticReadbackAdmission::Rejected(_))
    ));
    service.abandon_active_batch(DiagnosticReadbackTerminal::OverBudget);

    assert!(service.begin_frame(2).is_ok());
}

#[test]
fn texture_layout_pads_native_rows_but_delivers_tightly_packed_payload() {
    let layout = DiagnosticTextureReadbackLayout::new(12, 2).unwrap();
    assert_eq!(
        layout.padded_bytes_per_row(),
        wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
    );
    assert_eq!(layout.staging_byte_len(), 512);

    let mut mapped = vec![0; 512];
    mapped[..12].copy_from_slice(&[1; 12]);
    mapped[256..268].copy_from_slice(&[2; 12]);

    assert_eq!(
        layout.unpack(&mapped),
        Some([vec![1; 12], vec![2; 12]].concat())
    );
}

#[test]
fn texture_layout_accepts_the_largest_representable_aligned_row() {
    let row_bytes = u64::from(u32::MAX) + 1 - texture_row_alignment();
    let layout = DiagnosticTextureReadbackLayout::new(row_bytes, u32::MAX).unwrap();

    assert_eq!(u64::from(layout.padded_bytes_per_row()), row_bytes);
    assert_eq!(layout.staging_byte_len(), row_bytes * u64::from(u32::MAX));
}

#[test]
fn texture_layout_rejects_zero_narrowing_alignment_and_length_overflow() {
    assert!(DiagnosticTextureReadbackLayout::new(0, 1).is_none());
    assert!(DiagnosticTextureReadbackLayout::new(1, 0).is_none());
    assert!(DiagnosticTextureReadbackLayout::new(u64::from(u32::MAX), 1).is_none());
    assert!(DiagnosticTextureReadbackLayout::new(u64::MAX, 1).is_none());
    assert!(DiagnosticTextureReadbackLayout::new(1_u64 << 63, 2).is_none());
}

#[test]
fn texture_mip_chain_layout_unpacks_subresources_in_mip_order() {
    let mip_zero = DiagnosticTextureReadbackLayout::new(8, 2).unwrap();
    let mip_one = DiagnosticTextureReadbackLayout::new(8, 1).unwrap();
    let chain = DiagnosticTextureMipChainReadbackLayout::new([
        (TextureCopyRegion::new(1, 2), mip_zero),
        (TextureCopyRegion::new(1, 1).with_mip_level(1), mip_one),
    ])
    .unwrap();
    let mut staging = vec![0_u8; chain.staging_byte_len() as usize];
    staging[0..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
    staging[256..264].copy_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]);
    staging[512..520].copy_from_slice(&[17, 18, 19, 20, 21, 22, 23, 24]);

    assert_eq!(chain.subresources()[0].staging_offset(), 0);
    assert_eq!(chain.subresources()[1].staging_offset(), 512);
    assert_eq!(
        chain.unpack(&staging).unwrap(),
        (1_u8..=24).collect::<Vec<_>>()
    );
}

#[test]
fn mixed_request_alignment_is_charged_before_texture_admission() {
    let budget = DiagnosticReadbackBudget::new(4, 4, 512, 260, 512, 4);
    let mut service =
        WgpuDiagnosticReadbackService::new(DeviceId::new(17), DeviceGeneration::initial(), budget);
    let handles =
        RenderResourceHandleAllocator::new(DeviceId::new(17), DeviceGeneration::initial());
    let buffer = handles.allocate_buffer().unwrap();
    let texture = handles.allocate_texture().unwrap();
    service.begin_frame(1).unwrap();
    assert!(matches!(
        service.admit_buffer(buffer, 0, 4),
        Ok(DiagnosticReadbackAdmission::Admitted(_))
    ));

    let rejected = service
        .admit_texture(
            texture,
            TextureCopyRegion::new(3, 1),
            DiagnosticTextureReadbackLayout::new(12, 1).unwrap(),
        )
        .unwrap();
    assert!(matches!(
        rejected,
        DiagnosticReadbackAdmission::Rejected(receipt)
            if receipt.kind() == DiagnosticReadbackKind::Texture
                && receipt.byte_len() == 508
                && receipt.terminal() == DiagnosticReadbackTerminal::OverBudget
    ));
}

#[test]
fn native_buffer_sources_join_the_submission_bound_diagnostic_batch() {
    let request_source = include_str!("request.rs");
    let service_source = include_str!("service.rs");
    let device_source = include_str!("../../device/diagnostics.rs");

    assert!(request_source.contains("struct DiagnosticNativeBufferReadbackRequest"));
    assert!(request_source.contains("NativeBuffer(DiagnosticNativeBufferReadbackRequest)"));
    assert!(service_source.contains("fn admit_native_buffer("));
    assert!(device_source.contains("enqueue_native_diagnostic_buffer_readback"));
    assert!(device_source.contains("encoder.copy_buffer_to_buffer("));

    let native_prepare = device_source
        .split("pub fn prepare_native_diagnostic_readback_frame")
        .nth(1)
        .and_then(|source| source.split("pub fn abort_prepared_native").next())
        .expect("native diagnostic preparation body");
    assert!(!native_prepare.contains("begin_packet("));
    assert!(!native_prepare.contains("queue.submit"));
    assert!(!native_prepare.contains(".poll("));
}

#[test]
fn native_rgba16float_textures_join_the_same_submission_bound_batch() {
    let request_source = include_str!("request.rs");
    let service_source = include_str!("service.rs");
    let device_source = include_str!("../../device/diagnostics.rs");

    assert!(request_source.contains("struct DiagnosticNativeTextureReadbackRequest"));
    assert!(request_source.contains("NativeTexture(DiagnosticNativeTextureReadbackRequest)"));
    assert!(service_source.contains("fn admit_native_texture("));
    assert!(device_source.contains("enqueue_native_diagnostic_texture_rgba16float_readback"));
    assert!(device_source.contains("ensure_native_rgba16float_texture_readback"));
    assert!(device_source.contains("wgpu::TextureFormat::Rgba16Float"));
    assert!(device_source.contains("encoder.copy_texture_to_buffer("));

    let native_prepare = device_source
        .split("pub fn prepare_native_diagnostic_readback_frame")
        .nth(1)
        .and_then(|source| source.split("pub fn abort_prepared_native").next())
        .expect("native diagnostic preparation body");
    assert!(!native_prepare.contains("queue.submit"));
    assert!(!native_prepare.contains(".poll("));
}

#[test]
fn native_rgba16float_mip_chains_use_one_direct_staging_request() {
    let request_source = include_str!("request.rs");
    let service_source = include_str!("service.rs");
    let device_source = include_str!("../../device/diagnostics.rs");

    assert!(request_source.contains("struct DiagnosticNativeTextureMipChainReadbackRequest"));
    assert!(request_source
        .contains("NativeTextureMipChain(DiagnosticNativeTextureMipChainReadbackRequest)",));
    assert!(service_source.contains("fn admit_native_texture_mip_chain("));
    assert!(
        device_source.contains("enqueue_native_diagnostic_texture_rgba16float_mip_chain_readback",)
    );
    assert!(device_source.contains("source_request.layout().subresources()"));
    assert!(device_source.contains("request.staging_offset()"));

    let native_prepare = device_source
        .split("pub fn prepare_native_diagnostic_readback_frame")
        .nth(1)
        .and_then(|source| source.split("pub fn abort_prepared_native").next())
        .expect("native diagnostic preparation body");
    assert_eq!(
        native_prepare
            .matches("let staging = self.device.create_buffer")
            .count(),
        1
    );
    assert!(!native_prepare.contains("queue.submit"));
    assert!(!native_prepare.contains(".poll("));
}

#[test]
fn native_pick_texels_join_the_submission_bound_batch_without_a_second_poll_owner() {
    let device_source = include_str!("../../device/diagnostics.rs");

    assert!(device_source.contains("enqueue_native_diagnostic_texture_r32_uint_texel_readback"));
    assert!(device_source.contains("enqueue_native_diagnostic_texture_rgba32float_texel_readback"));
    assert!(device_source.contains("wgpu::TextureFormat::R32Uint"));
    assert!(device_source.contains("wgpu::TextureFormat::Rgba32Float"));
    assert!(device_source.contains("TextureCopyRegion::new(1, 1).with_origin("));

    let native_prepare = device_source
        .split("pub fn prepare_native_diagnostic_readback_frame")
        .nth(1)
        .and_then(|source| source.split("pub fn abort_prepared_native").next())
        .expect("native diagnostic preparation body");
    assert!(!native_prepare.contains("queue.submit"));
    assert!(!native_prepare.contains(".poll("));
}
