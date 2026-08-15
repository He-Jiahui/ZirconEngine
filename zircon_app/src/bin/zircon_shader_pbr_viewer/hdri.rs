use std::error::Error;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::Instant;

use image::{ImageFormat, ImageReader};
use zircon_runtime::asset::artifact::IblSourceCubemapStagingStore;
use zircon_runtime::asset::importer::{
    restore_environment_ibl_source_if_current,
    stage_environment_ibl_source_with_parallel_executor_and_decoded_image,
    DecodedTextureImageRgba32F, EnvironmentIblSourceStagingOutput,
    EnvironmentIblSourceStagingRestore, EnvironmentIblSourceStagingStatus,
    EnvironmentIblSourceStagingTiming,
};
use zircon_runtime::asset::{AssetImportContext, AssetUri};
use zircon_runtime::core::framework::{
    render::SourceCubemapEnvironment, tasks::ParallelSliceExecutor,
};
use zircon_runtime::core::resource::io::atomic_write;

use crate::args::{MAX_FACE_SIZE, MIN_FACE_SIZE};

const VIEWER_HDRI_EXPOSURE_CACHE_SCHEMA: &str = "zircon_shader_pbr_viewer_hdri_exposure_v1";

const DEFAULT_ENVIRONMENT_INTENSITY: f32 = 0.65;

pub(crate) struct SourceCubemapEnvironmentLoad {
    pub(crate) environment: SourceCubemapEnvironment,
    pub(crate) staging_status: EnvironmentIblSourceStagingStatus,
    pub(crate) staging_elapsed: std::time::Duration,
    pub(crate) staging_timing: EnvironmentIblSourceStagingTiming,
    pub(crate) staging_output: EnvironmentIblSourceStagingOutput,
    pub(crate) total_elapsed: std::time::Duration,
    pub(crate) source_pixel_decode_elapsed: std::time::Duration,
    pub(crate) source_cubemap_face_size: u32,
    pub(crate) source_cubemap_mip_count: u32,
    pub(crate) pmrem_face_size: u32,
    pub(crate) pmrem_mip_count: u32,
}

pub(crate) struct ViewerHdriPreflight {
    source_path: PathBuf,
    bytes: Vec<u8>,
    width: u32,
    height: u32,
    face_size: u32,
    pmrem_face_size: u32,
}

struct DecodedViewerHdri {
    image: DecodedTextureImageRgba32F,
    exposure: f32,
}

pub(crate) fn preflight_viewer_hdri(
    hdri_path: &Path,
    requested_face_size: Option<u32>,
    requested_pmrem_face_size: Option<u32>,
) -> Result<ViewerHdriPreflight, Box<dyn Error>> {
    let bytes = fs::read(hdri_path)?;
    let (width, height) =
        ImageReader::with_format(Cursor::new(&bytes), ImageFormat::Hdr).into_dimensions()?;
    validate_equirectangular_dimensions(width, height)?;
    let face_size = resolved_face_size(requested_face_size, height);
    let pmrem_face_size = resolved_pmrem_face_size(requested_pmrem_face_size, face_size);

    Ok(ViewerHdriPreflight {
        source_path: hdri_path.to_path_buf(),
        bytes,
        width,
        height,
        face_size,
        pmrem_face_size,
    })
}

fn decode_viewer_hdri(context: &AssetImportContext) -> Result<DecodedViewerHdri, Box<dyn Error>> {
    let image =
        image::load_from_memory_with_format(&context.source_bytes, ImageFormat::Hdr)?.to_rgba32f();
    let image = DecodedTextureImageRgba32F {
        width: image.width(),
        height: image.height(),
        rgba: image.pixels().map(|pixel| pixel.0).collect(),
    };
    validate_equirectangular_dimensions(image.width, image.height)?;
    Ok(DecodedViewerHdri {
        exposure: sampled_hdri_exposure(&image),
        image,
    })
}

