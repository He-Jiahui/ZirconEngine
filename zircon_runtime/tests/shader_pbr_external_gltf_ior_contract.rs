use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::{
    AssetImportOutcome, AssetImporter, AssetUri, ImportedAsset, MaterialAsset,
};

const EXTERNAL_GLTF_IOR2_URI: &str = "res://materials/shader06_dielectric_ior2.gltf";
const EXTERNAL_GLTF_IOR_ZERO_URI: &str = "res://materials/shader06_dielectric_ior_zero.gltf";

#[test]
fn external_gltf_required_ior_two_projects_canonical_pbr_reflection_inputs() {
    let fixture = ExternalGltfIorFixture::new("external_gltf_ior2_import", 2.0);

    let root_uri = AssetUri::parse(EXTERNAL_GLTF_IOR2_URI).expect("external glTF IOR URI");
    let outcome = AssetImporter::default()
        .import_with_settings(&fixture.source_path, &root_uri, Default::default())
        .expect("import required KHR_materials_ior external glTF fixture");
    let material = external_gltf_material(&outcome, &root_uri);

    assert_eq!(
        material.shader,
        zircon_runtime::asset::assets::default_pbr_shader_reference(),
        "external glTF material must retain the canonical default PBR shader"
    );
    assert_eq!(material.base_color, [0.86, 0.9, 1.0, 1.0]);
    assert_eq!(material.metallic, 0.0);
    assert_eq!(material.roughness, 0.08);
    assert_eq!(
        material.property_values.get("ior"),
        Some(&toml::Value::Float(2.0)),
        "the external glTF IOR must be retained before PBR feature derivation"
    );
    assert!(
        material
            .validation_diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.contains("KHR_materials_ior")),
        "valid required KHR_materials_ior must not be diagnosed as unsupported: {:#?}",
        material.validation_diagnostics
    );

    let features = material.advanced_pbr_features();
    assert_eq!(features.ior, 2.0);
    assert!(features.uses_dielectric_f0_override());
    assert!(
        features.requires_forward_path(),
        "a non-default dielectric F0 must leave the fixed deferred GBuffer for Forward PBR"
    );
    assert!(
        (features.dielectric_f0() - (1.0 / 9.0)).abs() <= f32::EPSILON,
        "IOR 2 must derive the Fresnel F0 used by direct and environment PBR reflection"
    );
}

#[test]
fn external_gltf_zero_ior_is_diagnosed_instead_of_silently_using_default_fresnel() {
    let fixture = ExternalGltfIorFixture::new("external_gltf_ior_zero_import", 0.0);

    let root_uri = AssetUri::parse(EXTERNAL_GLTF_IOR_ZERO_URI).expect("external glTF zero-IOR URI");
    let outcome = AssetImporter::default()
        .import_with_settings(&fixture.source_path, &root_uri, Default::default())
        .expect("import external glTF fixture with zero IOR diagnostic");
    let material = external_gltf_material(&outcome, &root_uri);

    assert!(
        material.validation_diagnostics.iter().any(|diagnostic| diagnostic
            .contains("KHR_materials_ior.ior = 0 requests the unsupported specular-glossiness compatibility mode")),
        "zero IOR has special Khronos semantics and must remain an explicit unsupported-MVP diagnostic: {:#?}",
        material.validation_diagnostics
    );
    assert!(
        !material.property_values.contains_key("ior"),
        "the rejected zero-IOR value must not reach material normalization"
    );
    let features = material.advanced_pbr_features();
    assert!(!features.uses_dielectric_f0_override());
    assert!(
        !features.requires_forward_path(),
        "the rejected zero-IOR field must not silently create a Forward-only material path"
    );
}

#[test]
fn external_gltf_subunit_nonzero_ior_remains_an_invalid_numeric_value() {
    let fixture = ExternalGltfIorFixture::new("external_gltf_ior_subunit_import", 0.5);

    let root_uri = AssetUri::parse("res://materials/shader06_dielectric_ior_subunit.gltf")
        .expect("external glTF subunit-IOR URI");
    let outcome = AssetImporter::default()
        .import_with_settings(&fixture.source_path, &root_uri, Default::default())
        .expect("import external glTF fixture with invalid subunit IOR");
    let material = external_gltf_material(&outcome, &root_uri);

    assert!(
        material
            .validation_diagnostics
            .iter()
            .any(|diagnostic| diagnostic
                .contains("KHR_materials_ior.ior contains an invalid numeric value")),
        "only the zero IOR compatibility mode receives the dedicated MVP diagnostic: {:#?}",
        material.validation_diagnostics
    );
    assert!(!material.property_values.contains_key("ior"));
}

struct ExternalGltfIorFixture {
    root: PathBuf,
    source_path: PathBuf,
}

impl ExternalGltfIorFixture {
    fn new(label: &str, ior: f32) -> Self {
        let root = unique_shader_evidence_root(label);
        fs::create_dir_all(&root).expect("create Shader06 external glTF IOR fixture directory");
        let source_path = root.join("material.gltf");
        fs::write(&source_path, external_gltf_ior_document(ior))
            .expect("write Shader06 external glTF IOR fixture");
        Self { root, source_path }
    }
}

impl Drop for ExternalGltfIorFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn external_gltf_ior_document(ior: f32) -> String {
    format!(
        r#"{{
  "asset": {{ "version": "2.0" }},
  "extensionsUsed": ["KHR_materials_ior"],
  "extensionsRequired": ["KHR_materials_ior"],
  "materials": [{{
    "name": "Shader06 dielectric IOR fixture",
    "pbrMetallicRoughness": {{
      "baseColorFactor": [0.86, 0.90, 1.0, 1.0],
      "metallicFactor": 0.0,
      "roughnessFactor": 0.08
    }},
    "extensions": {{ "KHR_materials_ior": {{ "ior": {ior} }} }}
  }}]
}}"#
    )
}

fn external_gltf_material<'a>(
    outcome: &'a AssetImportOutcome,
    root_uri: &AssetUri,
) -> &'a MaterialAsset {
    let material_uri =
        AssetUri::parse(&format!("{root_uri}#Material0")).expect("external glTF IOR material URI");
    match &outcome
        .entries
        .iter()
        .find(|entry| entry.locator == material_uri)
        .expect("external glTF IOR material subasset")
        .asset
    {
        ImportedAsset::Material(material) => material,
        other => panic!("external glTF IOR subasset must be a material, got {other:?}"),
    }
}

fn unique_shader_evidence_root(label: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after Unix epoch")
        .as_nanos();
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../docs/tests/runtime/shader/.work")
        .join(format!("{label}_{}_{}", std::process::id(), timestamp))
}
