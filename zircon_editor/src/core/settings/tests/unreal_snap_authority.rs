use super::*;
use crate::core::settings::{
    SettingsAuthority, VIEWPORT_ROTATE_SNAP_ENABLED_KEY, VIEWPORT_SCALE_SNAP_ENABLED_KEY,
    VIEWPORT_TRANSLATE_SNAP_ENABLED_KEY,
};

#[test]
fn unreal_snap_enable_authority_updates_only_the_matching_typed_boolean_and_preserves_old_snapshot()
{
    let authority = SettingsAuthority::with_defaults();
    let before = authority.snapshot();
    let snap = before.viewport_snap();
    assert!(snap.translate_enabled() && snap.rotate_enabled() && snap.scale_enabled());
    for (index, key_name) in [
        VIEWPORT_TRANSLATE_SNAP_ENABLED_KEY,
        VIEWPORT_ROTATE_SNAP_ENABLED_KEY,
        VIEWPORT_SCALE_SNAP_ENABLED_KEY,
    ]
    .into_iter()
    .enumerate()
    {
        authority
            .set(
                SettingsScope::Project,
                &key(key_name),
                SettingValue::Bool(false),
            )
            .unwrap();
        let current = authority.snapshot().viewport_snap();
        assert_eq!(current.translate_enabled(), false);
        assert_eq!(current.rotate_enabled(), index < 1);
        assert_eq!(current.scale_enabled(), index < 2);
        assert_eq!(current.translate_step(), snap.translate_step());
        assert_eq!(current.rotate_step_degrees(), snap.rotate_step_degrees());
        assert_eq!(current.scale_step(), snap.scale_step());
    }
    assert!(before.viewport_snap().translate_enabled());
    let generation = authority.snapshot().generation();
    assert!(authority
        .set(
            SettingsScope::Project,
            &key(VIEWPORT_TRANSLATE_SNAP_ENABLED_KEY),
            SettingValue::Float(0.0)
        )
        .is_err());
    assert_eq!(authority.snapshot().generation(), generation);
}

#[test]
fn unreal_snap_enable_project_preferences_round_trip_without_writing_scene_sources() {
    let root = temporary_root("unreal-snap-enable");
    let project = root.join("project");
    let scene = project.join("assets/scenes/main.scene.toml");
    fs::create_dir_all(scene.parent().unwrap()).unwrap();
    fs::write(&scene, b"entities = []\n").unwrap();
    let digest = blake3::hash(&fs::read(&scene).unwrap());
    let store = SettingsStore::from_roots(&root, Some(&project));
    let mut registry = settings_registry_with_defaults();
    registry
        .set(
            SettingsScope::Project,
            &key(VIEWPORT_TRANSLATE_SNAP_ENABLED_KEY),
            SettingValue::Bool(false),
        )
        .unwrap();
    registry
        .set(
            SettingsScope::Project,
            &key(VIEWPORT_SCALE_SNAP_ENABLED_KEY),
            SettingValue::Bool(false),
        )
        .unwrap();
    store.save_from(SettingsScope::Project, &registry).unwrap();
    let mut reopened = settings_registry_with_defaults();
    store
        .load_into(SettingsScope::Project, &mut reopened)
        .unwrap();
    assert_eq!(
        reopened
            .resolve(&key(VIEWPORT_TRANSLATE_SNAP_ENABLED_KEY))
            .unwrap(),
        &SettingValue::Bool(false)
    );
    assert_eq!(
        reopened
            .resolve(&key(VIEWPORT_ROTATE_SNAP_ENABLED_KEY))
            .unwrap(),
        &SettingValue::Bool(true)
    );
    assert_eq!(
        reopened
            .resolve(&key(VIEWPORT_SCALE_SNAP_ENABLED_KEY))
            .unwrap(),
        &SettingValue::Bool(false)
    );
    assert_eq!(blake3::hash(&fs::read(&scene).unwrap()), digest);
    remove_temporary_root(&root);
}