pub(crate) fn source_cubemap_environment<E>(
    hdri: ViewerHdriPreflight,
    cache_root: &Path,
    parallel_executor: &E,
) -> Result<SourceCubemapEnvironmentLoad, Box<dyn Error>>
where
    E: ParallelSliceExecutor,
{
    let ViewerHdriPreflight {
        source_path,
        bytes,
        width,
        height,
        face_size,
        pmrem_face_size,
    } = hdri;
    let uri = AssetUri::parse("res://environment/viewer_hdri.hdr")?;
    let context = AssetImportContext::new(
        source_path,
        uri.clone(),
        bytes,
        format!(
            "environment_ibl = true\nenvironment_ibl_face_size = {face_size}\nenvironment_ibl_pmrem_face_size = {pmrem_face_size}"
        )
        .parse()?,
    );

    let restore_started = Instant::now();
    let restored = restore_environment_ibl_source_if_current(&context, cache_root, width, height)?;
    let restore_elapsed = restore_started.elapsed();
    let mut restored_without_exposure = None;
    if let Some(restored) = restored {
        let report = restored.report();
        let exposure = report
            .source_zcube_path()
            .and_then(read_cached_hdri_exposure);
        if let Some(exposure) = exposure {
            return Ok(restored_viewer_hdri_environment_load(
                restored,
                exposure,
                restore_elapsed,
                std::time::Duration::ZERO,
            ));
        }
        restored_without_exposure = Some(restored);
    }

    let decode_started = Instant::now();
    let DecodedViewerHdri { image, exposure } = decode_viewer_hdri(&context)?;
    let source_pixel_decode_elapsed = decode_started.elapsed();
    // A valid artifact remains authoritative while the Viewer reconstructs its local exposure.
    if let Some(restored) = restored_without_exposure {
        if let Some(source_zcube_path) = restored.report().source_zcube_path() {
            if let Err(error) = write_cached_hdri_exposure(source_zcube_path, exposure) {
                eprintln!(
                    "write viewer HDRI exposure cache {}: {error}",
                    viewer_hdri_exposure_path(source_zcube_path).display(),
                );
            }
        }
        return Ok(restored_viewer_hdri_environment_load(
            restored,
            exposure,
            restore_elapsed,
            source_pixel_decode_elapsed,
        ));
    }

    let staging_started = Instant::now();
    let staged = stage_environment_ibl_source_with_parallel_executor_and_decoded_image(
        &context,
        cache_root,
        image,
        parallel_executor,
    )?;
    let staging_elapsed = staging_started.elapsed();
    let staging_timing = staged.timing();
    let staging_output = staged.output();
    let request = *staged.request().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "viewer HDRI did not produce a staged IBL request",
        )
    })?;
    if let Some(source_zcube_path) = staged.source_zcube_path() {
        if let Err(error) = write_cached_hdri_exposure(source_zcube_path, exposure) {
            eprintln!(
                "write viewer HDRI exposure cache {}: {error}",
                viewer_hdri_exposure_path(source_zcube_path).display(),
            );
        }
    }
    let store = IblSourceCubemapStagingStore::new(cache_root);
    let hydration_started = Instant::now();
    let mut environment = store.read_source_cubemap_environment(&request, uri)?;
    let hydration_elapsed = hydration_started.elapsed();
    environment.intensity = DEFAULT_ENVIRONMENT_INTENSITY * exposure;
    environment.rotation_radians = 0.0;
    // Keep the user-visible IBL time scoped to staging and artifact restoration. The Viewer
    // builds its temporary project and renderer separately, so including those phases here
    // would misreport renderer startup as an environment-reflection cost.
    let total_elapsed = staging_elapsed.saturating_add(hydration_elapsed);
    println!(
        "loaded staged HDRI environment: status={:?}, staging_elapsed={:.2?}, staging_timing={staging_timing:?}, staging_output={staging_output:?}, total_elapsed={:.2?}, source_face_size={}, source_mip_count={}, staged_pmrem_face_size={}, staged_pmrem_mip_count={}, active_pmrem_face_size={}, active_pmrem_mip_count={}, source={}, derived={}",
        staged.status(),
        staging_elapsed,
        total_elapsed,
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
    Ok(source_cubemap_environment_load(
        environment,
        staged.status(),
        staging_elapsed,
        staging_timing,
        staging_output,
        total_elapsed,
        source_pixel_decode_elapsed,
    ))
}

