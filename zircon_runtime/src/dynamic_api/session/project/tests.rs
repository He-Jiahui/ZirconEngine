use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::project::{ProjectManifest, ProjectPaths, ProjectScriptManifest};
use crate::asset::{AssetUri, ProjectInfo};
use crate::core::framework::project::ProjectPluginManifest;
use crate::script::{
    CapabilitySet, DiscoveredVmPluginPackage, VmPluginManagementPolicy, VmPluginManifest,
    VmPluginPackage, VmPluginPackageSource,
};
use zircon_runtime_interface::ZrByteSlice;

use super::{
    project_opened_log, RuntimeLoadedProjectManifest, RuntimeProjectConfig, RuntimeProjectError,
};

#[test]
fn project_opened_log_reports_the_activated_project_snapshot() {
    let log = project_opened_log(&ProjectInfo {
        root_path: "C:\\projects\\renderable-empty".to_string(),
        name: "Renderable Empty".to_string(),
        default_scene_uri: "res://scenes/main.scene.toml".to_string(),
        library_version: 1,
        asset_count: 8,
        ready_asset_count: 7,
        failed_asset_count: 1,
        registry_diagnostic_count: 2,
    });

    assert_eq!(
        log,
        "runtime_project_opened root=C:\\projects\\renderable-empty name=Renderable Empty default_scene=res://scenes/main.scene.toml library_version=1 assets=8 ready_assets=7 failed_assets=1 registry_diagnostics=2"
    );
}

#[test]
fn project_config_omits_empty_abi_slice() {
    let parsed = RuntimeProjectConfig::from_abi_slice(ZrByteSlice::empty()).unwrap();

    assert_eq!(parsed, None);
}

#[test]
fn project_config_rejects_whitespace_only_path() {
    let raw = b"   ";
    let error = RuntimeProjectConfig::from_abi_slice(ZrByteSlice {
        data: raw.as_ptr(),
        len: raw.len(),
    })
    .unwrap_err();

    assert_eq!(error.to_string(), "runtime project root cannot be empty");
}

#[test]
fn project_startup_rejects_play_inputs_without_a_project_root() {
    let scene = b".zircon/play/instance/play-scene.zrscene.json";
    let error = RuntimeProjectConfig::from_abi_startup_config(
        ZrByteSlice::empty(),
        ZrByteSlice {
            data: scene.as_ptr(),
            len: scene.len(),
        },
        ZrByteSlice::empty(),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        RuntimeProjectError::PlaySceneRequiresProject
    ));
}

#[test]
fn project_startup_keeps_the_existing_rel_path_contract_for_play_scene() {
    let root = b"examples/vampire";
    let scene = b".zircon/play/instance/play-scene.zrscene.json";
    let parsed = RuntimeProjectConfig::from_abi_startup_config(
        ZrByteSlice {
            data: root.as_ptr(),
            len: root.len(),
        },
        ZrByteSlice {
            data: scene.as_ptr(),
            len: scene.len(),
        },
        ZrByteSlice::empty(),
    )
    .unwrap()
    .unwrap();

    assert_eq!(
        parsed.play_scene.as_ref().map(|scene| scene.as_str()),
        Some(".zircon/play/instance/play-scene.zrscene.json")
    );
}

#[test]
fn project_startup_rejects_absolute_play_scene_and_blank_report_outlet() {
    let root = b"examples/vampire";
    let absolute_scene = b"C:/outside.scene.toml";
    let scene_error = RuntimeProjectConfig::from_abi_startup_config(
        ZrByteSlice {
            data: root.as_ptr(),
            len: root.len(),
        },
        ZrByteSlice {
            data: absolute_scene.as_ptr(),
            len: absolute_scene.len(),
        },
        ZrByteSlice::empty(),
    )
    .unwrap_err();
    assert!(matches!(
        scene_error,
        RuntimeProjectError::InvalidPlayScene { .. }
    ));

    let blank_outlet = b"  ";
    let outlet_error = RuntimeProjectConfig::from_abi_startup_config(
        ZrByteSlice {
            data: root.as_ptr(),
            len: root.len(),
        },
        ZrByteSlice::empty(),
        ZrByteSlice {
            data: blank_outlet.as_ptr(),
            len: blank_outlet.len(),
        },
    )
    .unwrap_err();
    assert!(matches!(
        outlet_error,
        RuntimeProjectError::EmptyPlayReportPipe
    ));
}

#[test]
fn project_config_parses_project_root_path() {
    let raw = b"examples/vampire";
    let parsed = RuntimeProjectConfig::from_abi_slice(ZrByteSlice {
        data: raw.as_ptr(),
        len: raw.len(),
    })
    .unwrap()
    .unwrap();

    assert_eq!(
        parsed.root_display(),
        ProjectPaths::resolve_path("examples/vampire")
            .unwrap()
            .display_path()
            .display()
            .to_string()
    );
}

