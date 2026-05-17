use zircon_runtime::asset::{AssetImportContext, AssetImportError};
use zircon_runtime::core::framework::render::RenderImageDimension;

pub(crate) struct TextureContainerInfo {
    pub(crate) format: String,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) dimension: RenderImageDimension,
    pub(crate) depth_or_array_layers: u32,
    pub(crate) mip_count: u32,
    pub(crate) array_layers: u32,
}

pub(crate) fn parse_container_info(
    context: &AssetImportContext,
) -> Result<TextureContainerInfo, AssetImportError> {
    let extension = context
        .source_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "dds" => parse_dds(context),
        "ktx" => parse_ktx1(context),
        "ktx2" => parse_ktx2(context),
        "astc" => parse_astc(context),
        _ => Err(AssetImportError::UnsupportedFormat(format!(
            "texture container importer does not handle {}",
            context.source_path.display()
        ))),
    }
}

fn parse_dds(context: &AssetImportContext) -> Result<TextureContainerInfo, AssetImportError> {
    let bytes = &context.source_bytes;
    require_len(context, 128, "dds header")?;
    if &bytes[..4] != b"DDS " {
        return parse_error(context, "dds header missing DDS magic");
    }
    if read_u32_le(context, 4)? != 124 {
        return parse_error(context, "dds header size must be 124 bytes");
    }

    let height = read_nonzero_u32(context, 12, "dds height")?;
    let width = read_nonzero_u32(context, 16, "dds width")?;
    let mip_count = read_u32_le(context, 28)?.max(1);
    let pixel_format_size = read_u32_le(context, 76)?;
    if pixel_format_size != 32 {
        return parse_error(context, "dds pixel format size must be 32 bytes");
    }

    let fourcc = fourcc_string(&bytes[84..88]);
    let caps2 = read_u32_le(context, 112)?;
    let is_cubemap = caps2 & DDSCAPS2_CUBEMAP != 0;
    let (format, array_layers) = if fourcc.as_deref() == Some("DX10") {
        require_len(context, 148, "dds dx10 header")?;
        let dxgi_format = read_u32_le(context, 128)?;
        let array_size = read_u32_le(context, 140)?.max(1);
        let faces = if is_cubemap { 6 } else { 1 };
        let array_layers =
            checked_layer_count(context, "dds dx10 array layer count", array_size, faces)?;
        (format!("dds/dxgi-{dxgi_format}"), array_layers)
    } else {
        let format = fourcc
            .map(|fourcc| format!("dds/{fourcc}"))
            .unwrap_or_else(|| "dds/uncompressed".to_string());
        let layers = if is_cubemap { 6 } else { 1 };
        (format, layers)
    };

    Ok(TextureContainerInfo {
        format,
        width,
        height,
        dimension: RenderImageDimension::D2,
        depth_or_array_layers: array_layers,
        mip_count,
        array_layers,
    })
}

fn parse_ktx1(context: &AssetImportContext) -> Result<TextureContainerInfo, AssetImportError> {
    let bytes = &context.source_bytes;
    require_len(context, 64, "ktx header")?;
    if &bytes[..12] != KTX1_IDENTIFIER {
        return parse_error(context, "ktx header missing KTX 1 identifier");
    }
    if read_u32_le(context, 12)? != KTX_LITTLE_ENDIAN {
        return parse_error(context, "only little-endian KTX 1 files are supported");
    }

    let gl_internal_format = read_u32_le(context, 28)?;
    let width = read_nonzero_u32(context, 36, "ktx width")?;
    let raw_height = read_u32_le(context, 40)?;
    let raw_depth = read_u32_le(context, 44)?;
    let height = raw_height.max(1);
    let array_elements = read_u32_le(context, 48)?.max(1);
    let faces = read_u32_le(context, 52)?.max(1);
    let mip_count = read_u32_le(context, 56)?.max(1);
    let dimension = texture_dimension_from_header(raw_height, raw_depth);
    let parsed_layers =
        checked_layer_count(context, "ktx array layer count", array_elements, faces)?;
    let array_layers = texture_array_layers(dimension, parsed_layers);

    Ok(TextureContainerInfo {
        format: format!("ktx/gl-internal-0x{gl_internal_format:08x}"),
        width,
        height,
        dimension,
        depth_or_array_layers: texture_depth_or_array_layers(dimension, raw_depth, array_layers),
        mip_count,
        array_layers,
    })
}

