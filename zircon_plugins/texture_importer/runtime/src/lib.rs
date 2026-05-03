use image::GenericImageView;
use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetKind,
    DiagnosticOnlyAssetImporter, FunctionAssetImporter, ImportedAsset, TextureAsset,
    TexturePayload,
};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::PluginModuleManifest,
    plugin::PluginPackageManifest, plugin::ProjectPluginSelection,
    plugin::RuntimeExtensionRegistry, plugin::RuntimeExtensionRegistryError,
    plugin::RuntimePluginRegistrationReport, RuntimeTargetMode,
};

pub const PLUGIN_ID: &str = "texture_importer";
pub const RUNTIME_CRATE_NAME: &str = "zircon_plugin_texture_importer_runtime";
pub const MODULE_NAME: &str = "TextureImporterModule";
pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.texture_importer";
pub const IMAGE_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.texture.image";
pub const CONTAINER_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.texture.container";
pub const PSD_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.texture.psd";

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[
        RUNTIME_CAPABILITY,
        IMAGE_IMPORTER_CAPABILITY,
        CONTAINER_IMPORTER_CAPABILITY,
        PSD_IMPORTER_CAPABILITY,
    ]
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    [
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ]
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    [
        ExportTargetPlatform::Windows,
        ExportTargetPlatform::Linux,
        ExportTargetPlatform::Macos,
    ]
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(MODULE_NAME, "Texture and image importer plugin")
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        descriptor(
            "texture_importer.image",
            120,
            [
                "png", "jpg", "jpeg", "bmp", "tga", "tiff", "tif", "gif", "webp", "hdr", "exr",
                "qoi", "pnm", "pbm", "pgm", "ppm",
            ],
        )
        .with_required_capabilities([IMAGE_IMPORTER_CAPABILITY]),
        descriptor(
            "texture_importer.container",
            90,
            ["dds", "ktx", "ktx2", "astc"],
        )
        .with_required_capabilities([CONTAINER_IMPORTER_CAPABILITY]),
        descriptor("texture_importer.psd", 100, ["psd"])
            .with_required_capabilities([PSD_IMPORTER_CAPABILITY]),
        descriptor(
            "texture_importer.optional_native_container",
            80,
            ["cubemap", "dxgi"],
        )
        .with_required_capabilities(["runtime.asset.importer.native"]),
    ]
}

pub fn package_manifest() -> PluginPackageManifest {
    let mut manifest = PluginPackageManifest::new(PLUGIN_ID, "Texture Importer")
        .with_category("asset_importer")
        .with_supported_targets(supported_targets())
        .with_supported_platforms(supported_platforms())
        .with_capabilities(runtime_capabilities().iter().copied())
        .with_runtime_module(runtime_module_manifest());
    for importer in asset_importer_descriptors() {
        manifest = manifest.with_asset_importer(importer);
    }
    manifest
}

pub fn runtime_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::runtime("texture_importer.runtime", RUNTIME_CRATE_NAME)
        .with_target_modes(supported_targets())
        .with_capabilities(runtime_capabilities().iter().copied())
}

pub fn runtime_selection() -> ProjectPluginSelection {
    ProjectPluginSelection {
        id: PLUGIN_ID.to_string(),
        enabled: true,
        required: false,
        target_modes: supported_targets().to_vec(),
        packaging: ExportPackagingStrategy::LibraryEmbed,
        runtime_crate: Some(RUNTIME_CRATE_NAME.to_string()),
        editor_crate: None,
        features: Vec::new(),
    }
}

pub fn plugin_registration() -> RuntimePluginRegistrationReport {
    let mut extensions = RuntimeExtensionRegistry::default();
    let mut diagnostics = Vec::new();
    if let Err(error) = register_runtime_extensions(&mut extensions) {
        diagnostics.push(error.to_string());
    }
    RuntimePluginRegistrationReport {
        package_manifest: package_manifest(),
        project_selection: runtime_selection(),
        extensions,
        diagnostics,
    }
}

pub fn register_runtime_extensions(
    registry: &mut RuntimeExtensionRegistry,
) -> Result<(), RuntimeExtensionRegistryError> {
    registry.register_module(module_descriptor())?;
    for importer in asset_importer_descriptors() {
        match importer.id.as_str() {
            "texture_importer.image" => registry
                .register_asset_importer(FunctionAssetImporter::new(importer, import_image))?,
            "texture_importer.container" => registry.register_asset_importer(
                FunctionAssetImporter::new(importer, import_texture_container),
            )?,
            "texture_importer.psd" => registry
                .register_asset_importer(FunctionAssetImporter::new(importer, import_psd))?,
            "texture_importer.optional_native_container" => {
                registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
                    importer,
                    "cubemap/dxgi texture import requires a NativeDynamic texture backend",
                ))?;
            }
            _ => unreachable!("asset_importer_descriptors returns only known texture importer ids"),
        }
    }
    Ok(())
}

