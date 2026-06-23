use super::*;

#[test]
fn importer_decodes_physics_material_and_animation_sequence_assets() {
    let root = unique_temp_project_root("physics_animation_import");
    fs::create_dir_all(&root).unwrap();
    let physics_material_path = root.join("default.physics_material.toml");
    let sequence_path = root.join("hero.sequence.zranim");

    write_default_physics_material(physics_material_path.clone());
    write_default_animation_sequence(sequence_path.clone());

    let importer = AssetImporter::default();
    let physics_material = importer
        .import_from_source(
            &physics_material_path,
            &AssetUri::parse("res://physics/materials/default.physics_material.toml").unwrap(),
        )
        .unwrap();
    let sequence = importer
        .import_from_source(
            &sequence_path,
            &AssetUri::parse("res://animation/hero.sequence.zranim").unwrap(),
        )
        .unwrap();

    assert_eq!(
        physics_material,
        ImportedAsset::PhysicsMaterial(sample_physics_material_asset())
    );
    assert_eq!(
        sequence,
        ImportedAsset::AnimationSequence(sample_animation_sequence_asset())
    );

    let _ = fs::remove_dir_all(root);
}
