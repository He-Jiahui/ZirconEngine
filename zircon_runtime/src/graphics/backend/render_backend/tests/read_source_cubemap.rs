use crate::core::framework::render::{
    encode_rgba16f_texels, source_cubemap_mip_count, source_cubemap_mip_size,
    source_cubemap_sample_count, CubemapFace,
};
use zr_rhi::DiagnosticReadbackBudget;

use super::{SourceCubemapFaceReadbackLayout, SourceCubemapWgpuPendingReadback};

#[test]
fn source_cubemap_mip_chain_copies_directly_into_owner_staging() {
    let source = include_str!("../read_source_cubemap.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("source cubemap production owner");
    let request = source
        .split("pub(crate) fn request_source_cubemap_wgpu_readback_batch")
        .nth(1)
        .and_then(|source| {
            source
                .split("struct SourceCubemapFaceReadbackLayout")
                .next()
        })
        .expect("source cubemap batch request body");

    assert!(request.contains("enqueue_product_diagnostic_texture_rgba16float_mip_chain"));
    assert!(!request.contains("backend.device.create_buffer"));
    assert!(!request.contains("&backend.device"));
    assert!(!request.contains("create_buffer"));
    assert!(!request.contains("enqueue_product_diagnostic_buffer"));
    assert!(!production.contains("retained_buffers"));
    assert!(!production.contains("record_texture_copies"));
}

#[test]
fn pending_readback_assembles_canonical_face_major_rgba16f_bytes() {
    let face_size = 2;
    let mip_count = 2;
    let pending = SourceCubemapWgpuPendingReadback::new(face_size, mip_count).unwrap();
    let batch = pending
        .plan_next_batch(DiagnosticReadbackBudget::default())
        .unwrap();
    assert_eq!(batch.face_count(), 6);

    let mut expected = Vec::new();
    for face in CubemapFace::ALL {
        let mut packed = Vec::new();
        for mip_level in 0..mip_count {
            let mip_size = source_cubemap_mip_size(face_size, mip_level);
            let value = (face.index() * mip_count as usize + mip_level as usize) as f32;
            let texels =
                vec![[value, value + 0.25, value + 0.5, 1.0]; (mip_size * mip_size) as usize];
            let encoded = encode_rgba16f_texels(&texels);
            expected.extend_from_slice(&encoded);
            packed.extend_from_slice(&encoded);
        }
        pending.record_delivery(face.index(), Ok(packed));
    }

    assert!(pending.poll_ready());
    let readback = pending.finish().unwrap();
    assert_eq!(readback.face_size(), face_size);
    assert_eq!(readback.mip_count(), mip_count);
    assert_eq!(
        readback.source_rgba16f_bytes().len(),
        source_cubemap_sample_count(face_size, mip_count) * 8
    );
    assert_eq!(readback.source_rgba16f_bytes(), expected);
    assert_eq!(readback.into_source_rgba16f_bytes(), expected);
}

#[test]
fn malformed_face_fails_only_after_all_callbacks_reach_terminal_state() {
    let pending = SourceCubemapWgpuPendingReadback::new(1, 1).unwrap();
    pending
        .plan_next_batch(DiagnosticReadbackBudget::default())
        .unwrap();
    let expected = SourceCubemapFaceReadbackLayout::new(1, 1)
        .unwrap()
        .canonical_byte_len;
    for face in 0..6 {
        let bytes = if face == 2 {
            vec![0; 7]
        } else {
            vec![0; expected]
        };
        pending.record_delivery(face, Ok(bytes));
    }

    assert!(pending.poll_ready());
    assert!(pending.finish().unwrap_err().to_string().contains("face 2"));
}

#[test]
fn source_readback_requires_a_complete_mip_pyramid() {
    let error = SourceCubemapWgpuPendingReadback::new(4, 2)
        .unwrap_err()
        .to_string();

    assert!(error.contains("expected 3"));
    assert!(error.contains("found 2"));
}

#[test]
fn default_budget_streams_a_1024_source_chain_as_two_faces_per_batch() {
    let face_size = 1024;
    let mip_count = source_cubemap_mip_count(face_size);
    let budget = DiagnosticReadbackBudget::default();
    let pending = SourceCubemapWgpuPendingReadback::new(face_size, mip_count).unwrap();

    for expected_first_face in [0, 2, 4] {
        let batch = pending.plan_next_batch(budget).unwrap();
        assert_eq!(batch.first_face(), expected_first_face);
        assert_eq!(batch.face_count(), 2);
        assert!(batch.padded_byte_len() <= budget.max_frame_bytes());
        assert!(batch.max_face_byte_len() <= budget.max_request_bytes());
        for face in batch.faces() {
            pending.record_delivery(face, Err("synthetic terminal delivery".to_string()));
        }
    }

    assert!(pending.all_faces_queued());
    assert!(pending.poll_ready());
    assert!(pending
        .finish()
        .unwrap_err()
        .to_string()
        .contains("synthetic"));
}

#[test]
fn next_batch_waits_for_the_previous_face_deliveries() {
    let pending = SourceCubemapWgpuPendingReadback::new(1024, 11).unwrap();
    let first = pending
        .plan_next_batch(DiagnosticReadbackBudget::default())
        .unwrap();

    let error = pending
        .plan_next_batch(DiagnosticReadbackBudget::default())
        .unwrap_err()
        .to_string();

    assert!(error.contains("still in flight"));
    for face in first.faces() {
        pending.record_delivery(face, Err("synthetic terminal delivery".to_string()));
    }
    assert!(pending
        .plan_next_batch(DiagnosticReadbackBudget::default())
        .is_ok());
}
