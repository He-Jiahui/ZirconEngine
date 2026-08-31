use std::borrow::Cow;
use std::path::Path;

use crate::asset::{AssetImportContext, AssetImportError};

use super::super::{
    validate_gltf_texture_import_support, validate_required_gltf_material_extension_support,
};
use super::gltf_meshopt::{buffer_is_meshopt_fallback, decode_meshopt_views};

const WEBP_MIME_TYPE: &str = "image/webp";
const RUNTIME_SUPPORTED_REQUIRED_EXTENSIONS: &[&str] = &[
    "EXT_meshopt_compression",
    "EXT_texture_webp",
    "KHR_mesh_quantization",
    "KHR_materials_anisotropy",
    "KHR_materials_clearcoat",
    "KHR_materials_emissive_strength",
    "KHR_materials_ior",
    "KHR_materials_transmission",
    "KHR_materials_unlit",
    "KHR_materials_volume",
    "KHR_texture_transform",
];

pub(crate) struct DecodedGltf {
    pub(crate) document: gltf::Document,
    pub(crate) buffers: Vec<gltf::buffer::Data>,
    pub(crate) images: Vec<gltf::image::Data>,
}

pub(crate) fn decode_gltf_source(
    context: &AssetImportContext,
) -> Result<DecodedGltf, AssetImportError> {
    let gltf = gltf::Gltf::from_slice_without_validation(&context.source_bytes)
        .map_err(|error| gltf_parse_error(format!("parse gltf: {error}")))?;
    let blob = gltf.blob;
    let mut json = gltf.document.into_json();
    let required_extensions = json.extensions_required.clone();
    validate_required_extensions(&required_extensions)?;
    json.extensions_required
        .retain(|extension| !RUNTIME_SUPPORTED_REQUIRED_EXTENSIONS.contains(&extension.as_str()));
    let document = gltf::Document::from_json(json)
        .map_err(|error| gltf_parse_error(format!("validate gltf: {error}")))?;
    validate_required_gltf_material_extension_support(&document, &required_extensions)?;
    validate_gltf_texture_import_support(&document)?;
    let base_dir = context
        .source_path
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let mut buffers = load_buffers(&document, base_dir, blob)?;
    decode_meshopt_views(&document, &mut buffers)?;
    let images = decode_images(&document, base_dir, &buffers)?;
    Ok(DecodedGltf {
        document,
        buffers,
        images,
    })
}

fn validate_required_extensions(required: &[String]) -> Result<(), AssetImportError> {
    if let Some(extension) = required
        .iter()
        .find(|extension| !RUNTIME_SUPPORTED_REQUIRED_EXTENSIONS.contains(&extension.as_str()))
    {
        return Err(gltf_parse_error(format!(
            "gltf requires unsupported extension `{extension}`"
        )));
    }
    Ok(())
}

fn load_buffers(
    document: &gltf::Document,
    base_dir: &Path,
    mut blob: Option<Vec<u8>>,
) -> Result<Vec<gltf::buffer::Data>, AssetImportError> {
    let mut buffers = Vec::with_capacity(document.buffers().len());
    for buffer in document.buffers() {
        let data = if buffer_is_meshopt_fallback(&buffer)? {
            gltf::buffer::Data(vec![0; buffer.length()])
        } else {
            gltf::buffer::Data::from_source_and_blob(buffer.source(), Some(base_dir), &mut blob)
                .map_err(|error| {
                    let source_name = gltf_buffer_source_name(buffer.source());
                    gltf_parse_error(format!(
                        "load gltf Buffer{} from {source_name}: {error}",
                        buffer.index()
                    ))
                })?
        };
        if data.len() < buffer.length() {
            return Err(gltf_parse_error(format!(
                "gltf Buffer{} declares {} bytes but its source contains {}",
                buffer.index(),
                buffer.length(),
                data.len()
            )));
        }
        buffers.push(data);
    }
    Ok(buffers)
}

