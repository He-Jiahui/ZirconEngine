use std::{mem::size_of, ops::Range, sync::Arc};

use crate::core::framework::render::{
    RenderFrameExtract, SubsurfaceProfileData, ViewProjectionMatrixPair, ZR_SSS_MAX_PROFILES,
    resolve_subsurface_profile_table,
};
use crate::core::math::UVec2;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

use super::SSS_TILE_SIZE;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuSubsurfaceParams {
    viewport_width: u32,
    viewport_height: u32,
    profile_count: u32,
    active_profile_mask: u32,
    inverse_view_projection: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuSubsurfaceProfile {
    scatter_radius_and_world_scale: [f32; 4],
    falloff_and_profile_id: [f32; 4],
}

impl From<SubsurfaceProfileData> for GpuSubsurfaceProfile {
    fn from(profile: SubsurfaceProfileData) -> Self {
        Self {
            scatter_radius_and_world_scale: [
                profile.scatter_radius_rgb.x.max(0.001),
                profile.scatter_radius_rgb.y.max(0.001),
                profile.scatter_radius_rgb.z.max(0.001),
                profile.world_unit_scale.max(0.0),
            ],
            falloff_and_profile_id: [
                profile.falloff_rgb.x.max(0.0),
                profile.falloff_rgb.y.max(0.0),
                profile.falloff_rgb.z.max(0.0),
                profile.profile_id as f32,
            ],
        }
    }
}

pub(crate) const SSS_PARAMS_BUFFER_SIZE_BYTES: u64 = size_of::<GpuSubsurfaceParams>() as u64;
pub(crate) const SSS_PROFILE_TABLE_BUFFER_SIZE_BYTES: u64 =
    size_of::<[GpuSubsurfaceProfile; ZR_SSS_MAX_PROFILES]>() as u64;

pub(super) struct PreparedSubsurfaceFrame {
    dispatch: [u32; 3],
    payload: Arc<[u8]>,
    params_range: Range<usize>,
    profile_table_range: Range<usize>,
}

impl PreparedSubsurfaceFrame {
    pub(super) fn prepare(extract: &RenderFrameExtract, size: UVec2) -> Result<Self, String> {
        let table = resolve_subsurface_profile_table(
            &extract.lighting.advanced_lighting.subsurface_profiles,
        );
        if table.profiles.is_empty() {
            return Err(
                "sss.prepare requires at least one resolved subsurface profile".to_string(),
            );
        }

        let camera = extract.view.selected_effective_camera();
        let inverse_view_projection = ViewProjectionMatrixPair::from_camera(&camera, size)
            .clip_from_world_jittered
            .inverse();
        let profile_count = u32::try_from(table.profiles.len())
            .map_err(|_| "sss profile count exceeds u32".to_string())?;
        let params = GpuSubsurfaceParams {
            viewport_width: size.x.max(1),
            viewport_height: size.y.max(1),
            profile_count,
            active_profile_mask: table.active_profile_mask,
            inverse_view_projection: inverse_view_projection.to_cols_array_2d(),
        };
        let mut gpu_profiles = [GpuSubsurfaceProfile::default(); ZR_SSS_MAX_PROFILES];
        for (output, profile) in gpu_profiles.iter_mut().zip(table.profiles.iter().copied()) {
            *output = profile.into();
        }

        let payload_capacity = usize::try_from(Self::uploaded_byte_len())
            .map_err(|_| "sss upload payload size exceeds usize".to_string())?;
        let mut payload = Vec::with_capacity(payload_capacity);
        let params_start = payload.len();
        payload.extend_from_slice(bytemuck::bytes_of(&params));
        let params_range = params_start..payload.len();
        let profile_table_start = payload.len();
        payload.extend_from_slice(bytemuck::cast_slice(&gpu_profiles));
        let profile_table_range = profile_table_start..payload.len();
        debug_assert_eq!(payload.len(), payload_capacity);

        Ok(Self {
            dispatch: [
                size.x.max(1).div_ceil(SSS_TILE_SIZE[0]),
                size.y.max(1).div_ceil(SSS_TILE_SIZE[1]),
                1,
            ],
            payload: payload.into(),
            params_range,
            profile_table_range,
        })
    }

    pub(super) const fn dispatch(&self) -> [u32; 3] {
        self.dispatch
    }

    pub(super) const fn uploaded_byte_len() -> u64 {
        SSS_PARAMS_BUFFER_SIZE_BYTES + SSS_PROFILE_TABLE_BUFFER_SIZE_BYTES
    }

    pub(super) fn buffer_uploads(
        &self,
        params: wgpu::Buffer,
        profile_table: wgpu::Buffer,
    ) -> Result<WgpuBufferUploadBatch, String> {
        let params_upload = WgpuBufferUpload::new(
            params,
            0,
            Arc::clone(&self.payload),
            self.params_range.clone(),
        )
        .ok_or_else(|| "sss.prepare produced an invalid params upload range".to_string())?;
        let profile_table_upload = WgpuBufferUpload::new(
            profile_table,
            0,
            Arc::clone(&self.payload),
            self.profile_table_range.clone(),
        )
        .ok_or_else(|| "sss.prepare produced an invalid profile-table upload range".to_string())?;
        let mut uploads = WgpuBufferUploadBatch::new();
        uploads.push(params_upload);
        uploads.push(profile_table_upload);
        Ok(uploads)
    }
}
