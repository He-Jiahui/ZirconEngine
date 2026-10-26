use super::*;

use crate::asset::ZMaterialDocument;
use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RelPath};
use zircon_runtime_interface::resource::ResourceScheme;

const CANONICAL_SLOTS: [&str; 5] = [
    "base_color",
    "normal",
    "metallic_roughness",
    "occlusion",
    "emissive",
];

fn material(
    has_parent: bool,
    overrides: BTreeMap<String, toml::Value>,
    textures: BTreeMap<String, MaterialTextureSlotValue>,
) -> MaterialAsset {
    MaterialAsset::from_zmaterial_document(ZMaterialDocument {
        version: 2,
        name: None,
        shader: AssetReference::from_locator(AssetUri::parse("builtin://shader/pbr").unwrap()),
        parent: has_parent.then(|| asset_reference("parent", "res://materials/parent.zmaterial")),
        options: BTreeMap::new(),
        overrides,
        textures,
        queue: None,
        editor: toml::Table::new(),
        validation_diagnostics: Vec::new(),
    })
}

fn defaults() -> BTreeMap<String, toml::Value> {
    toml::from_str(
        r#"
base_color = [1.0, 1.0, 1.0, 1.0]
metallic = 0.0
roughness = 1.0
emissive = [0.0, 0.0, 0.0]
double_sided = false
cast_shadows = true
receive_shadows = true
render_queue = 0
material_queue = 0
depth_bias = 0.0
taa_reactive_mask_strength = 0.0
occlusion_strength = 1.0
normal_scale = 1.0
[alpha_mode]
mode = "opaque"
"#,
    )
    .unwrap()
}

fn parent_material() -> MaterialAsset {
    let overrides = toml::from_str(
        r#"
base_color = [0.2, 0.3, 0.4, 1.0]
metallic = 0.8
roughness = 0.2
emissive = [0.1, 0.2, 0.3]
double_sided = true
cast_shadows = false
receive_shadows = false
render_queue = 12
material_queue = 3
depth_bias = 0.2
taa_reactive_mask_strength = 0.4
occlusion_strength = 0.3
normal_scale = 0.5
[alpha_mode]
mode = "mask"
cutoff = 0.5
"#,
    )
    .unwrap();
    let textures = CANONICAL_SLOTS
        .map(|slot| {
            let locator = AssetUri::parse(&format!("builtin://textures/{slot}")).unwrap();
            (
                slot.to_owned(),
                MaterialTextureSlotValue::new(AssetReference::from_locator(locator)),
            )
        })
        .into_iter()
        .collect();
    material(false, overrides, textures)
}

// Exercise the production project codec, including project parent and builtin references.
fn round_trip(material: &MaterialAsset) -> MaterialAsset {
    let encoded = material
        .to_project_toml_string(|reference| {
            if reference.locator.scheme() == ResourceScheme::Builtin {
                return Ok(PersistedAssetReference::builtin(reference.locator.clone()));
            }
            Ok(PersistedAssetReference::project(
                AssetRef::try_new(
                    reference.uuid,
                    RelPath::parse("materials/parent.zmaterial").unwrap(),
                    None,
                )
                .unwrap(),
            ))
        })
        .unwrap();
    let document = ZMaterialDocument::from_project_toml_str(&encoded, |reference| {
        if let Some(locator) = reference.builtin_locator() {
            return Ok(AssetReference::from_locator(locator.clone()));
        }
        let reference = reference.project_ref().unwrap();
        Ok(AssetReference::new(
            reference.guid(),
            AssetUri::parse(&format!("res://{}", reference.path_hint())).unwrap(),
        ))
    })
    .unwrap();
    MaterialAsset::from_zmaterial_document(document)
}

fn empty_slots() -> BTreeMap<String, MaterialTextureSlotValue> {
    CANONICAL_SLOTS
        .map(|slot| {
            (
                slot.to_owned(),
                MaterialTextureSlotValue {
                    reference: None,
                    fallback: None,
                    transform: None,
                    uv_channel: 0,
                },
            )
        })
        .into_iter()
        .collect()
}