pub fn import_image(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let image = image::load_from_memory(&context.source_bytes).map_err(|error| {
        AssetImportError::Parse(format!(
            "decode image {}: {error}",
            context.source_path.display()
        ))
    })?;
    let rgba = image.to_rgba8();
    let (width, height) = image.dimensions();
    Ok(AssetImportOutcome::new(ImportedAsset::Texture(
        TextureAsset {
            uri: context.uri.clone(),
            width,
            height,
            rgba: rgba.into_raw(),
            payload: TexturePayload::Rgba8,
        },
    )))
}

pub fn import_psd(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let psd = psd::Psd::from_bytes(&context.source_bytes).map_err(|error| {
        AssetImportError::Parse(format!(
            "decode psd {}: {error}",
            context.source_path.display()
        ))
    })?;
    let width = psd.width();
    let height = psd.height();
    let rgba = psd.rgba();
    let expected_len = width as usize * height as usize * 4;
    if rgba.len() != expected_len {
        return Err(AssetImportError::Parse(format!(
            "decode psd {}: decoded rgba length {} did not match expected {}",
            context.source_path.display(),
            rgba.len(),
            expected_len
        )));
    }

    Ok(AssetImportOutcome::new(ImportedAsset::Texture(
        TextureAsset {
            uri: context.uri.clone(),
            width,
            height,
            rgba,
            payload: TexturePayload::Rgba8,
        },
    )))
}

pub fn import_texture_container(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let info = TextureContainerInfo::parse(context)?;
    Ok(AssetImportOutcome::new(ImportedAsset::Texture(
        TextureAsset {
            uri: context.uri.clone(),
            width: info.width,
            height: info.height,
            rgba: Vec::new(),
            payload: TexturePayload::Container {
                format: info.format,
                bytes: context.source_bytes.clone(),
                mip_count: info.mip_count,
                array_layers: info.array_layers,
            },
        },
    )))
}

struct TextureContainerInfo {
    format: String,
    width: u32,
    height: u32,
    mip_count: u32,
    array_layers: u32,
}

