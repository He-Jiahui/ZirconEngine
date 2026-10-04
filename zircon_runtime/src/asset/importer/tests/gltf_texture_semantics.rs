use super::{gltf_texture_color_space_usages, gltf_texture_label, GltfTextureColorSpace};
use crate::core::framework::render::TextureUsageHint;

#[test]
fn clearcoat_normal_texture_registers_the_normal_variant_owner() {
    let gltf = gltf::Gltf::from_slice(
        br#"{
                "asset": { "version": "2.0" },
                "extensionsUsed": ["KHR_materials_clearcoat"],
                "images": [{ "uri": "coat.png" }],
                "textures": [{ "source": 0 }],
                "materials": [{
                    "extensions": {
                        "KHR_materials_clearcoat": {
                            "clearcoatNormalTexture": { "index": 0 }
                        }
                    }
                }]
            }"#,
    )
    .expect("clearcoat normal usage fixture");

    let usages = gltf_texture_color_space_usages(&gltf.document);
    let variants = usages[0].texture_variants();

    assert_eq!(variants.len(), 1);
    assert_eq!(variants[0].color_space(), GltfTextureColorSpace::Linear);
    assert_eq!(variants[0].usage_hint(), TextureUsageHint::Normal);
    assert_eq!(gltf_texture_label(0, variants[0], &usages), "Texture0");
}