#[test]
fn explicit_defaults_still_override_parent_after_project_round_trip() {
    let mut child = round_trip(&material(true, defaults(), BTreeMap::new()));
    assert_eq!(child.property_values, defaults());
    assert!(child.parent.is_some());

    child.inherit_parent_values_from(&parent_material());

    for (key, value) in defaults() {
        assert_eq!(child.property_values.get(&key), Some(&value), "{key}");
    }
    assert_eq!(child.base_color, [1.0; 4]);
    assert_eq!(child.metallic, 0.0);
    assert_eq!(child.roughness, 1.0);
    assert_eq!(child.emissive, [0.0; 3]);
    assert_eq!(child.alpha_mode, AlphaMode::Opaque);
    assert!(!child.double_sided);
}

#[test]
fn absent_overrides_and_slots_remain_inherited_after_project_round_trip() {
    let mut child = round_trip(&material(true, BTreeMap::new(), BTreeMap::new()));
    assert!(child.property_values.is_empty());
    assert!(child.texture_slots.is_empty());
    let parent = parent_material();

    child.inherit_parent_values_from(&parent);

    assert_eq!(child.property_values, parent.property_values);
    assert_eq!(child.texture_slots, parent.texture_slots);
    assert_eq!(child.metallic, parent.metallic);
    assert_eq!(child.normal_texture, parent.normal_texture);
}

#[test]
fn editing_existing_pbr_overrides_to_defaults_rewrites_the_local_values() {
    let mut child = parent_material();
    child.parent = material(true, BTreeMap::new(), BTreeMap::new()).parent;
    child.base_color = [1.0; 4];
    child.metallic = 0.0;
    child.roughness = 1.0;
    child.emissive = [0.0; 3];
    child.alpha_mode = AlphaMode::Opaque;
    child.double_sided = false;

    let child = round_trip(&child);

    for key in [
        "base_color",
        "metallic",
        "roughness",
        "emissive",
        "alpha_mode",
        "double_sided",
    ] {
        assert_eq!(child.property_values.get(key), defaults().get(key), "{key}");
    }
}

#[test]
fn empty_canonical_slots_suppress_parent_textures_after_project_round_trip() {
    let mut child = round_trip(&material(true, BTreeMap::new(), empty_slots()));
    assert_eq!(child.texture_slots, empty_slots());

    child.inherit_parent_values_from(&parent_material());

    assert!(child.base_color_texture.is_none());
    assert!(child.normal_texture.is_none());
    assert!(child.metallic_roughness_texture.is_none());
    assert!(child.occlusion_texture.is_none());
    assert!(child.emissive_texture.is_none());
}

#[test]
fn clearing_an_existing_child_texture_preserves_local_presence_and_metadata() {
    let mut child = parent_material();
    child.parent = material(true, BTreeMap::new(), BTreeMap::new()).parent;
    child.normal_texture = None;
    let slot = child.texture_slots.get_mut("normal").unwrap();
    slot.fallback = Some("normal".to_owned());
    slot.transform = Some(RenderMaterialTextureTransform::default());
    slot.uv_channel = 2;
    let mut expected_slot = slot.clone();
    expected_slot.reference = None;

    let mut child = round_trip(&child);
    assert_eq!(child.texture_slots["normal"], expected_slot);
    child.inherit_parent_values_from(&parent_material());
    assert!(child.normal_texture.is_none());
    assert_eq!(child.texture_slots["normal"], expected_slot);
}

#[test]
fn removing_local_presence_restores_inheritance_after_project_round_trip() {
    let mut child = material(true, defaults(), empty_slots());
    child.property_values.clear();
    child.texture_slots.clear();
    let mut child = round_trip(&child);
    let parent = parent_material();

    child.inherit_parent_values_from(&parent);

    assert_eq!(child.property_values, parent.property_values);
    assert_eq!(child.texture_slots, parent.texture_slots);
}

#[test]
fn standalone_defaults_and_empty_slots_keep_existing_compact_serialization() {
    let material = round_trip(&material(false, defaults(), empty_slots()));

    assert!(material.parent.is_none());
    assert!(material.property_values.is_empty());
    assert!(material.texture_slots.is_empty());
}

#[test]
fn small_explicit_scalar_does_not_revert_to_the_parent_value() {
    let mut values = BTreeMap::new();
    values.insert("metallic".to_owned(), toml::Value::Float(1.0e-8));
    let mut child = round_trip(&material(true, values, BTreeMap::new()));
    assert!(child.property_values.contains_key("metallic"));

    child.inherit_parent_values_from(&parent_material());

    assert_eq!(child.metallic, 1.0e-8_f32);
}
