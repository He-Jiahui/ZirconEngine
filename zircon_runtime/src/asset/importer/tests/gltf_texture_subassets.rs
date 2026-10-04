use crate::core::framework::render::{TextureMipPolicy, TextureNormalConvention};

use super::gltf_texture_descriptor;
use crate::asset::importer::{gltf_texture_variant, GltfTextureColorSpace};
use crate::core::framework::render::TextureUsageHint;

#[test]
fn explicit_non_mip_sampler_does_not_publish_generated_mips() {
    let gltf = gltf::Gltf::from_slice(
        br#"{
                "asset": { "version": "2.0" },
                "images": [{ "uri": "texture.png" }],
                "samplers": [
                    { "minFilter": 9729 },
                    { "minFilter": 9987 }
                ],
                "textures": [
                    { "sampler": 0, "source": 0 },
                    { "sampler": 1, "source": 0 }
                ]
            }"#,
    )
    .expect("glTF sampler fixture");
    let variant = gltf_texture_variant(GltfTextureColorSpace::Srgb, TextureUsageHint::Albedo);
    let textures = gltf.document.textures().collect::<Vec<_>>();

    assert_eq!(
        gltf_texture_descriptor(&textures[0], variant)
            .metadata
            .mip_policy,
        TextureMipPolicy::None
    );
    assert_eq!(
        gltf_texture_descriptor(&textures[1], variant)
            .metadata
            .mip_policy,
        TextureMipPolicy::GenerateOffline
    );
}

#[test]
fn gltf_normal_texture_descriptor_declares_canonical_gl_source() {
    let gltf = gltf::Gltf::from_slice(
        br#"{
                "asset": { "version": "2.0" },
                "images": [{ "uri": "normal.png" }],
                "textures": [{ "source": 0 }]
            }"#,
    )
    .expect("glTF normal texture fixture");
    let texture = gltf.document.textures().next().expect("texture");
    let variant = gltf_texture_variant(GltfTextureColorSpace::Linear, TextureUsageHint::Normal);

    assert_eq!(
        gltf_texture_descriptor(&texture, variant)
            .metadata
            .normal_convention,
        TextureNormalConvention::TangentSpaceGl
    );
}
