use std::error::Error;
use std::fs;
use std::path::Path;

use image::ImageFormat;
use zircon_runtime::asset::artifact::IblSourceCubemapStagingStore;
use zircon_runtime::asset::{stage_environment_ibl_source, AssetImportContext, AssetUri};
use zircon_runtime::core::framework::render::{
    SourceCubemapEnvironment, SourceCubemapPrefilterQuality,
};

use crate::args::{MAX_FACE_SIZE, MIN_FACE_SIZE};

const DEFAULT_ENVIRONMENT_INTENSITY: f32 = 0.65;

pub(crate) fn source_cubemap_environment(
    hdri_path: &Path,
    requested_face_size: Option<u32>,
    requested_pmrem_face_size: Option<u32>,
    cache_root: &Path,
) -> Result<SourceCubemapEnvironment, Box<dyn Error>> {
    let bytes = fs::read(hdri_path)?;
    let image = image::load_from_memory_with_format(&bytes, ImageFormat::Hdr)?.to_rgb32f();
    let exposure = sampled_hdri_exposure(&image);
    let face_size = resolved_face_size(requested_face_size, image.height());
    let uri = AssetUri::parse("res://environment/viewer_hdri.hdr")?;
    let context = AssetImportContext::new(
        hdri_path.to_path_buf(),
        uri.clone(),
        bytes,
        format!("environment_ibl = true\nenvironment_ibl_face_size = {face_size}").parse()?,
    );
    let staged = stage_environment_ibl_source(&context, cache_root)?;
    let request = *staged.request().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "viewer HDRI did not produce a staged IBL request",
        )
    })?;
    let store = IblSourceCubemapStagingStore::new(cache_root);
    let mut environment = store.read_source_cubemap_environment(&request, uri)?;
    let pmrem_face_size = resolved_pmrem_face_size(requested_pmrem_face_size, face_size);
    apply_viewer_pmrem_layout(&mut environment, pmrem_face_size);
    environment.intensity = DEFAULT_ENVIRONMENT_INTENSITY * exposure;
    environment.rotation_radians = 0.0;
    println!(
        "loaded staged HDRI environment: status={:?}, source_face_size={}, source_mip_count={}, staged_pmrem_face_size={}, staged_pmrem_mip_count={}, active_pmrem_face_size={}, active_pmrem_mip_count={}, source={}, derived={}",
        staged.status(),
        request.source_face_size(),
        request.source_mip_count(),
        request.pmrem_face_size(),
        request.pmrem_mip_count(),
        environment.mip_chain.pmrem_face_size(),
        environment.mip_chain.pmrem_mip_count(),
        staged
            .source_zcube_path()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "<missing>".to_string()),
        staged
            .asset_derived_path()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "<missing>".to_string()),
    );
    Ok(environment)
}

// Explicit CLI sizing wins; otherwise use the runtime's native equirectangular mapping.
fn resolved_face_size(requested_face_size: Option<u32>, equirect_height: u32) -> u32 {
    requested_face_size.unwrap_or_else(|| {
        zircon_runtime::core::framework::render::source_cubemap_face_size_from_equirect_height(
            equirect_height,
        )
        .clamp(MIN_FACE_SIZE, MAX_FACE_SIZE)
    })
}

fn resolved_pmrem_face_size(requested_pmrem_face_size: Option<u32>, source_face_size: u32) -> u32 {
    requested_pmrem_face_size.unwrap_or(source_face_size)
}

fn apply_viewer_pmrem_layout(environment: &mut SourceCubemapEnvironment, pmrem_face_size: u32) {
    if environment.mip_chain.pmrem_face_size() == pmrem_face_size {
        return;
    }

    environment.mip_chain = environment
        .mip_chain
        .with_pmrem_face_size(pmrem_face_size, SourceCubemapPrefilterQuality::Normal);
    environment.irradiance_sh9 = *environment.mip_chain.irradiance_sh9();
    // This PMREM is a viewer-side validation result, not the staged derived artifact.
    environment.bake_artifact_hash = [0; 4];
}

fn sampled_hdri_exposure(image: &image::Rgb32FImage) -> f32 {
    let step_x = (image.width() / 128).max(1);
    let step_y = (image.height() / 64).max(1);
    let mut sum = 0.0_f32;
    let mut count = 0.0_f32;
    let mut y = 0;
    while y < image.height() {
        let mut x = 0;
        while x < image.width() {
            sum += luma(image.get_pixel(x, y).0);
            count += 1.0;
            x += step_x;
        }
        y += step_y;
    }
    (0.45 / (sum / count.max(1.0)).max(0.0001)).clamp(0.02, 4.0)
}

fn luma(rgb: [f32; 3]) -> f32 {
    rgb[0] * 0.2126 + rgb[1] * 0.7152 + rgb[2] * 0.0722
}

#[cfg(test)]
mod tests {
    use super::{resolved_face_size, resolved_pmrem_face_size};

    #[test]
    fn automatic_face_size_matches_native_equirect_angular_resolution() {
        assert_eq!(resolved_face_size(None, 512), 256);
        assert_eq!(resolved_face_size(None, 1024), 512);
        assert_eq!(resolved_face_size(None, 4096), 1024);
    }

    #[test]
    fn explicit_face_size_overrides_hdri_resolution() {
        assert_eq!(resolved_face_size(Some(128), 4096), 128);
    }

    #[test]
    fn automatic_pmrem_result_size_matches_resolved_source_size() {
        assert_eq!(resolved_pmrem_face_size(None, 512), 512);
    }

    #[test]
    fn explicit_pmrem_result_size_is_independent_from_source_size() {
        assert_eq!(resolved_pmrem_face_size(Some(256), 512), 256);
    }
}