fn parse_ktx2(context: &AssetImportContext) -> Result<TextureContainerInfo, AssetImportError> {
    let bytes = &context.source_bytes;
    require_len(context, 68, "ktx2 header")?;
    if &bytes[..12] != KTX2_IDENTIFIER {
        return parse_error(context, "ktx2 header missing KTX 2 identifier");
    }

    let vk_format = read_u32_le(context, 12)?;
    let width = read_nonzero_u32(context, 20, "ktx2 width")?;
    let raw_height = read_u32_le(context, 24)?;
    let raw_depth = read_u32_le(context, 28)?;
    let height = raw_height.max(1);
    let layer_count = read_u32_le(context, 32)?.max(1);
    let face_count = read_u32_le(context, 36)?.max(1);
    let level_count = read_u32_le(context, 40)?.max(1);
    let supercompression = read_u32_le(context, 44)?;
    let dimension = texture_dimension_from_header(raw_height, raw_depth);
    let parsed_layers =
        checked_layer_count(context, "ktx2 array layer count", layer_count, face_count)?;
    let array_layers = texture_array_layers(dimension, parsed_layers);

    Ok(TextureContainerInfo {
        format: format!("ktx2/vk-{vk_format}/supercompression-{supercompression}"),
        width,
        height,
        dimension,
        depth_or_array_layers: texture_depth_or_array_layers(dimension, raw_depth, array_layers),
        mip_count: level_count,
        array_layers,
    })
}

fn parse_astc(context: &AssetImportContext) -> Result<TextureContainerInfo, AssetImportError> {
    let bytes = &context.source_bytes;
    require_len(context, 16, "astc header")?;
    if &bytes[..4] != ASTC_MAGIC {
        return parse_error(context, "astc header missing ASTC magic");
    }

    let block_x = bytes[4].max(1);
    let block_y = bytes[5].max(1);
    let block_z = bytes[6].max(1);
    let width = read_u24_le(bytes, 7).max(1);
    let height = read_u24_le(bytes, 10).max(1);
    let depth = read_u24_le(bytes, 13).max(1);
    let dimension = if depth > 1 || block_z > 1 {
        RenderImageDimension::D3
    } else {
        RenderImageDimension::D2
    };

    Ok(TextureContainerInfo {
        format: format!("astc/{block_x}x{block_y}x{block_z}"),
        width,
        height,
        dimension,
        depth_or_array_layers: texture_depth_or_array_layers(dimension, depth, depth),
        mip_count: 1,
        array_layers: astc_array_layers(dimension, depth),
    })
}

fn texture_dimension_from_header(height: u32, depth: u32) -> RenderImageDimension {
    if depth > 0 {
        RenderImageDimension::D3
    } else if height == 0 {
        RenderImageDimension::D1
    } else {
        RenderImageDimension::D2
    }
}

fn texture_depth_or_array_layers(
    dimension: RenderImageDimension,
    depth: u32,
    array_layers: u32,
) -> u32 {
    if dimension == RenderImageDimension::D3 {
        depth.max(1)
    } else {
        array_layers.max(1)
    }
}

fn texture_array_layers(dimension: RenderImageDimension, array_layers: u32) -> u32 {
    if dimension == RenderImageDimension::D3 {
        1
    } else {
        array_layers.max(1)
    }
}

fn astc_array_layers(dimension: RenderImageDimension, depth: u32) -> u32 {
    if dimension == RenderImageDimension::D3 {
        1
    } else {
        depth.max(1)
    }
}

fn checked_layer_count(
    context: &AssetImportContext,
    label: &str,
    layers: u32,
    faces: u32,
) -> Result<u32, AssetImportError> {
    layers
        .checked_mul(faces)
        .filter(|value| *value > 0)
        .ok_or_else(|| parse_error_value(context, format!("{label} overflows u32")))
}

fn require_len(
    context: &AssetImportContext,
    required: usize,
    label: &str,
) -> Result<(), AssetImportError> {
    if context.source_bytes.len() < required {
        return parse_error(
            context,
            format!(
                "{label} requires at least {required} bytes, got {}",
                context.source_bytes.len()
            ),
        );
    }
    Ok(())
}

fn read_nonzero_u32(
    context: &AssetImportContext,
    offset: usize,
    label: &str,
) -> Result<u32, AssetImportError> {
    let value = read_u32_le(context, offset)?;
    if value == 0 {
        return parse_error(context, format!("{label} must be nonzero"));
    }
    Ok(value)
}