#[test]
fn project_config_normalizes_a_manifest_input_to_its_project_root() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("templates")
        .join("projects")
        .join("renderable-empty");
    let config = RuntimeProjectConfig::from_root(root.join("zircon-project.toml")).unwrap();

    assert_eq!(
        config.root_display(),
        ProjectPaths::resolve_existing(&root)
            .unwrap()
            .display_path()
            .display()
            .to_string()
    );
}

#[cfg(windows)]
#[test]
fn project_config_displays_operation_root_without_verbatim_prefix() {
    let project = RuntimeProjectConfig::from_root(r"\\?\C:\ZirconBuilds\stage\project").unwrap();

    assert_eq!(project.root_display(), r"C:\ZirconBuilds\stage\project");
}

#[cfg(windows)]
#[test]
fn project_config_rejects_drive_relative_abi_paths_at_the_resolver_boundary() {
    let raw = br"C:runtime-project";
    let error = RuntimeProjectConfig::from_abi_slice(ZrByteSlice {
        data: raw.as_ptr(),
        len: raw.len(),
    })
    .unwrap_err();

    assert!(matches!(
        error,
        RuntimeProjectError::ResolveProjectRoot { .. }
    ));
    assert!(error
        .to_string()
        .contains("Windows project paths must be drive-rooted, not drive-relative"));
}

#[test]
fn project_startup_snapshot_survives_disk_manifest_rewrite_before_activation() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon_runtime_prepared_project_{}_{}",
        std::process::id(),
        unique
    ));
    fs::create_dir_all(&root).unwrap();
    let manifest_path = root.join("zircon-project.toml");
    ProjectManifest::new(
        "Prepared Snapshot One",
        AssetUri::parse("res://scenes/one.scene.toml").unwrap(),
        1,
    )
    .save(&manifest_path)
    .unwrap();

    let prepared = RuntimeProjectConfig::from_root(&root)
        .unwrap()
        .prepare()
        .unwrap();

    ProjectManifest::new(
        "Prepared Snapshot Two",
        AssetUri::parse("res://scenes/two.scene.toml").unwrap(),
        2,
    )
    .save(&manifest_path)
    .unwrap();

    assert_eq!(
        prepared.manifest.default_scene,
        "res://scenes/one.scene.toml"
    );
    assert!(prepared.project.is_some());
    assert_eq!(prepared.root_display(), root.to_string_lossy());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn prepared_project_normalizes_a_relative_root_before_runtime_consumers_use_it() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let relative_root = PathBuf::from(format!(
        "zircon_runtime_relative_project_{}_{}",
        std::process::id(),
        unique
    ));
    fs::create_dir_all(&relative_root).unwrap();
    ProjectManifest::new(
        "Relative Runtime Root",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(relative_root.join("zircon-project.toml"))
    .unwrap();

    let config = RuntimeProjectConfig::from_root(&relative_root).unwrap();
    assert_eq!(
        config.root_display(),
        std::env::current_dir()
            .unwrap()
            .join(&relative_root)
            .to_string_lossy()
            .into_owned()
    );
    let prepared = config.prepare().unwrap();

    assert_eq!(
        prepared.root_display(),
        std::env::current_dir()
            .unwrap()
            .join(&relative_root)
            .to_string_lossy()
            .into_owned()
    );

    fs::remove_dir_all(relative_root).unwrap();
}

#[test]
fn project_manifest_filters_startup_script_packages() {
    let manifest = RuntimeLoadedProjectManifest {
        default_scene: "res://scenes/main.scene.toml".to_string(),
        ui_roots: Vec::new(),
        plugins: ProjectPluginManifest::default(),
        scripts: ProjectScriptManifest {
            package_roots: vec!["scripts".to_string()],
            startup_packages: vec!["vampire_game".to_string()],
        },
    };

    let packages = manifest
        .filter_startup_packages(vec![
            script_package("debug_tools"),
            script_package("vampire_game"),
        ])
        .unwrap();

    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].package.manifest.name, "vampire_game");
}

#[test]
fn project_manifest_rejects_missing_startup_script_package() {
    let manifest = RuntimeLoadedProjectManifest {
        default_scene: "res://scenes/main.scene.toml".to_string(),
        ui_roots: Vec::new(),
        plugins: ProjectPluginManifest::default(),
        scripts: ProjectScriptManifest {
            package_roots: vec!["scripts".to_string()],
            startup_packages: vec!["vampire_game".to_string()],
        },
    };

    let error = manifest
        .filter_startup_packages(vec![script_package("debug_tools")])
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "runtime startup script package vampire_game was not found"
    );
}

fn script_package(name: &str) -> DiscoveredVmPluginPackage {
    DiscoveredVmPluginPackage {
        backend_name: "mock".to_string(),
        source: VmPluginPackageSource::default(),
        package: VmPluginPackage {
            manifest: VmPluginManifest {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                entry: "main".to_string(),
                capabilities: CapabilitySet::default(),
                management: VmPluginManagementPolicy::default(),
            },
            zr_vm_project: None,
            bytecode: Vec::new(),
        },
    }
}
