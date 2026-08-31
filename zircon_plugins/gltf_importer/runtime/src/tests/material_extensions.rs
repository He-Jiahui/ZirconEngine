use super::*;

#[test]
fn importer_required_extension_preflight_accepts_owned_material_semantics_only() {
    let supported = br#"{
        "asset": { "version": "2.0" },
        "extensionsRequired": ["KHR_materials_unlit"],
        "materials": [{ "extensions": { "KHR_materials_unlit": {} } }]
    }"#;
    crate::parse_gltf_preflight_document(supported)
        .expect("stable importer should accept required material semantics it owns");

    let supported_advanced = br#"{
        "asset": { "version": "2.0" },
        "extensionsUsed": [
            "KHR_materials_anisotropy",
            "KHR_materials_ior",
            "KHR_materials_transmission",
            "KHR_materials_volume"
        ],
        "extensionsRequired": [
            "KHR_materials_anisotropy",
            "KHR_materials_ior",
            "KHR_materials_transmission",
            "KHR_materials_volume"
        ],
        "materials": [{ "extensions": {
            "KHR_materials_anisotropy": {
                "anisotropyStrength": 0.6,
                "anisotropyRotation": 1.25
            },
            "KHR_materials_ior": { "ior": 1.333 },
            "KHR_materials_transmission": { "transmissionFactor": 0.75 },
            "KHR_materials_volume": {
                "thicknessFactor": 0.25,
                "attenuationDistance": 4.0,
                "attenuationColor": [0.2, 0.6, 0.9]
            }
        } }]
    }"#;
    crate::parse_gltf_preflight_document(supported_advanced)
        .expect("factor-only required advanced material semantics are supported");

    let unsupported = br#"{
        "asset": { "version": "2.0" },
        "extensionsRequired": ["KHR_materials_specular"],
        "materials": [{ "extensions": { "KHR_materials_specular": { "specularFactor": 0.5 } } }]
    }"#;
    let error = crate::parse_gltf_preflight_document(unsupported).expect_err(
        "stable importer must not accept a required material extension without an owner",
    );
    assert!(error
        .to_string()
        .contains("requires unsupported extension `KHR_materials_specular`"));

    let supported_clearcoat = br#"{
        "asset": { "version": "2.0" },
        "extensionsUsed": ["KHR_materials_clearcoat"],
        "extensionsRequired": ["KHR_materials_clearcoat"],
        "materials": [{ "extensions": { "KHR_materials_clearcoat": {
            "clearcoatFactor": 0.5,
            "clearcoatRoughnessFactor": 0.2
        } } }]
    }"#;
    crate::parse_gltf_preflight_document(supported_clearcoat)
        .expect("factor-only required clearcoat is owned by Standard PBR");

    for (source, expected_field) in [
        (
            br#"{
                "asset": { "version": "2.0" },
                "extensionsUsed": ["KHR_materials_clearcoat"],
                "extensionsRequired": ["KHR_materials_clearcoat"],
                "images": [{ "uri": "unused.png" }],
                "textures": [{ "source": 0 }],
                "materials": [{ "extensions": {
                    "KHR_materials_clearcoat": {
                        "clearcoatFactor": 0.8,
                        "clearcoatTexture": { "index": 0 }
                    }
                } }]
            }"#
            .as_slice(),
            "KHR_materials_clearcoat.clearcoatTexture",
        ),
        (
            br#"{
                "asset": { "version": "2.0" },
                "extensionsUsed": ["KHR_materials_clearcoat"],
                "extensionsRequired": ["KHR_materials_clearcoat"],
                "images": [{ "uri": "unused.png" }],
                "textures": [{ "source": 0 }],
                "materials": [{ "extensions": {
                    "KHR_materials_clearcoat": {
                        "clearcoatRoughnessFactor": 0.2,
                        "clearcoatRoughnessTexture": { "index": 0 }
                    }
                } }]
            }"#
            .as_slice(),
            "KHR_materials_clearcoat.clearcoatRoughnessTexture",
        ),
        (
            br#"{
                "asset": { "version": "2.0" },
                "extensionsUsed": ["KHR_materials_anisotropy"],
                "extensionsRequired": ["KHR_materials_anisotropy"],
                "images": [{ "uri": "unused.png" }],
                "textures": [{ "source": 0 }],
                "materials": [{ "extensions": {
                    "KHR_materials_anisotropy": {
                        "anisotropyStrength": 0.6,
                        "anisotropyTexture": { "index": 0 }
                    }
                } }]
            }"#
            .as_slice(),
            "KHR_materials_anisotropy.anisotropyTexture",
        ),
        (
            br#"{
                "asset": { "version": "2.0" },
                "extensionsUsed": ["KHR_materials_transmission"],
                "extensionsRequired": ["KHR_materials_transmission"],
                "images": [{ "uri": "unused.png" }],
                "textures": [{ "source": 0 }],
                "materials": [{ "extensions": {
                    "KHR_materials_transmission": {
                        "transmissionFactor": 0.75,
                        "transmissionTexture": { "index": 0 }
                    }
                } }]
            }"#
            .as_slice(),
            "KHR_materials_transmission.transmissionTexture",
        ),
        (
            br#"{
                "asset": { "version": "2.0" },
                "extensionsUsed": ["KHR_materials_transmission", "KHR_materials_volume"],
                "extensionsRequired": ["KHR_materials_transmission", "KHR_materials_volume"],
                "images": [{ "uri": "unused.png" }],
                "textures": [{ "source": 0 }],
                "materials": [{ "extensions": {
                    "KHR_materials_transmission": { "transmissionFactor": 0.75 },
                    "KHR_materials_volume": {
                        "thicknessFactor": 0.25,
                        "thicknessTexture": { "index": 0 }
                    }
                } }]
            }"#
            .as_slice(),
            "KHR_materials_volume.thicknessTexture",
        ),
        (
            br#"{
                "asset": { "version": "2.0" },
                "extensionsUsed": ["KHR_materials_ior"],
                "extensionsRequired": ["KHR_materials_ior"],
                "materials": [{ "extensions": {
                    "KHR_materials_ior": { "ior": 0.0 }
                } }]
            }"#
            .as_slice(),
            "KHR_materials_ior.ior=0",
        ),
    ] {
        let error = crate::parse_gltf_preflight_document(source)
            .expect_err("required material semantics without shader owners must fail closed");
        assert!(
            error.to_string().contains(expected_field),
            "expected `{expected_field}` in {error}"
        );
    }
}