fn restored_viewer_hdri_environment_load(
    restored: EnvironmentIblSourceStagingRestore,
    exposure: f32,
    restore_elapsed: std::time::Duration,
    source_pixel_decode_elapsed: std::time::Duration,
) -> SourceCubemapEnvironmentLoad {
    let report = restored.report();
    let staging_status = report.status();
    let staging_timing = report.timing();
    let staging_output = report.output();
    let mut environment = restored.into_environment();
    environment.intensity = DEFAULT_ENVIRONMENT_INTENSITY * exposure;
    environment.rotation_radians = 0.0;
    source_cubemap_environment_load(
        environment,
        staging_status,
        restore_elapsed,
        staging_timing,
        staging_output,
        restore_elapsed,
        source_pixel_decode_elapsed,
    )
}

fn source_cubemap_environment_load(
    mut environment: SourceCubemapEnvironment,
    staging_status: EnvironmentIblSourceStagingStatus,
    staging_elapsed: std::time::Duration,
    staging_timing: EnvironmentIblSourceStagingTiming,
    staging_output: EnvironmentIblSourceStagingOutput,
    total_elapsed: std::time::Duration,
    source_pixel_decode_elapsed: std::time::Duration,
) -> SourceCubemapEnvironmentLoad {
    let source_cubemap_face_size = environment.mip_chain.source_face_size();
    let source_cubemap_mip_count = environment.mip_chain.source_mip_count();
    let pmrem_face_size = environment.mip_chain.pmrem_face_size();
    let pmrem_mip_count = environment.mip_chain.pmrem_mip_count();
    environment.rotation_radians = 0.0;

    SourceCubemapEnvironmentLoad {
        environment,
        staging_status,
        staging_elapsed,
        staging_timing,
        staging_output,
        total_elapsed,
        source_pixel_decode_elapsed,
        source_cubemap_face_size,
        source_cubemap_mip_count,
        pmrem_face_size,
        pmrem_mip_count,
    }
}

fn viewer_hdri_exposure_path(source_zcube_path: &Path) -> PathBuf {
    let mut name = source_zcube_path
        .file_name()
        .unwrap_or_default()
        .to_os_string();
    name.push(".viewer-exposure-v1");
    source_zcube_path.with_file_name(name)
}

fn write_cached_hdri_exposure(source_zcube_path: &Path, exposure: f32) -> std::io::Result<()> {
    let contents = format!(
        "schema={VIEWER_HDRI_EXPOSURE_CACHE_SCHEMA}\nexposure_bits={:08x}\n",
        exposure.to_bits()
    );
    atomic_write(
        &viewer_hdri_exposure_path(source_zcube_path),
        contents.as_bytes(),
    )
}

fn read_cached_hdri_exposure(source_zcube_path: &Path) -> Option<f32> {
    let contents = fs::read_to_string(viewer_hdri_exposure_path(source_zcube_path)).ok()?;
    parse_cached_hdri_exposure(&contents)
}

