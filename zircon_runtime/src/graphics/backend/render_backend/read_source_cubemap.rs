use std::fmt;
use std::sync::{Arc, Mutex};

use crate::core::framework::render::{
    RGBA16F_TEXEL_SIZE_BYTES, SOURCE_CUBEMAP_FACE_COUNT, source_cubemap_mip_count,
    source_cubemap_mip_size,
};
use crate::graphics::types::GraphicsError;
use zr_rhi::DiagnosticReadbackBudget;

use super::RenderBackend;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceCubemapWgpuReadback {
    face_size: u32,
    mip_count: u32,
    source_rgba16f: Vec<u8>,
}

impl SourceCubemapWgpuReadback {
    pub(crate) const fn face_size(&self) -> u32 {
        self.face_size
    }

    pub(crate) const fn mip_count(&self) -> u32 {
        self.mip_count
    }

    pub(crate) fn source_rgba16f_bytes(&self) -> &[u8] {
        &self.source_rgba16f
    }

    pub(crate) fn into_source_rgba16f_bytes(self) -> Vec<u8> {
        self.source_rgba16f
    }
}

/// One bounded diagnostic frame from a multi-frame source-cubemap readback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SourceCubemapWgpuReadbackBatch {
    first_face: usize,
    face_count: usize,
    padded_byte_len: u64,
    max_face_byte_len: u64,
    all_faces_queued: bool,
}

impl SourceCubemapWgpuReadbackBatch {
    pub(crate) const fn first_face(self) -> usize {
        self.first_face
    }

    pub(crate) const fn face_count(self) -> usize {
        self.face_count
    }

    pub(crate) const fn padded_byte_len(self) -> u64 {
        self.padded_byte_len
    }

    pub(crate) const fn max_face_byte_len(self) -> u64 {
        self.max_face_byte_len
    }

    pub(crate) const fn all_faces_queued(self) -> bool {
        self.all_faces_queued
    }

    pub(crate) fn faces(self) -> std::ops::Range<usize> {
        self.first_face..self.first_face + self.face_count
    }
}

/// CPU aggregation and GPU staging ownership for a streamed source-cubemap readback.
///
/// Each request packs every mip of one face into one COPY_SRC buffer. At the default RHI budget a
/// 1024 source chain therefore uses six requests over three frames instead of 66 requests in one
/// frame. Only one batch may be in flight, bounding padded staging bytes independently from the
/// final canonical RGBA16F payload.
#[derive(Clone)]
pub(crate) struct SourceCubemapWgpuPendingReadback {
    state: Arc<Mutex<PendingSourceCubemapFaces>>,
}

impl fmt::Debug for SourceCubemapWgpuPendingReadback {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.lock();
        formatter
            .debug_struct("SourceCubemapWgpuPendingReadback")
            .field("face_size", &state.face_size)
            .field("mip_count", &state.mip_count)
            .field("next_face", &state.next_face)
            .field("in_flight", &state.in_flight)
            .field("remaining", &state.remaining)
            .finish()
    }
}

impl SourceCubemapWgpuPendingReadback {
    fn new(face_size: u32, mip_count: u32) -> Result<Self, GraphicsError> {
        validate_source_layout(face_size, mip_count)?;
        let face_layout = SourceCubemapFaceReadbackLayout::new(face_size, mip_count)?;
        Ok(Self {
            state: Arc::new(Mutex::new(PendingSourceCubemapFaces {
                face_size,
                mip_count,
                face_layout,
                completed_faces: vec![false; SOURCE_CUBEMAP_FACE_COUNT],
                source_rgba16f: None,
                next_face: 0,
                in_flight: 0,
                remaining: SOURCE_CUBEMAP_FACE_COUNT,
                first_error: None,
            })),
        })
    }

