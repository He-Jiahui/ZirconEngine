use std::sync::{Arc, Mutex};

use crate::core::framework::render::{
    IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES, IblBakeArtifactContents, IblBakeArtifactDescriptor,
    IblBakeArtifactReadbackSections, SOURCE_CUBEMAP_FACE_COUNT, source_cubemap_mip_size,
};
use crate::graphics::types::GraphicsError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum IblBakeArtifactWgpuReadbackSection {
    Pmrem,
    IrradianceSh9,
    IrradianceCube,
}

/// CPU-only aggregation state for section deliveries owned by the product diagnostic router.
#[derive(Clone)]
pub(crate) struct IblBakeArtifactWgpuPendingReadback {
    state: Arc<Mutex<PendingIblBakeArtifactSections>>,
}

impl IblBakeArtifactWgpuPendingReadback {
    pub(super) fn new(descriptor: IblBakeArtifactDescriptor) -> Result<Self, GraphicsError> {
        let mip_count = usize::try_from(descriptor.mip_count()).map_err(|_| {
            GraphicsError::BufferMap("IBL artifact mip count exceeds host indexing".to_string())
        })?;
        let pmrem_slot_count = if descriptor
            .contents()
            .contains(IblBakeArtifactContents::PMREM)
        {
            SOURCE_CUBEMAP_FACE_COUNT
                .checked_mul(mip_count)
                .ok_or_else(|| {
                    GraphicsError::BufferMap("IBL artifact PMREM slot count overflowed".to_string())
                })?
        } else {
            0
        };
        let sh9_slot_count =
            usize::from(descriptor.contents().contains(IblBakeArtifactContents::SH9));
        let irradiance_cube_slot_count =
            if descriptor.contents().contains(IblBakeArtifactContents::IEM) {
                SOURCE_CUBEMAP_FACE_COUNT
            } else {
                0
            };
        let remaining = pmrem_slot_count
            .checked_add(sh9_slot_count)
            .and_then(|count| count.checked_add(irradiance_cube_slot_count))
            .ok_or_else(|| {
                GraphicsError::BufferMap("IBL artifact section slot count overflowed".to_string())
            })?;
        Ok(Self {
            state: Arc::new(Mutex::new(PendingIblBakeArtifactSections {
                descriptor,
                pmrem: empty_slots(pmrem_slot_count),
                sh9: empty_slots(sh9_slot_count),
                irradiance_cube: empty_slots(irradiance_cube_slot_count),
                remaining,
                first_error: None,
            })),
        })
    }