fn parse_cached_hdri_exposure(contents: &str) -> Option<f32> {
    let mut schema = None;
    let mut exposure_bits = None;
    for line in contents.lines() {
        let (key, value) = line.split_once('=')?;
        match key {
            "schema" => schema = Some(value),
            "exposure_bits" => exposure_bits = Some(value),
            _ => return None,
        }
    }
    if schema != Some(VIEWER_HDRI_EXPOSURE_CACHE_SCHEMA) {
        return None;
    }
    let bits = u32::from_str_radix(exposure_bits?, 16).ok()?;
    let exposure = f32::from_bits(bits);
    (exposure.is_finite() && (0.02..=4.0).contains(&exposure)).then_some(exposure)
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

fn validate_equirectangular_dimensions(width: u32, height: u32) -> Result<(), Box<dyn Error>> {
    if width > 0 && height > 0 && height.checked_mul(2) == Some(width) {
        return Ok(());
    }

    Err(
        format!("viewer HDRI must be a non-empty 2:1 equirectangular image, got {width}x{height}")
            .into(),
    )
}

fn sampled_hdri_exposure(image: &DecodedTextureImageRgba32F) -> f32 {
    let step_x = (image.width / 128).max(1);
    let step_y = (image.height / 64).max(1);
    let mut sum = 0.0_f64;
    let mut count = 0_u64;
    let mut y = 0;
    while y < image.height {
        let mut x = 0;
        while x < image.width {
            let pixel = image.rgba[y as usize * image.width as usize + x as usize];
            let luma = luma([pixel[0], pixel[1], pixel[2]]);
            if luma.is_finite() {
                sum += luma;
                count += 1;
            }
            x += step_x;
        }
        y += step_y;
    }
    let average_luma = if count == 0 || !sum.is_finite() {
        0.0001
    } else {
        (sum / count as f64).max(0.0001)
    };
    (0.45 / average_luma).clamp(0.02, 4.0) as f32
}

fn luma(rgb: [f32; 3]) -> f64 {
    rgb[0] as f64 * 0.2126 + rgb[1] as f64 * 0.7152 + rgb[2] as f64 * 0.0722
}

#[cfg(test)]
mod tests {
    use super::{
        parse_cached_hdri_exposure, resolved_face_size, resolved_pmrem_face_size,
        sampled_hdri_exposure, validate_equirectangular_dimensions,
    };
    use zircon_runtime::asset::importer::DecodedTextureImageRgba32F;

    const SOURCE: &str = include_str!("hdri.rs");

    fn production_source() -> &'static str {
        SOURCE
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("HDRI source should retain a test-module boundary")
    }

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

    #[test]
    fn viewer_accepts_standard_equirectangular_dimensions() {
        validate_equirectangular_dimensions(4096, 2048)
            .expect("a 2:1 HDRI should be accepted by the equirectangular viewer path");
    }

    #[test]
    fn viewer_rejects_non_equirectangular_dimensions_before_staging() {
        let error = validate_equirectangular_dimensions(2048, 2048)
            .expect_err("the viewer must not project a square image as equirectangular HDRI");

        assert!(error.to_string().contains("2:1 equirectangular"));
    }

    #[test]
    fn viewer_does_not_treat_saturated_dimensions_as_equirectangular() {
        let error = validate_equirectangular_dimensions(u32::MAX, u32::MAX)
            .expect_err("dimension arithmetic must not accept a saturated 2:1 relation");

        assert!(error.to_string().contains("2:1 equirectangular"));
    }

    #[test]
    fn viewer_exposure_ignores_non_finite_hdr_texels() {
        let image = DecodedTextureImageRgba32F {
            width: 2,
            height: 1,
            rgba: vec![[f32::NAN, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0]],
        };

        let exposure = sampled_hdri_exposure(&image);

        assert!(exposure.is_finite());
        assert!((exposure - 0.45).abs() < 0.0001);
    }

    #[test]
    fn viewer_exposure_has_a_deterministic_fallback_for_all_invalid_texels() {
        let image = DecodedTextureImageRgba32F {
            width: 2,
            height: 1,
            rgba: vec![[f32::NAN, f32::INFINITY, 0.0, 1.0]; 2],
        };

        let exposure = sampled_hdri_exposure(&image);

        assert_eq!(exposure, 4.0);
    }

    #[test]
    fn viewer_hdri_staging_requires_a_runtime_owned_parallel_executor() {
        let source = production_source();
        assert!(source.contains("parallel_executor: &E"));
        assert!(source
            .contains("stage_environment_ibl_source_with_parallel_executor_and_decoded_image("));
        assert!(source.contains("restore_environment_ibl_source_if_current("));
        assert!(source.contains("DecodedTextureImageRgba32F"));
        assert!(source.contains("let staging_started = Instant::now();"));
        assert!(source.contains("staging_elapsed"));
        assert!(
            !source.contains("stage_environment_ibl_source(&context"),
            "the interactive viewer must not fall back to serial IBL staging"
        );
    }

    #[test]
    fn viewer_staging_consumes_the_predecoded_hdr_input_without_a_second_decode() {
        let source = production_source();
        let staging = source
            .split("pub(crate) fn source_cubemap_environment<E>(")
            .nth(1)
            .expect("viewer must retain a dedicated source-cubemap staging owner");

        assert!(source.contains("pub(crate) fn preflight_viewer_hdri("));
        assert!(staging.contains("hdri: ViewerHdriPreflight"));
        assert!(staging.contains("let DecodedViewerHdri { image, exposure }"));
        assert!(staging.contains("let DecodedViewerHdri {"));
        assert!(
            !staging.contains("fs::read("),
            "staging must consume the preflight bytes instead of reading the HDRI again"
        );
        assert!(
            !staging.contains("load_from_memory_with_format"),
            "staging must consume the preflight image instead of decoding it again"
        );
    }

    #[test]
    fn cached_hdri_exposure_requires_the_current_schema_and_finite_clamped_bits() {
        assert_eq!(
            parse_cached_hdri_exposure(
                "schema=zircon_shader_pbr_viewer_hdri_exposure_v1\nexposure_bits=3f000000\n"
            ),
            Some(0.5)
        );
        assert_eq!(
            parse_cached_hdri_exposure(
                "schema=zircon_shader_pbr_viewer_hdri_exposure_v1\nexposure_bits=7fc00000\n"
            ),
            None
        );
        assert_eq!(
            parse_cached_hdri_exposure("schema=old\nexposure_bits=3f000000\n"),
            None
        );
    }

    #[test]
    fn viewer_exposure_sidecar_uses_runtime_atomic_publication() {
        let source = production_source();
        let sidecar_writer = source
            .split("fn write_cached_hdri_exposure(")
            .nth(1)
            .and_then(|writer| writer.split("fn read_cached_hdri_exposure(").next())
            .expect("viewer must retain an exposure sidecar writer");

        assert!(source.contains("core::resource::io::atomic_write"));
        assert!(sidecar_writer.contains("atomic_write("));
        assert!(!sidecar_writer.contains("fs::write("));
    }

    #[test]
    fn valid_ibl_restore_survives_a_missing_viewer_exposure_sidecar() {
        let source = production_source();
        let after_decode = source
            .split("let DecodedViewerHdri { image, exposure } = decode_viewer_hdri(&context)?;")
            .nth(1)
            .expect("viewer must retain a decoded HDRI fallback");
        let restored_environment = after_decode
            .find("if let Some(restored) = restored_without_exposure {")
            .expect("a valid restored bundle must survive a missing exposure sidecar");
        let staging = after_decode
            .find("let staging_started = Instant::now();")
            .expect("a genuine cache miss must retain staging");

        assert!(
            source.contains("let mut restored_without_exposure = None;"),
            "the valid restored bundle must remain available while exposure is decoded"
        );
        assert!(
            source.contains("restored_without_exposure = Some(restored);"),
            "a missing exposure sidecar must not discard a valid restored bundle"
        );
        assert!(
            restored_environment < staging,
            "the restored bundle must be returned before cache-miss staging can run"
        );
    }

    #[test]
    fn viewer_ibl_total_time_excludes_unrelated_startup_and_exposure_sidecar_io() {
        let source = production_source();
        let staging = source
            .split("pub(crate) fn source_cubemap_environment<E>(")
            .nth(1)
            .expect("viewer must retain a dedicated source-cubemap staging owner");
        let exposure_write = staging
            .find("write_cached_hdri_exposure(source_zcube_path, exposure)")
            .expect("the cold staging path must persist exposure separately");
        let hydration_started = staging
            .find("let hydration_started = Instant::now();")
            .expect("artifact hydration must have an explicit timing boundary");

        assert!(
            exposure_write < hydration_started,
            "Viewer-local exposure persistence must not inflate artifact hydration"
        );
        assert!(staging.contains("let hydration_elapsed = hydration_started.elapsed();"));
        assert!(staging
            .contains("let total_elapsed = staging_elapsed.saturating_add(hydration_elapsed);"));
        assert!(!staging.contains("let total_elapsed = staging_started.elapsed();"));
        assert!(!source.contains("load_started_at"));
    }
}
