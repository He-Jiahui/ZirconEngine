use std::path::Path;

use crate::asset::{AssetImportContext, AssetImportError};

use super::gltf_meshopt::{buffer_is_meshopt_fallback, decode_meshopt_views};

const WEBP_MIME_TYPE: &str = "image/webp";
const RUNTIME_SUPPORTED_REQUIRED_EXTENSIONS: &[&str] = &[
    "EXT_meshopt_compression",
    "EXT_texture_webp",
    "KHR_mesh_quantization",
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
    validate_required_extensions(&json.extensions_required)?;
    json.extensions_required
        .retain(|extension| !RUNTIME_SUPPORTED_REQUIRED_EXTENSIONS.contains(&extension.as_str()));
    let document = gltf::Document::from_json(json)
        .map_err(|error| gltf_parse_error(format!("validate gltf: {error}")))?;
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
            let source = buffer.source();
            let source_name = match &source {
                gltf::buffer::Source::Bin => "the GLB binary chunk".to_string(),
                gltf::buffer::Source::Uri(uri)
                    if uri
                        .get(..5)
                        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:")) =>
                {
                    "an embedded data URI".to_string()
                }
                gltf::buffer::Source::Uri(uri) => format!("`{uri}`"),
            };
            gltf::buffer::Data::from_source_and_blob(source, Some(base_dir), &mut blob).map_err(
                |error| {
                    gltf_parse_error(format!(
                        "load gltf Buffer{} from {source_name}: {error}",
                        buffer.index()
                    ))
                },
            )?
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