    pub(super) fn callback(
        &self,
        section: IblBakeArtifactWgpuReadbackSection,
        slot: usize,
    ) -> Box<dyn FnOnce(Result<Vec<u8>, String>) + Send + 'static> {
        let pending = self.clone();
        Box::new(move |result| pending.record_delivery(section, slot, result))
    }

    pub(super) fn record_delivery(
        &self,
        section: IblBakeArtifactWgpuReadbackSection,
        slot: usize,
        result: Result<Vec<u8>, String>,
    ) {
        self.lock().record(section, slot, result);
    }

    pub(crate) fn poll_ready(&self) -> bool {
        self.lock().remaining == 0
    }

    pub(crate) fn finish(self) -> Result<IblBakeArtifactReadbackSections, GraphicsError> {
        self.lock().take_sections()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, PendingIblBakeArtifactSections> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

struct PendingIblBakeArtifactSections {
    descriptor: IblBakeArtifactDescriptor,
    pmrem: Vec<Option<Result<Vec<u8>, String>>>,
    sh9: Vec<Option<Result<Vec<u8>, String>>>,
    irradiance_cube: Vec<Option<Result<Vec<u8>, String>>>,
    remaining: usize,
    first_error: Option<String>,
}

impl PendingIblBakeArtifactSections {
    fn record(
        &mut self,
        section: IblBakeArtifactWgpuReadbackSection,
        slot: usize,
        result: Result<Vec<u8>, String>,
    ) {
        let expected = self.expected_slot_byte_len(section, slot);
        let result = match (expected, result) {
            (Some(expected), Ok(bytes)) if bytes.len() == expected => Ok(bytes),
            (Some(expected), Ok(bytes)) => Err(format!(
                "IBL artifact {section:?} slot {slot} returned {} bytes, expected {expected}",
                bytes.len()
            )),
            (Some(_), Err(error)) => Err(error),
            (None, _) => {
                self.first_error.get_or_insert_with(|| {
                    format!("IBL artifact {section:?} delivery used invalid slot {slot}")
                });
                return;
            }
        };
        let slots = match section {
            IblBakeArtifactWgpuReadbackSection::Pmrem => &mut self.pmrem,
            IblBakeArtifactWgpuReadbackSection::IrradianceSh9 => &mut self.sh9,
            IblBakeArtifactWgpuReadbackSection::IrradianceCube => &mut self.irradiance_cube,
        };
        let Some(destination) = slots.get_mut(slot) else {
            return;
        };
        if destination.is_some() {
            self.first_error.get_or_insert_with(|| {
                format!("IBL artifact {section:?} slot {slot} completed more than once")
            });
            return;
        }
        if let Err(error) = &result {
            self.first_error.get_or_insert_with(|| error.clone());
        }
        *destination = Some(result);
        self.remaining = self.remaining.saturating_sub(1);
    }

    fn expected_slot_byte_len(
        &self,
        section: IblBakeArtifactWgpuReadbackSection,
        slot: usize,
    ) -> Option<usize> {
        match section {
            IblBakeArtifactWgpuReadbackSection::Pmrem => {
                if slot >= self.pmrem.len() {
                    return None;
                }
                let mip_count = self.descriptor.mip_count() as usize;
                let mip_level = u32::try_from(slot % mip_count).ok()?;
                let mip_size =
                    source_cubemap_mip_size(self.descriptor.face_size(), mip_level) as usize;
                mip_size
                    .checked_mul(mip_size)?
                    .checked_mul(IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES)
            }
            IblBakeArtifactWgpuReadbackSection::IrradianceSh9 => (slot < self.sh9.len())
                .then(|| self.descriptor.expected_irradiance_sh9_size_bytes())
                .flatten(),
            IblBakeArtifactWgpuReadbackSection::IrradianceCube => {
                if slot >= self.irradiance_cube.len() {
                    return None;
                }
                self.descriptor
                    .expected_irradiance_cube_rgba16f_size_bytes()?
                    .checked_div(SOURCE_CUBEMAP_FACE_COUNT)
            }
        }
    }

    fn take_sections(&mut self) -> Result<IblBakeArtifactReadbackSections, GraphicsError> {
        if self.remaining != 0 {
            return Err(GraphicsError::BufferMap(format!(
                "IBL artifact readback still has {} pending sections",
                self.remaining
            )));
        }
        if let Some(error) = self.first_error.take() {
            return Err(GraphicsError::BufferMap(error));
        }
        let mut sections = IblBakeArtifactReadbackSections::new(self.descriptor);
        if !self.pmrem.is_empty() {
            sections = sections.with_pmrem_rgba16f_bytes(take_ordered_bytes(&mut self.pmrem)?);
        }
        if !self.sh9.is_empty() {
            sections = sections.with_irradiance_sh9_bytes(take_ordered_bytes(&mut self.sh9)?);
        }
        if !self.irradiance_cube.is_empty() {
            sections = sections
                .with_irradiance_cube_rgba16f_bytes(take_ordered_bytes(&mut self.irradiance_cube)?);
        }
        Ok(sections)
    }
}

fn empty_slots(count: usize) -> Vec<Option<Result<Vec<u8>, String>>> {
    std::iter::repeat_with(|| None).take(count).collect()
}

fn take_ordered_bytes(
    slots: &mut [Option<Result<Vec<u8>, String>>],
) -> Result<Vec<u8>, GraphicsError> {
    let total = slots
        .iter()
        .filter_map(|slot| slot.as_ref())
        .filter_map(|result| result.as_ref().ok())
        .map(Vec::len)
        .sum();
    let mut bytes = Vec::with_capacity(total);
    for slot in slots {
        let result = slot.take().ok_or_else(|| {
            GraphicsError::BufferMap("IBL artifact section completed without bytes".to_string())
        })?;
        bytes.extend(result.map_err(GraphicsError::BufferMap)?);
    }
    Ok(bytes)
}