#[test]
fn importer_projects_required_gltf_material_extension_semantics() {
    let gltf = gltf::Gltf::from_slice(
        br#"
{
  "asset": { "version": "2.0" },
  "extensionsUsed": ["KHR_materials_unlit", "KHR_materials_anisotropy", "KHR_materials_clearcoat", "KHR_materials_diffuse_transmission", "KHR_materials_dispersion", "KHR_materials_emissive_strength", "KHR_materials_ior", "KHR_materials_iridescence", "KHR_materials_pbrSpecularGlossiness", "KHR_materials_sheen", "KHR_materials_specular", "KHR_materials_subsurface", "KHR_materials_transmission", "KHR_materials_volume", "KHR_texture_transform"],
  "images": [{ "uri": "clearcoat-normal.png" }],
  "textures": [{ "source": 0 }],
  "materials": [{
    "emissiveFactor": [0.5, 1.0, 1.5],
    "extensions": {
      "KHR_materials_unlit": {},
      "KHR_materials_anisotropy": {
        "anisotropyStrength": 0.6,
        "anisotropyRotation": 1.25,
        "anisotropyTexture": { "index": 0 }
      },
      "KHR_materials_diffuse_transmission": {},
      "KHR_materials_dispersion": {},
      "KHR_materials_ior": { "ior": 1.333 },
      "KHR_materials_iridescence": {},
      "KHR_materials_pbrSpecularGlossiness": {},
      "KHR_materials_sheen": {},
      "KHR_materials_subsurface": {},
      "KHR_materials_transmission": { "transmissionFactor": 0.75 },
      "KHR_materials_volume": { "thicknessFactor": 0.24, "attenuationDistance": 4.5, "attenuationColor": [0.14, 0.7, 0.94] },
      "KHR_materials_emissive_strength": { "emissiveStrength": 2.0 },
      "KHR_materials_specular": { "specularFactor": 0.5 },
      "KHR_materials_clearcoat": {
        "clearcoatFactor": 0.8,
        "clearcoatRoughnessFactor": 0.15,
        "clearcoatTexture": { "index": 0 },
        "clearcoatRoughnessTexture": { "index": 0 },
        "clearcoatNormalTexture": {
          "index": 0,
          "texCoord": 0,
          "scale": 0.35,
          "extensions": {
            "KHR_texture_transform": {
              "offset": [0.1, 0.2],
              "scale": [0.5, 0.75],
              "rotation": 0.4,
              "texCoord": 1
            }
          }
        }
      }
    }
  }, {
    "extensions": { "KHR_materials_anisotropy": {} }
  }]
}
"#,
    )
    .expect("in-memory material-extension glTF fixture should parse");
    let root_uri = AssetUri::parse("res://models/material_extensions.gltf").unwrap();
    let outcome = crate::subassets::add_gltf_material_subassets(
        AssetImportOutcome::new(
            root_uri.clone(),
            ImportedAsset::Model(ModelAsset {
                uri: root_uri.clone(),
                primitives: Vec::new(),
            }),
        ),
        &root_uri,
        &gltf.document,
    );

    match &entry_for_locator(&outcome, &label_uri_for(&root_uri, "Material0")).asset {
        ImportedAsset::Material(material) => {
            assert!(material.standard_material_descriptor().unlit);
            assert_eq!(material.emissive, [1.0, 2.0, 3.0]);
            let features = material.advanced_pbr_features();
            assert_f32_near(features.anisotropy_strength, 0.6);
            assert_f32_near(features.anisotropy_rotation, 1.25);
            assert_f32_near(features.ior, 1.333);
            assert_f32_near(features.specular_transmission, 0.75);
            assert_f32_near(features.thickness, 0.24);
            assert_f32_near(features.attenuation_distance, 4.5);
            assert_eq!(features.attenuation_color, [0.14, 0.7, 0.94]);
            assert_f32_near(features.clearcoat, 0.8);
            assert_f32_near(features.clearcoat_perceptual_roughness, 0.15);
            assert_f32_near(features.clearcoat_normal_scale, 0.35);
            assert_eq!(
                features
                    .clearcoat_normal_texture
                    .as_ref()
                    .map(|reference| &reference.locator),
                Some(&label_uri_for(&root_uri, "Texture0"))
            );
            let descriptor = material.standard_material_descriptor();
            assert_eq!(descriptor.clearcoat_normal_texture_uv_channel, 1);
            assert_eq!(
                descriptor.clearcoat_normal_texture_transform,
                RenderMaterialTextureTransform {
                    scale: [0.5, 0.75],
                    offset: [0.1, 0.2],
                    rotation: 0.4,
                }
            );
            assert!(material
                .validation_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("KHR_materials_specular")));
            assert!(material
                .validation_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("clearcoatTexture")));
            assert!(material
                .validation_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("clearcoatRoughnessTexture")));
            assert!(material
                .validation_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("anisotropyTexture")));
            for extension in [
                "KHR_materials_diffuse_transmission",
                "KHR_materials_dispersion",
                "KHR_materials_iridescence",
                "KHR_materials_pbrSpecularGlossiness",
                "KHR_materials_sheen",
                "KHR_materials_subsurface",
            ] {
                assert!(
                    material
                        .validation_diagnostics
                        .iter()
                        .any(|diagnostic| diagnostic.contains(extension)),
                    "missing optional fallback diagnostic for {extension}"
                );
            }
        }
        other => panic!("unexpected Material0 asset: {other:?}"),
    }

    match &entry_for_locator(&outcome, &label_uri_for(&root_uri, "Material1")).asset {
        ImportedAsset::Material(material) => {
            let features = material.advanced_pbr_features();
            assert_eq!(features.anisotropy_strength, 0.0);
            assert_eq!(features.anisotropy_rotation, 0.0);
        }
        other => panic!("unexpected Material1 asset: {other:?}"),
    }
}
