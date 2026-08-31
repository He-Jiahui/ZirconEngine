#[cfg(test)]
mod batch;
mod pending;
mod resources;
#[cfg(test)]
mod staging;
#[cfg(test)]
mod tests;

#[cfg(test)]
use crate::core::framework::render::IblBakeArtifactReadbackSections;
use crate::core::framework::render::{
    SOURCE_CUBEMAP_FACE_COUNT, SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE, source_cubemap_mip_size,
};
use crate::graphics::types::GraphicsError;

#[cfg(test)]
use batch::IblBakeArtifactWgpuReadbackBatch;
pub(crate) use pending::IblBakeArtifactWgpuPendingReadback;
use pending::IblBakeArtifactWgpuReadbackSection;
pub(crate) use resources::IblBakeArtifactWgpuReadbackResources;
use resources::{
    IblBakeArtifactWgpuBufferReadback, required_irradiance_sh9_readback_resource,
    required_wgpu_readback_resource,
};

use super::render_backend::RenderBackend;

#[cfg(test)]
pub(crate) fn read_ibl_bake_artifact_wgpu_sections(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    resources: IblBakeArtifactWgpuReadbackResources<'_>,
) -> Result<IblBakeArtifactReadbackSections, GraphicsError> {
    let descriptor = resources.descriptor();
    build_ibl_bake_artifact_wgpu_readback_batch(device, resources)?
        .finish(device, queue, descriptor)
}

pub(crate) fn request_ibl_bake_artifact_wgpu_readback(
    backend: &RenderBackend,
    resources: IblBakeArtifactWgpuReadbackResources<'_>,
) -> Result<IblBakeArtifactWgpuPendingReadback, GraphicsError> {
    let descriptor = resources.descriptor();
    let pmrem = resources
        .requires_pmrem_texture()
        .then(|| required_wgpu_readback_resource(resources.pmrem_texture(), "PMREM texture"))
        .transpose()?;
    let sh9 = resources
        .requires_irradiance_sh9_buffer()
        .then(|| required_irradiance_sh9_readback_resource(&resources))
        .transpose()?;
    let irradiance_cube = resources
        .requires_irradiance_cube_texture()
        .then(|| {
            required_wgpu_readback_resource(
                resources.irradiance_cube_texture(),
                "irradiance cube texture",
            )
        })
        .transpose()?;
    let pending = IblBakeArtifactWgpuPendingReadback::new(descriptor)?;

    if let Some(texture) = pmrem {
        for face in 0..SOURCE_CUBEMAP_FACE_COUNT {
            for mip_level in 0..descriptor.mip_count() as usize {
                let slot = face * descriptor.mip_count() as usize + mip_level;
                let mip_size = source_cubemap_mip_size(descriptor.face_size(), mip_level as u32);
                request_texture_section(
                    backend,
                    &pending,
                    IblBakeArtifactWgpuReadbackSection::Pmrem,
                    slot,
                    texture,
                    mip_level as u32,
                    face as u32,
                    mip_size,
                );
            }
        }
    }
    if let Some(buffer) = sh9 {
        request_buffer_section(
            backend,
            &pending,
            IblBakeArtifactWgpuReadbackSection::IrradianceSh9,
            0,
            buffer,
        );
    }
    if let Some(texture) = irradiance_cube {
        for face in 0..SOURCE_CUBEMAP_FACE_COUNT {
            request_texture_section(
                backend,
                &pending,
                IblBakeArtifactWgpuReadbackSection::IrradianceCube,
                face,
                texture,
                0,
                face as u32,
                SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
            );
        }
    }
    Ok(pending)
}

#[cfg(test)]
fn build_ibl_bake_artifact_wgpu_readback_batch(
    device: &wgpu::Device,
    resources: IblBakeArtifactWgpuReadbackResources<'_>,
) -> Result<IblBakeArtifactWgpuReadbackBatch, GraphicsError> {
    let descriptor = resources.descriptor();
    let mut batch = IblBakeArtifactWgpuReadbackBatch::new(device);

    if resources.requires_pmrem_texture() {
        let texture = required_wgpu_readback_resource(resources.pmrem_texture(), "PMREM texture")?;
        batch.add_pmrem(
            device,
            texture,
            descriptor.face_size(),
            descriptor.mip_count(),
        );
    }

    if resources.requires_irradiance_sh9_buffer() {
        let buffer = required_irradiance_sh9_readback_resource(&resources)?;
        batch.add_sh9(device, &buffer);
    }

    if resources.requires_irradiance_cube_texture() {
        let texture = required_wgpu_readback_resource(
            resources.irradiance_cube_texture(),
            "irradiance cube texture",
        )?;
        batch.add_irradiance_cube(device, texture);
    }

    Ok(batch)
}

fn request_texture_section(
    backend: &RenderBackend,
    pending: &IblBakeArtifactWgpuPendingReadback,
    section: IblBakeArtifactWgpuReadbackSection,
    slot: usize,
    texture: &wgpu::Texture,
    mip_level: u32,
    array_layer: u32,
    size: u32,
) {
    let callback = pending.callback(section, slot);
    match backend.enqueue_product_diagnostic_texture_rgba16float(
        texture,
        mip_level,
        array_layer,
        size,
        size,
        callback,
    ) {
        Ok(true) => {}
        Ok(false) => pending.record_delivery(
            section,
            slot,
            Err(format!(
                "IBL artifact {section:?} slot {slot} exceeded the product diagnostic budget"
            )),
        ),
        Err(error) => pending.record_delivery(section, slot, Err(error.to_string())),
    }
}

fn request_buffer_section(
    backend: &RenderBackend,
    pending: &IblBakeArtifactWgpuPendingReadback,
    section: IblBakeArtifactWgpuReadbackSection,
    slot: usize,
    buffer: IblBakeArtifactWgpuBufferReadback<'_>,
) {
    let callback = pending.callback(section, slot);
    match backend.enqueue_product_diagnostic_buffer(
        buffer.buffer,
        buffer.offset,
        buffer.size,
        callback,
    ) {
        Ok(true) => {}
        Ok(false) => pending.record_delivery(
            section,
            slot,
            Err(format!(
                "IBL artifact {section:?} slot {slot} exceeded the product diagnostic budget"
            )),
        ),
        Err(error) => pending.record_delivery(section, slot, Err(error.to_string())),
    }
}
