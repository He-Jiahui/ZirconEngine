//! Config writer lifetime coverage without compiling the runtime's unrelated unit tests.
#![cfg(feature = "native-host")]

#[path = "../src/dynamic_api/session/project/play_config.rs"]
mod play_config;

use std::time::{SystemTime, UNIX_EPOCH};
use zircon_runtime::core::CoreRuntime;

#[test]
fn play_snapshot_config_writers_coexist_and_release_on_retirement() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("play-config-{}-{unique}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let bind = |path| {
        let runtime = CoreRuntime::new();
        let mut descriptor = zircon_runtime::foundation::module_descriptor();
        zircon_runtime::foundation::bind_config_file_path(&mut descriptor, path).unwrap();
        runtime.register_module(descriptor).unwrap();
        runtime.activate_registered_modules().unwrap();
        runtime
    };
    let edit = bind(root.join("editor-config.json"));
    let mut owners = Vec::new();
    for id in ["one", "two"] {
        let relative = format!(".zircon/play/{id}/play-scene.zrscene.json");
        let snapshot = root.join(&relative);
        std::fs::create_dir_all(snapshot.parent().unwrap()).unwrap();
        let path = play_config::snapshot_config_path(&relative, &snapshot).unwrap();
        assert_eq!(path.parent(), snapshot.parent());
        owners.push((path.clone(), bind(path)));
    }
    assert_ne!(owners[0].0, owners[1].0);
    let retired = owners.remove(0);
    let path = retired.0;
    drop(retired.1);
    // Re-admission of the exact path proves that retirement released its writer.
    drop(bind(path));
    drop(owners);
    drop(edit);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn ordinary_scenes_do_not_receive_play_snapshot_config_paths() {
    for relative in [
        "assets/main.scene.toml",
        "assets/main.zrscene.json",
        ".zircon/play-other/main.zrscene.json",
        ".zircon/play/one/main.scene.toml",
    ] {
        assert!(
            play_config::snapshot_config_path(relative, std::path::Path::new(relative)).is_none()
        );
    }
}