fn read_u32_le(context: &AssetImportContext, offset: usize) -> Result<u32, AssetImportError> {
    let bytes = context
        .source_bytes
        .get(offset..offset + 4)
        .ok_or_else(|| parse_error_value(context, format!("missing u32 at byte {offset}")))?;
    Ok(u32::from_le_bytes(
        bytes
            .try_into()
            .expect("slice length checked before conversion"),
    ))
}

fn read_u24_le(bytes: &[u8], offset: usize) -> u32 {
    u32::from(bytes[offset])
        | (u32::from(bytes[offset + 1]) << 8)
        | (u32::from(bytes[offset + 2]) << 16)
}

fn fourcc_string(bytes: &[u8]) -> Option<String> {
    if bytes.iter().all(|byte| *byte == 0) {
        return None;
    }
    let fourcc = String::from_utf8_lossy(bytes)
        .trim_end_matches('\0')
        .to_string();
    if fourcc.is_empty() {
        None
    } else {
        Some(fourcc)
    }
}

fn parse_error<T>(
    context: &AssetImportContext,
    message: impl Into<String>,
) -> Result<T, AssetImportError> {
    Err(parse_error_value(context, message))
}

fn parse_error_value(context: &AssetImportContext, message: impl Into<String>) -> AssetImportError {
    AssetImportError::Parse(format!(
        "parse texture container {}: {}",
        context.source_path.display(),
        message.into()
    ))
}

