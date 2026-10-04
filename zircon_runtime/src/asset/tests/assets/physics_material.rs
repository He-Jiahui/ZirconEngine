//! 验证物理材质文档往返保留摩擦与恢复系数的组合规则，供场景碰撞体引用时使用。

use crate::core::framework::scene::physics::PhysicsCombineRule;

use crate::asset::tests::support::sample_physics_material_asset;
use crate::asset::PhysicsMaterialAsset;

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