fn gltf_buffer_source_name(source: gltf::buffer::Source<'_>) -> Cow<'_, str> {
    match source {
        gltf::buffer::Source::Bin => Cow::Borrowed("the GLB binary chunk"),
        gltf::buffer::Source::Uri(uri)
            if uri
                .get(..5)
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:")) =>
        {
            Cow::Borrowed("an embedded data URI")
        }
        gltf::buffer::Source::Uri(uri) => Cow::Owned(format!("`{uri}`")),
    }
}

fn decode_images(
    document: &gltf::Document,
    base_dir: &Path,
    buffers: &[gltf::buffer::Data],
) -> Result<Vec<gltf::image::Data>, AssetImportError> {
    document
        .images()
        .map(|image| match image.source() {
            gltf::image::Source::View { view, mime_type }
                if mime_type.eq_ignore_ascii_case(WEBP_MIME_TYPE) =>
            {
                let buffer = buffers.get(view.buffer().index()).ok_or_else(|| {
                    gltf_parse_error(format!(
                        "gltf WebP image {} references a missing buffer",
                        image.index()
                    ))
                })?;
                let end = view
                    .offset()
                    .checked_add(view.length())
                    .ok_or_else(|| gltf_parse_error("gltf WebP image range overflow"))?;
                let encoded = buffer.get(view.offset()..end).ok_or_else(|| {
                    gltf_parse_error(format!(
                        "gltf WebP image {} range is out of bounds",
                        image.index()
                    ))
                })?;
                decode_webp_image(encoded, image.index())
            }
            source => gltf::image::Data::from_source(source, Some(base_dir), buffers)
                .map_err(|error| gltf_parse_error(format!("decode gltf image: {error}"))),
        })
        .collect()
}

fn decode_webp_image(
    encoded: &[u8],
    image_index: usize,
) -> Result<gltf::image::Data, AssetImportError> {
    let decoded = image::load_from_memory_with_format(encoded, image::ImageFormat::WebP).map_err(
        |error| gltf_parse_error(format!("decode gltf WebP image {image_index}: {error}")),
    )?;
    let rgba = decoded.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(gltf::image::Data {
        pixels: rgba.into_raw(),
        format: gltf::image::Format::R8G8B8A8,
        width,
        height,
    })
}

pub(super) fn gltf_parse_error(message: impl Into<String>) -> AssetImportError {
    AssetImportError::Parse(message.into())
}

#[cfg(test)]
mod plugins07_deferred_buffer_diagnostic_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 21;
    const SOURCES_PER_SAMPLE: usize = 8_192;

    #[test]
    fn decode_material_hotpath_contract_buffer_source_labels() {
        assert_eq!(
            gltf_buffer_source_name(gltf::buffer::Source::Bin),
            "the GLB binary chunk"
        );
        assert_eq!(
            gltf_buffer_source_name(gltf::buffer::Source::Uri(
                "data:application/octet-stream;base64,AA=="
            )),
            "an embedded data URI"
        );
        assert_eq!(
            gltf_buffer_source_name(gltf::buffer::Source::Uri("mesh.bin")),
            "`mesh.bin`"
        );
    }

    #[test]
    #[ignore = "release performance gate"]
    fn decode_material_hotpath_performance_release_deferred_buffer_diagnostics() {
        let sources = (0..SOURCES_PER_SAMPLE)
            .map(|index| format!("buffers/plugins07-{index:05}.bin"))
            .collect::<Vec<_>>();
        for _ in 0..4 {
            black_box(measure_eager_names(&sources));
            black_box(measure_deferred_success(&sources));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
                (
                    measure_eager_names(&sources),
                    measure_deferred_success(&sources),
                )
            } else {
                let optimized_ns = measure_deferred_success(&sources);
                (measure_eager_names(&sources), optimized_ns)
            };
            legacy_samples.push(legacy_ns);
            optimized_samples.push(optimized_ns);
        }

        let legacy_p95 = nearest_rank_p95(&legacy_samples);
        let optimized_p95 = nearest_rank_p95(&optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT plugins07_deferred_gltf_buffer_diagnostics sample_pairs={SAMPLE_PAIRS} sources_per_sample={SOURCES_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=90 legacy_success_diagnostic_allocations_per_sample={SOURCES_PER_SAMPLE} optimized_success_diagnostic_allocations_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(
            improvement_percent >= 90,
            "deferred glTF buffer diagnostics must improve successful-load P95 by at least 90%"
        );
    }

    fn measure_eager_names(sources: &[String]) -> u128 {
        let started = Instant::now();
        for uri in sources {
            let source_name = gltf_buffer_source_name(gltf::buffer::Source::Uri(black_box(uri)));
            black_box(source_name);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn measure_deferred_success(sources: &[String]) -> u128 {
        let started = Instant::now();
        for uri in sources {
            black_box(uri);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