const DDSCAPS2_CUBEMAP: u32 = 0x0000_0200;
const KTX_LITTLE_ENDIAN: u32 = 0x0403_0201;
const KTX1_IDENTIFIER: &[u8] = b"\xABKTX 11\xBB\r\n\x1A\n";
const KTX2_IDENTIFIER: &[u8] = b"\xABKTX 20\xBB\r\n\x1A\n";
const ASTC_MAGIC: &[u8] = b"\x13\xAB\xA1\x5C";

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::asset::{AssetImportContext, AssetUri, ImportedAsset, TexturePayload};

    #[test]
    fn dds_container_importer_preserves_compressed_payload() {
        let imported = import_container_fixture("albedo.dds", tiny_dds_bytes());

        match imported {
            ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 8);
                assert_eq!(texture.height, 4);
                assert!(texture.rgba.is_empty());
                assert_eq!(texture.render_image_descriptor().format, "dds/DXT1");
                match texture.payload {
                    TexturePayload::Container {
                        format,
                        bytes,
                        mip_count,
                        array_layers,
                    } => {
                        assert_eq!(format, "dds/DXT1");
                        assert_eq!(bytes.len(), 128);
                        assert_eq!(mip_count, 3);
                        assert_eq!(array_layers, 1);
                    }
                    other => panic!("unexpected texture payload: {other:?}"),
                }
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn dds_dx10_container_importer_reads_cubemap_array_layers() {
        let imported = import_container_fixture("skybox.dds", tiny_dds_dx10_cubemap_array_bytes());

        match imported {
            ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 32);
                assert_eq!(texture.height, 16);
                let descriptor = texture.render_image_descriptor();
                assert_eq!(descriptor.format, "dds/dxgi-98");
                assert_eq!(descriptor.dimension, RenderImageDimension::D2);
                assert_eq!(descriptor.depth_or_array_layers, 12);
                assert_eq!(descriptor.mip_count, 5);
                assert_eq!(descriptor.array_layer_count, 12);
                match texture.payload {
                    TexturePayload::Container {
                        format,
                        bytes,
                        mip_count,
                        array_layers,
                    } => {
                        assert_eq!(format, "dds/dxgi-98");
                        assert_eq!(bytes.len(), 148);
                        assert_eq!(mip_count, 5);
                        assert_eq!(array_layers, 12);
                    }
                    other => panic!("unexpected texture payload: {other:?}"),
                }
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn container_importer_applies_descriptor_settings_without_expanding_payload() {
        let imported = import_container_fixture_with_settings(
            "albedo.dds",
            tiny_dds_bytes(),
            r#"
texture_format = "rgba16float"
sampler = "nearest"
asset_usage = "render_world"
"#,
        );

        match imported {
            ImportedAsset::Texture(texture) => {
                assert!(texture.rgba.is_empty());
                let descriptor = texture.render_image_descriptor();
                assert_eq!(descriptor.format, "rgba16float");
                assert_eq!(descriptor.depth_or_array_layers, 1);
                assert_eq!(
                    descriptor.sampler.mag_filter,
                    zircon_runtime::core::framework::render::RenderSamplerFilter::Nearest
                );
                assert_eq!(
                    descriptor.sampler.min_filter,
                    zircon_runtime::core::framework::render::RenderSamplerFilter::Nearest
                );
                assert_eq!(
                    descriptor.asset_usage,
                    vec![
                        zircon_runtime::core::framework::render::RenderImageAssetUsage::RenderWorld
                    ]
                );
                match texture.payload {
                    TexturePayload::Container {
                        format,
                        mip_count,
                        array_layers,
                        ..
                    } => {
                        assert_eq!(format, "dds/DXT1");
                        assert_eq!(mip_count, 3);
                        assert_eq!(array_layers, 1);
                    }
                    other => panic!("unexpected texture payload: {other:?}"),
                }
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn container_importer_rejects_array_layout_without_decoded_rgba() {
        let error = import_container_error_with_settings(
            "albedo.dds",
            tiny_dds_bytes(),
            r#"
[array_layout]
row_count = 2
"#,
        );

        assert!(
            error.contains("texture import setting `array_layout` requires a decoded rgba8 image"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn ktx1_container_importer_reads_1d_dimension() {
        let imported = import_container_fixture("strip.ktx", tiny_ktx1_1d_bytes());

        match imported {
            ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 32);
                assert_eq!(texture.height, 1);
                let descriptor = texture.render_image_descriptor();
                assert_eq!(descriptor.dimension, RenderImageDimension::D1);
                assert_eq!(descriptor.depth_or_array_layers, 1);
                assert_eq!(descriptor.array_layer_count, 1);
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn ktx1_3d_container_keeps_depth_separate_from_array_layers() {
        let mut bytes = ktx1_layer_face_bytes(2, 6);
        write_u32(&mut bytes, 40, 8);
        write_u32(&mut bytes, 44, 5);
        let imported = import_container_fixture("volume.ktx", bytes);

        match imported {
            ImportedAsset::Texture(texture) => {
                let descriptor = texture.render_image_descriptor();
                assert_eq!(descriptor.dimension, RenderImageDimension::D3);
                assert_eq!(descriptor.depth_or_array_layers, 5);
                assert_eq!(descriptor.array_layer_count, 1);
                match texture.payload {
                    TexturePayload::Container { array_layers, .. } => {
                        assert_eq!(array_layers, 1);
                    }
                    other => panic!("unexpected texture payload: {other:?}"),
                }
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn ktx2_container_importer_reads_layers_faces_and_mips() {
        let imported = import_container_fixture("array.ktx2", tiny_ktx2_bytes());

        match imported {
            ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 16);
                assert_eq!(texture.height, 8);
                let descriptor = texture.render_image_descriptor();
                assert_eq!(descriptor.dimension, RenderImageDimension::D2);
                assert_eq!(descriptor.depth_or_array_layers, 12);
                assert_eq!(descriptor.mip_count, 4);
                assert_eq!(descriptor.array_layer_count, 12);
                match texture.payload {
                    TexturePayload::Container {
                        format,
                        mip_count,
                        array_layers,
                        ..
                    } => {
                        assert_eq!(format, "ktx2/vk-37/supercompression-1");
                        assert_eq!(mip_count, 4);
                        assert_eq!(array_layers, 12);
                    }
                    other => panic!("unexpected texture payload: {other:?}"),
                }
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn ktx2_container_importer_reads_3d_dimension() {
        let imported = import_container_fixture("volume.ktx2", tiny_ktx2_3d_bytes());

        match imported {
            ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 16);
                assert_eq!(texture.height, 8);
                let descriptor = texture.render_image_descriptor();
                assert_eq!(descriptor.dimension, RenderImageDimension::D3);
                assert_eq!(descriptor.depth_or_array_layers, 5);
                assert_eq!(descriptor.array_layer_count, 1);
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn ktx2_3d_container_keeps_depth_separate_from_array_layers() {
        let mut bytes = ktx2_layer_face_bytes(2, 6);
        write_u32(&mut bytes, 28, 5);
        let imported = import_container_fixture("volume-array.ktx2", bytes);

        match imported {
            ImportedAsset::Texture(texture) => {
                let descriptor = texture.render_image_descriptor();
                assert_eq!(descriptor.dimension, RenderImageDimension::D3);
                assert_eq!(descriptor.depth_or_array_layers, 5);
                assert_eq!(descriptor.array_layer_count, 1);
                match texture.payload {
                    TexturePayload::Container { array_layers, .. } => {
                        assert_eq!(array_layers, 1);
                    }
                    other => panic!("unexpected texture payload: {other:?}"),
                }
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn astc_container_importer_reads_block_and_size() {
        let imported = import_container_fixture("tile.astc", tiny_astc_bytes());

        match imported {
            ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 32);
                assert_eq!(texture.height, 16);
                let descriptor = texture.render_image_descriptor();
                assert_eq!(descriptor.format, "astc/6x6x1");
                assert_eq!(descriptor.dimension, RenderImageDimension::D2);
                assert_eq!(descriptor.depth_or_array_layers, 1);
                match texture.payload {
                    TexturePayload::Container {
                        format,
                        mip_count,
                        array_layers,
                        ..
                    } => {
                        assert_eq!(format, "astc/6x6x1");
                        assert_eq!(mip_count, 1);
                        assert_eq!(array_layers, 1);
                    }
                    other => panic!("unexpected texture payload: {other:?}"),
                }
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn astc_container_importer_reads_3d_block_and_depth() {
        let imported = import_container_fixture("volume.astc", tiny_astc_3d_bytes());

        match imported {
            ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 32);
                assert_eq!(texture.height, 16);
                let descriptor = texture.render_image_descriptor();
                assert_eq!(descriptor.format, "astc/6x6x4");
                assert_eq!(descriptor.dimension, RenderImageDimension::D3);
                assert_eq!(descriptor.depth_or_array_layers, 8);
                assert_eq!(descriptor.array_layer_count, 1);
                match texture.payload {
                    TexturePayload::Container {
                        format,
                        mip_count,
                        array_layers,
                        ..
                    } => {
                        assert_eq!(format, "astc/6x6x4");
                        assert_eq!(mip_count, 1);
                        assert_eq!(array_layers, 1);
                    }
                    other => panic!("unexpected texture payload: {other:?}"),
                }
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn container_importer_reports_invalid_header_diagnostics() {
        let cases = [
            ("broken.dds", vec![0; 128], "dds header missing DDS magic"),
            (
                "broken.ktx",
                vec![0; 64],
                "ktx header missing KTX 1 identifier",
            ),
            (
                "broken.ktx2",
                vec![0; 68],
                "ktx2 header missing KTX 2 identifier",
            ),
            ("broken.astc", vec![0; 16], "astc header missing ASTC magic"),
        ];

        for (path, bytes, expected) in cases {
            let error = import_container_error(path, bytes);
            assert!(
                error.contains(expected),
                "expected `{expected}` in `{error}`"
            );
        }
    }

    #[test]
    fn container_importer_reports_layer_count_overflow_diagnostics() {
        let cases = [
            (
                "overflow.dds",
                dds_dx10_cubemap_array_bytes(u32::MAX),
                "dds dx10 array layer count overflows u32",
            ),
            (
                "overflow.ktx",
                ktx1_layer_face_bytes(u32::MAX, 6),
                "ktx array layer count overflows u32",
            ),
            (
                "overflow.ktx2",
                ktx2_layer_face_bytes(u32::MAX, 6),
                "ktx2 array layer count overflows u32",
            ),
        ];

        for (path, bytes, expected) in cases {
            let error = import_container_error(path, bytes);
            assert!(
                error.contains(expected),
                "expected `{expected}` in `{error}`"
            );
        }
    }

    fn import_container_fixture(path: &str, bytes: Vec<u8>) -> ImportedAsset {
        import_container_fixture_with_settings(path, bytes, "")
    }

    fn import_container_fixture_with_settings(
        path: &str,
        bytes: Vec<u8>,
        settings: &str,
    ) -> ImportedAsset {
        let report = crate::plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new(path))
            .unwrap();
        let uri = format!("res://textures/{path}");
        let settings = settings.parse().expect("valid texture import settings");
        let context =
            AssetImportContext::new(path.into(), AssetUri::parse(&uri).unwrap(), bytes, settings);
        importer
            .import(&context)
            .unwrap()
            .root_entry()
            .expect("root texture asset entry")
            .asset
            .clone()
    }

    fn import_container_error(path: &str, bytes: Vec<u8>) -> String {
        import_container_error_with_settings(path, bytes, "")
    }

    fn import_container_error_with_settings(path: &str, bytes: Vec<u8>, settings: &str) -> String {
        let report = crate::plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new(path))
            .unwrap();
        let uri = format!("res://textures/{path}");
        let settings = settings.parse().expect("valid texture import settings");
        let context =
            AssetImportContext::new(path.into(), AssetUri::parse(&uri).unwrap(), bytes, settings);
        importer.import(&context).unwrap_err().to_string()
    }

    fn tiny_dds_bytes() -> Vec<u8> {
        let mut bytes = vec![0; 128];
        bytes[0..4].copy_from_slice(b"DDS ");
        write_u32(&mut bytes, 4, 124);
        write_u32(&mut bytes, 12, 4);
        write_u32(&mut bytes, 16, 8);
        write_u32(&mut bytes, 28, 3);
        write_u32(&mut bytes, 76, 32);
        bytes[84..88].copy_from_slice(b"DXT1");
        bytes
    }

    fn tiny_dds_dx10_cubemap_array_bytes() -> Vec<u8> {
        dds_dx10_cubemap_array_bytes(2)
    }

    fn dds_dx10_cubemap_array_bytes(array_size: u32) -> Vec<u8> {
        let mut bytes = tiny_dds_bytes();
        bytes.resize(148, 0);
        write_u32(&mut bytes, 12, 16);
        write_u32(&mut bytes, 16, 32);
        write_u32(&mut bytes, 28, 5);
        bytes[84..88].copy_from_slice(b"DX10");
        write_u32(&mut bytes, 112, DDSCAPS2_CUBEMAP);
        write_u32(&mut bytes, 128, 98);
        write_u32(&mut bytes, 140, array_size);
        bytes
    }

    fn tiny_ktx1_1d_bytes() -> Vec<u8> {
        ktx1_layer_face_bytes(0, 1)
    }

    fn ktx1_layer_face_bytes(array_elements: u32, faces: u32) -> Vec<u8> {
        let mut bytes = vec![0; 64];
        bytes[0..12].copy_from_slice(KTX1_IDENTIFIER);
        write_u32(&mut bytes, 12, KTX_LITTLE_ENDIAN);
        write_u32(&mut bytes, 28, 0x8058);
        write_u32(&mut bytes, 36, 32);
        write_u32(&mut bytes, 40, 0);
        write_u32(&mut bytes, 44, 0);
        write_u32(&mut bytes, 48, array_elements);
        write_u32(&mut bytes, 52, faces);
        write_u32(&mut bytes, 56, 1);
        bytes
    }

    fn tiny_ktx2_bytes() -> Vec<u8> {
        ktx2_layer_face_bytes(2, 6)
    }

    fn ktx2_layer_face_bytes(layer_count: u32, face_count: u32) -> Vec<u8> {
        let mut bytes = vec![0; 68];
        bytes[0..12].copy_from_slice(KTX2_IDENTIFIER);
        write_u32(&mut bytes, 12, 37);
        write_u32(&mut bytes, 16, 1);
        write_u32(&mut bytes, 20, 16);
        write_u32(&mut bytes, 24, 8);
        write_u32(&mut bytes, 32, layer_count);
        write_u32(&mut bytes, 36, face_count);
        write_u32(&mut bytes, 40, 4);
        write_u32(&mut bytes, 44, 1);
        bytes
    }

    fn tiny_ktx2_3d_bytes() -> Vec<u8> {
        let mut bytes = tiny_ktx2_bytes();
        write_u32(&mut bytes, 28, 5);
        write_u32(&mut bytes, 32, 0);
        write_u32(&mut bytes, 36, 1);
        write_u32(&mut bytes, 40, 1);
        bytes
    }

    fn tiny_astc_bytes() -> Vec<u8> {
        let mut bytes = vec![0; 16];
        bytes[0..4].copy_from_slice(ASTC_MAGIC);
        bytes[4] = 6;
        bytes[5] = 6;
        bytes[6] = 1;
        write_u24(&mut bytes, 7, 32);
        write_u24(&mut bytes, 10, 16);
        write_u24(&mut bytes, 13, 1);
        bytes
    }

    fn tiny_astc_3d_bytes() -> Vec<u8> {
        let mut bytes = tiny_astc_bytes();
        bytes[6] = 4;
        write_u24(&mut bytes, 13, 8);
        bytes
    }

    fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u24(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset] = (value & 0xff) as u8;
        bytes[offset + 1] = ((value >> 8) & 0xff) as u8;
        bytes[offset + 2] = ((value >> 16) & 0xff) as u8;
    }
}
