use std::path::PathBuf;

use super::*;

#[test]
fn settings_default_and_serde_use_managed_build_storage() {
    let defaults = HubSettings::default();
    let deserialized: HubSettings = toml::from_str("").unwrap();

    assert_eq!(
        deserialized.default_build_output_dir,
        defaults.default_build_output_dir
    );
    assert_eq!(
        deserialized.default_build_output_provenance,
        OutputRootProvenance::Unverified
    );
}

#[cfg(windows)]
#[test]
fn windows_settings_default_uses_managed_build_storage() {
    assert_eq!(
        HubSettings::default().default_build_output_dir,
        PathBuf::from("E:/cargo-targets/zircon-local/hub-build")
    );
}

#[test]
fn legacy_home_build_default_is_migrated_with_windows_path_normalization() {
    let home = PathBuf::from(r"C:\Users\HeJiahui");
    let mut settings = HubSettings::default();
    settings.default_build_output_dir = PathBuf::from(r"c:/users/heJIAhui/zirconbuilds/");
    settings.default_build_output_provenance = OutputRootProvenance::Unverified;

    migrate_legacy_build_output_default(&mut settings, Some(home));

    assert_eq!(
        settings.default_build_output_dir,
        default_build_output_dir()
    );
    assert_eq!(
        settings.default_build_output_provenance,
        OutputRootProvenance::HubManaged
    );
}

#[test]
fn legacy_build_default_migration_preserves_custom_and_folder_picker_paths() {
    let home = PathBuf::from(r"C:\Users\HeJiahui");
    let custom_root = PathBuf::from(r"E:\cargo-targets\zircon-local\custom-build");
    let mut custom = HubSettings::default();
    custom.default_build_output_dir = custom_root.clone();
    custom.default_build_output_provenance = OutputRootProvenance::Unverified;

    migrate_legacy_build_output_default(&mut custom, Some(home.clone()));
    assert_eq!(custom.default_build_output_dir, custom_root);

    let picked_path = PathBuf::from(r"C:\Users\HeJiahui\ZirconBuilds");
    let mut picked = HubSettings::default();
    picked.default_build_output_dir = picked_path.clone();
    picked.default_build_output_provenance = OutputRootProvenance::NativeFolderPicker;

    migrate_legacy_build_output_default(&mut picked, Some(home));
    assert_eq!(picked.default_build_output_dir, picked_path);
    assert_eq!(
        picked.default_build_output_provenance,
        OutputRootProvenance::NativeFolderPicker
    );
}