impl TextureContainerInfo {
    fn parse(context: &AssetImportContext) -> Result<Self, AssetImportError> {
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
        (format!("dds/dxgi-{dxgi_format}"), array_size * faces)
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
    let height = read_u32_le(context, 40)?.max(1);
    let array_elements = read_u32_le(context, 48)?.max(1);
    let faces = read_u32_le(context, 52)?.max(1);
    let mip_count = read_u32_le(context, 56)?.max(1);

    Ok(TextureContainerInfo {
        format: format!("ktx/gl-internal-0x{gl_internal_format:08x}"),
        width,
        height,
        mip_count,
        array_layers: array_elements * faces,
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
    let height = read_u32_le(context, 24)?.max(1);
    let layer_count = read_u32_le(context, 32)?.max(1);
    let face_count = read_u32_le(context, 36)?.max(1);
    let level_count = read_u32_le(context, 40)?.max(1);
    let supercompression = read_u32_le(context, 44)?;

    Ok(TextureContainerInfo {
        format: format!("ktx2/vk-{vk_format}/supercompression-{supercompression}"),
        width,
        height,
        mip_count: level_count,
        array_layers: layer_count * face_count,
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

    Ok(TextureContainerInfo {
        format: format!("astc/{block_x}x{block_y}x{block_z}"),
        width,
        height,
        mip_count: 1,
        array_layers: depth,
    })
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

fn descriptor(
    id: impl Into<String>,
    priority: i32,
    extensions: impl IntoIterator<Item = impl Into<String>>,
) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(id, PLUGIN_ID, AssetKind::Texture, 1)
        .with_priority(priority)
        .with_source_extensions(extensions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_declares_texture_importers() {
        let manifest = package_manifest();

        assert_eq!(manifest.id, PLUGIN_ID);
        assert!(manifest
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert!(manifest
            .asset_importers
            .iter()
            .any(|importer| importer.source_extensions.contains(&"ktx2".to_string())));
    }

    #[test]
    fn registration_contributes_module_and_importers() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == MODULE_NAME));
        assert_eq!(report.extensions.asset_importers().descriptors().len(), 4);
    }

    #[test]
    fn image_importer_decodes_texture_asset() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("checker.png"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "checker.png".into(),
            zircon_runtime::asset::AssetUri::parse("res://textures/checker.png").unwrap(),
            tiny_png_bytes(),
            Default::default(),
        );

        let imported = importer.import(&context).unwrap().imported_asset;

        match imported {
            zircon_runtime::asset::ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 2);
                assert_eq!(texture.height, 2);
                assert_eq!(texture.rgba.len(), 16);
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn psd_importer_decodes_flattened_rgba_texture_asset() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("swatch.psd"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "swatch.psd".into(),
            zircon_runtime::asset::AssetUri::parse("res://textures/swatch.psd").unwrap(),
            tiny_psd_bytes(),
            Default::default(),
        );

        let imported = importer.import(&context).unwrap().imported_asset;

        match imported {
            zircon_runtime::asset::ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 1);
                assert_eq!(texture.height, 1);
                assert_eq!(texture.rgba, vec![12, 34, 56, 200]);
                assert_eq!(texture.payload, TexturePayload::Rgba8);
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn dds_container_importer_preserves_compressed_payload() {
        let imported = import_container_fixture("albedo.dds", tiny_dds_bytes());

        match imported {
            ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 8);
                assert_eq!(texture.height, 4);
                assert!(texture.rgba.is_empty());
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
    fn ktx2_container_importer_reads_layers_faces_and_mips() {
        let imported = import_container_fixture("array.ktx2", tiny_ktx2_bytes());

        match imported {
            ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 16);
                assert_eq!(texture.height, 8);
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
    fn astc_container_importer_reads_block_and_size() {
        let imported = import_container_fixture("tile.astc", tiny_astc_bytes());

        match imported {
            ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 32);
                assert_eq!(texture.height, 16);
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

    fn import_container_fixture(path: &str, bytes: Vec<u8>) -> ImportedAsset {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new(path))
            .unwrap();
        let uri = format!("res://textures/{path}");
        let context = zircon_runtime::asset::AssetImportContext::new(
            path.into(),
            zircon_runtime::asset::AssetUri::parse(&uri).unwrap(),
            bytes,
            Default::default(),
        );
        importer.import(&context).unwrap().imported_asset
    }

    fn tiny_png_bytes() -> Vec<u8> {
        use image::{ImageBuffer, ImageFormat, Rgba};

        let image = ImageBuffer::<Rgba<u8>, _>::from_fn(2, 2, |x, y| {
            if (x + y) % 2 == 0 {
                Rgba([255, 255, 255, 255])
            } else {
                Rgba([0, 0, 0, 255])
            }
        });
        let mut bytes = std::io::Cursor::new(Vec::new());
        image.write_to(&mut bytes, ImageFormat::Png).unwrap();
        bytes.into_inner()
    }

    fn tiny_psd_bytes() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"8BPS");
        bytes.extend_from_slice(&1_u16.to_be_bytes());
        bytes.extend_from_slice(&[0; 6]);
        bytes.extend_from_slice(&4_u16.to_be_bytes());
        bytes.extend_from_slice(&1_u32.to_be_bytes());
        bytes.extend_from_slice(&1_u32.to_be_bytes());
        bytes.extend_from_slice(&8_u16.to_be_bytes());
        bytes.extend_from_slice(&3_u16.to_be_bytes());
        bytes.extend_from_slice(&0_u32.to_be_bytes());
        bytes.extend_from_slice(&0_u32.to_be_bytes());
        bytes.extend_from_slice(&0_u32.to_be_bytes());
        bytes.extend_from_slice(&0_u16.to_be_bytes());
        bytes.extend_from_slice(&[12, 34, 56, 200]);
        bytes
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

    fn tiny_ktx2_bytes() -> Vec<u8> {
        let mut bytes = vec![0; 68];
        bytes[0..12].copy_from_slice(KTX2_IDENTIFIER);
        write_u32(&mut bytes, 12, 37);
        write_u32(&mut bytes, 16, 1);
        write_u32(&mut bytes, 20, 16);
        write_u32(&mut bytes, 24, 8);
        write_u32(&mut bytes, 32, 2);
        write_u32(&mut bytes, 36, 6);
        write_u32(&mut bytes, 40, 4);
        write_u32(&mut bytes, 44, 1);
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

    fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u24(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset] = (value & 0xff) as u8;
        bytes[offset + 1] = ((value >> 8) & 0xff) as u8;
        bytes[offset + 2] = ((value >> 16) & 0xff) as u8;
    }
}
