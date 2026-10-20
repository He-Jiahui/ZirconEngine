use crate::asset::{AssetUri, TextureAsset, TextureAssetDescriptor, TexturePayload};
use serde::Serialize;

// Field and variant order match the owned DTO exactly. No pixel/container bytes are copied.
#[derive(Serialize)]
pub(super) struct ArtifactCacheTextureAssetRef<'a> {
    uri: &'a AssetUri,
    width: u32,
    height: u32,
    rgba: &'a [u8],
    payload: ArtifactCacheTexturePayloadRef<'a>,
    descriptor: &'a Option<TextureAssetDescriptor>,
}
#[derive(Serialize)]
enum ArtifactCacheTexturePayloadRef<'a> {
    Rgba8,
    Container {
        format: &'a str,
        bytes: &'a [u8],
        mip_count: u32,
        array_layers: u32,
    },
}
impl<'a> ArtifactCacheTextureAssetRef<'a> {
    pub(super) fn new(asset: &'a TextureAsset) -> Self {
        Self {
            uri: &asset.uri,
            width: asset.width,
            height: asset.height,
            rgba: &asset.rgba,
            payload: match &asset.payload {
                TexturePayload::Rgba8 => ArtifactCacheTexturePayloadRef::Rgba8,
                TexturePayload::Container {
                    format,
                    bytes,
                    mip_count,
                    array_layers,
                } => ArtifactCacheTexturePayloadRef::Container {
                    format,
                    bytes,
                    mip_count: *mip_count,
                    array_layers: *array_layers,
                },
            },
            descriptor: &asset.descriptor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{from_imported_for_write, ArtifactCacheAsset};
    use crate::asset::{
        AssetUri, ImportedAsset, MeshAsset, MeshAttributeValues, MeshIndices, SoundAsset,
        TextureAsset, MESH_ATTRIBUTE_POSITION,
    };
    use zircon_runtime_interface::serialization::PositionalEncoding;
    #[test]
    fn borrowed_texture_sound_mesh_dtos_are_byte_identical_and_roundtrip() {
        let uri = |path| AssetUri::parse(path).unwrap();
        let mut attributes = std::collections::BTreeMap::new();
        attributes.insert(
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0; 3], [1.0; 3], [2.0; 3]]),
        );
        let mesh = MeshAsset::new(
            uri("res://memory/mesh.zrmesh"),
            Default::default(),
            attributes,
            Some(MeshIndices::U16(vec![0, 1, 2])),
        )
        .unwrap();
        let sound = SoundAsset {
            uri: uri("res://memory/sound.wav"),
            sample_rate_hz: 48000,
            channel_count: 1,
            channel_layout: crate::core::framework::audio::AudioChannelLayout::mono(),
            samples: vec![0.0, 0.5, -0.5],
        };
        let assets = [
            ImportedAsset::Texture(TextureAsset::new_rgba8(
                uri("res://memory/image.png"),
                2,
                2,
                vec![3; 16],
            )),
            ImportedAsset::Texture(TextureAsset::new_container(
                uri("res://memory/image.dds"),
                4,
                4,
                "bc1",
                vec![3; 8],
                1,
                1,
            )),
            ImportedAsset::Sound(sound),
            ImportedAsset::Mesh(mesh),
        ];
        for asset in assets {
            let encoding = PositionalEncoding::FixedLittleEndian;
            let expected = encoding
                .encode(&ArtifactCacheAsset::from_imported(&asset).unwrap())
                .unwrap();
            let actual = encoding
                .encode(&from_imported_for_write(&asset).unwrap())
                .unwrap();
            assert_eq!(actual, expected);
            let decoded: ArtifactCacheAsset = encoding.decode(&actual).unwrap();
            assert_eq!(decoded.into_imported().unwrap(), asset);
        }
    }
}
