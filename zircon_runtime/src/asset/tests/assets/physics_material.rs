use crate::core::framework::scene::physics::PhysicsCombineRule;

use crate::asset::PhysicsMaterialAsset;
use crate::asset::tests::support::sample_physics_material_asset;

#[test]
fn physics_material_asset_toml_roundtrip_preserves_combine_rules() {
    let material = sample_physics_material_asset();

    let document = material.to_toml_string().unwrap();
    let loaded = PhysicsMaterialAsset::from_toml_str(&document).unwrap();

    assert_eq!(loaded, material);
    assert!(document.contains("friction_combine"));
    assert!(document.contains("restitution_combine"));
    assert_eq!(
        loaded.metadata.friction_combine,
        PhysicsCombineRule::Maximum
    );
}