    fn callback(&self, face: usize) -> Box<dyn FnOnce(Result<Vec<u8>, String>) + Send + 'static> {
        let pending = self.clone();
        Box::new(move |result| pending.record_delivery(face, result))
    }

    fn record_delivery(&self, face: usize, result: Result<Vec<u8>, String>) {
        self.lock().record(face, result);
    }

    fn plan_next_batch(
        &self,
        budget: DiagnosticReadbackBudget,
    ) -> Result<SourceCubemapWgpuReadbackBatch, GraphicsError> {
        self.lock().plan_next_batch(budget)
    }

    fn face_layout(&self) -> SourceCubemapFaceReadbackLayout {
        self.lock().face_layout.clone()
    }

    pub(crate) fn batch_in_flight(&self) -> bool {
        self.lock().in_flight != 0
    }

    pub(crate) fn all_faces_queued(&self) -> bool {
        self.lock().next_face == SOURCE_CUBEMAP_FACE_COUNT
    }

    pub(crate) fn poll_ready(&self) -> bool {
        self.lock().remaining == 0
    }

    pub(crate) fn finish(self) -> Result<SourceCubemapWgpuReadback, GraphicsError> {
        self.lock().take_readback()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, PendingSourceCubemapFaces> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

pub(crate) fn begin_source_cubemap_wgpu_readback(
    face_size: u32,
    mip_count: u32,
) -> Result<SourceCubemapWgpuPendingReadback, GraphicsError> {
    SourceCubemapWgpuPendingReadback::new(face_size, mip_count)
}

/// Queues the next budget-bounded face batch into the caller's diagnostic frame.
///
/// The caller owns the surrounding readback scope and submission. A later batch may be requested
/// only after all callbacks from this batch have reached a terminal state.
pub(crate) fn request_source_cubemap_wgpu_readback_batch(
    backend: &RenderBackend,
    texture: &wgpu::Texture,
    pending: &SourceCubemapWgpuPendingReadback,
) -> Result<SourceCubemapWgpuReadbackBatch, GraphicsError> {
    let budget = backend.device_profile().diagnostic_readback_budget();
    let batch = pending.plan_next_batch(budget)?;
    let layout = pending.face_layout();

    for face in batch.faces() {
        let callback = pending.callback(face);
        match backend.enqueue_product_diagnostic_texture_rgba16float_mip_chain(
            texture,
            face as u32,
            layout.mip_count(),
            callback,
        ) {
            Ok(true) => {}
            Ok(false) => pending.record_delivery(
                face,
                Err(format!(
                    "source cubemap face {face} exceeded the product diagnostic budget"
                )),
            ),
            Err(error) => pending.record_delivery(face, Err(error.to_string())),
        }
    }

    Ok(batch)
}

#[derive(Clone, Debug)]
struct SourceCubemapFaceReadbackLayout {
    mip_count: u32,
    padded_byte_len: u64,
    canonical_byte_len: usize,
}

impl SourceCubemapFaceReadbackLayout {
    fn new(face_size: u32, mip_count: u32) -> Result<Self, GraphicsError> {
        let mut padded_byte_len = 0_u64;
        let mut canonical_byte_len = 0_usize;
        for mip_level in 0..mip_count {
            let mip_size = source_cubemap_mip_size(face_size, mip_level);
            let unpadded_bytes_per_row = mip_size
                .checked_mul(RGBA16F_TEXEL_SIZE_BYTES as u32)
                .ok_or_else(|| {
                    GraphicsError::BufferMap("source cubemap row byte count overflowed".to_string())
                })?;
            let padded_bytes_per_row = padded_copy_bytes_per_row(unpadded_bytes_per_row)?;
            let padded_mip_bytes = u64::from(padded_bytes_per_row)
                .checked_mul(u64::from(mip_size))
                .ok_or_else(|| {
                    GraphicsError::BufferMap(
                        "source cubemap padded mip byte count overflowed".to_string(),
                    )
                })?;
            let canonical_mip_bytes = usize::try_from(unpadded_bytes_per_row)
                .ok()
                .and_then(|row| row.checked_mul(mip_size as usize))
                .ok_or_else(|| {
                    GraphicsError::BufferMap(
                        "source cubemap canonical mip byte count overflowed".to_string(),
                    )
                })?;
            padded_byte_len = padded_byte_len
                .checked_add(padded_mip_bytes)
                .ok_or_else(|| {
                    GraphicsError::BufferMap(
                        "source cubemap padded face byte count overflowed".to_string(),
                    )
                })?;
            canonical_byte_len = canonical_byte_len
                .checked_add(canonical_mip_bytes)
                .ok_or_else(|| {
                    GraphicsError::BufferMap(
                        "source cubemap canonical face byte count overflowed".to_string(),
                    )
                })?;
        }
        Ok(Self {
            mip_count,
            padded_byte_len,
            canonical_byte_len,
        })
    }

    const fn mip_count(&self) -> u32 {
        self.mip_count
    }

    const fn padded_byte_len(&self) -> u64 {
        self.padded_byte_len
    }
}

struct PendingSourceCubemapFaces {
    face_size: u32,
    mip_count: u32,
    face_layout: SourceCubemapFaceReadbackLayout,
    completed_faces: Vec<bool>,
    source_rgba16f: Option<Vec<u8>>,
    next_face: usize,
    in_flight: usize,
    remaining: usize,
    first_error: Option<String>,
}

impl PendingSourceCubemapFaces {
    fn plan_next_batch(
        &mut self,
        budget: DiagnosticReadbackBudget,
    ) -> Result<SourceCubemapWgpuReadbackBatch, GraphicsError> {
        if self.in_flight != 0 {
            return Err(GraphicsError::BufferMap(format!(
                "source cubemap readback still has {} faces in flight",
                self.in_flight
            )));
        }
        if self.next_face == SOURCE_CUBEMAP_FACE_COUNT {
            return Err(GraphicsError::BufferMap(
                "source cubemap readback already queued every face".to_string(),
            ));
        }

        let face_bytes = self.face_layout.padded_byte_len;
        let frame_byte_limit = budget.max_frame_bytes().min(budget.max_pending_bytes());
        if face_bytes > budget.max_request_bytes() || face_bytes > frame_byte_limit {
            return Err(GraphicsError::BufferMap(format!(
                "source cubemap packed face requires {face_bytes} bytes; request/frame admission is {}/{} bytes",
                budget.max_request_bytes(),
                frame_byte_limit
            )));
        }
        let by_bytes = usize::try_from(frame_byte_limit / face_bytes).unwrap_or(usize::MAX);
        let remaining_faces = SOURCE_CUBEMAP_FACE_COUNT - self.next_face;
        let face_count = remaining_faces
            .min(budget.max_requests_per_frame())
            .min(by_bytes);
        if face_count == 0 {
            return Err(GraphicsError::BufferMap(
                "source cubemap readback budget admits no face requests".to_string(),
            ));
        }
        let padded_byte_len = face_bytes.checked_mul(face_count as u64).ok_or_else(|| {
            GraphicsError::BufferMap("source cubemap batch byte count overflowed".to_string())
        })?;
        let first_face = self.next_face;
        self.next_face += face_count;
        self.in_flight = face_count;
        Ok(SourceCubemapWgpuReadbackBatch {
            first_face,
            face_count,
            padded_byte_len,
            max_face_byte_len: face_bytes,
            all_faces_queued: self.next_face == SOURCE_CUBEMAP_FACE_COUNT,
        })
    }

    fn record(&mut self, face: usize, result: Result<Vec<u8>, String>) {
        let Some(completed) = self.completed_faces.get(face).copied() else {
            self.first_error
                .get_or_insert_with(|| format!("source cubemap delivery used invalid face {face}"));
            return;
        };
        if completed {
            self.first_error.get_or_insert_with(|| {
                format!("source cubemap face {face} completed more than once")
            });
            return;
        }
        let expected = self.face_layout.canonical_byte_len;
        let result = match result {
            Ok(bytes) if bytes.len() == expected => self.copy_face_into_output(face, &bytes),
            Ok(bytes) => Err(format!(
                "source cubemap face {face} returned {} bytes, expected {expected}",
                bytes.len()
            )),
            Err(error) => Err(error),
        };
        if let Err(error) = &result {
            self.first_error.get_or_insert_with(|| error.clone());
        }
        self.completed_faces[face] = true;
        self.in_flight = self.in_flight.saturating_sub(1);
        self.remaining = self.remaining.saturating_sub(1);
    }

    fn copy_face_into_output(&mut self, face: usize, packed: &[u8]) -> Result<(), String> {
        let total_bytes = self
            .face_layout
            .canonical_byte_len
            .checked_mul(SOURCE_CUBEMAP_FACE_COUNT)
            .ok_or_else(|| "source cubemap canonical payload byte count overflowed".to_string())?;
        let output = self
            .source_rgba16f
            .get_or_insert_with(|| vec![0; total_bytes]);
        let output_offset = face
            .checked_mul(self.face_layout.canonical_byte_len)
            .ok_or_else(|| "source cubemap canonical face offset overflowed".to_string())?;
        let output_end = output_offset
            .checked_add(self.face_layout.canonical_byte_len)
            .ok_or_else(|| "source cubemap canonical face extent overflowed".to_string())?;
        output[output_offset..output_end].copy_from_slice(packed);
        Ok(())
    }

    fn take_readback(&mut self) -> Result<SourceCubemapWgpuReadback, GraphicsError> {
        if self.remaining != 0 {
            return Err(GraphicsError::BufferMap(format!(
                "source cubemap readback still has {} pending faces",
                self.remaining
            )));
        }
        if let Some(error) = self.first_error.take() {
            return Err(GraphicsError::BufferMap(error));
        }
        let source_rgba16f = self.source_rgba16f.take().ok_or_else(|| {
            GraphicsError::BufferMap(
                "source cubemap readback completed without a canonical payload".to_string(),
            )
        })?;
        Ok(SourceCubemapWgpuReadback {
            face_size: self.face_size,
            mip_count: self.mip_count,
            source_rgba16f,
        })
    }
}

fn validate_source_layout(face_size: u32, mip_count: u32) -> Result<(), GraphicsError> {
    if face_size == 0 || mip_count == 0 {
        return Err(GraphicsError::BufferMap(format!(
            "source cubemap layout must be nonzero, found face_size={face_size}, mip_count={mip_count}"
        )));
    }
    let expected = source_cubemap_mip_count(face_size);
    if mip_count != expected {
        return Err(GraphicsError::BufferMap(format!(
            "source cubemap readback requires a complete mip pyramid: expected {expected}, found {mip_count}"
        )));
    }
    SourceCubemapFaceReadbackLayout::new(face_size, mip_count).map(|_| ())
}

fn padded_copy_bytes_per_row(unpadded_bytes_per_row: u32) -> Result<u32, GraphicsError> {
    let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    unpadded_bytes_per_row
        .checked_add(alignment - 1)
        .map(|value| value / alignment * alignment)
        .ok_or_else(|| {
            GraphicsError::BufferMap("source cubemap padded row byte count overflowed".to_string())
        })
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        CubemapFace, encode_rgba16f_texels, source_cubemap_mip_count, source_cubemap_mip_size,
        source_cubemap_sample_count,
    };
    use zr_rhi::DiagnosticReadbackBudget;

    use super::{SourceCubemapFaceReadbackLayout, SourceCubemapWgpuPendingReadback};

    #[test]
    fn source_cubemap_mip_chain_copies_directly_into_owner_staging() {
        let source = include_str!("read_source_cubemap.rs");
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
        assert!(
            pending
                .finish()
                .unwrap_err()
                .to_string()
                .contains("synthetic")
        );
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
        assert!(
            pending
                .plan_next_batch(DiagnosticReadbackBudget::default())
                .is_ok()
        );
    }
}
