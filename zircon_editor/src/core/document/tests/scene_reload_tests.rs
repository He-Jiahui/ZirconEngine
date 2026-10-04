use std::path::Path;

use super::{
    ActiveSceneReloader, DocumentLifecycleAuthority, SceneDocumentReloadCoordinator,
    SceneDocumentReloadError, SceneDocumentReloadOutcome,
};

#[derive(Default)]
struct RecordingReloader {
    prepare_count: usize,
    install_count: usize,
    reject_prepare: bool,
}

impl ActiveSceneReloader for RecordingReloader {
    type Error = &'static str;

    fn prepare_active_scene_reload(&mut self) -> Result<(), Self::Error> {
        self.prepare_count += 1;
        if self.reject_prepare {
            return Err("dirty scene requires a conflict decision");
        }
        Ok(())
    }

    fn install_active_scene_reload(&mut self) -> Result<(), Self::Error> {
        self.install_count += 1;
        Ok(())
    }
}

#[test]
fn reload_installs_only_while_the_exact_scene_identity_is_active() {
    let lifecycle = DocumentLifecycleAuthority::default();
    let root = Path::new("E:/projects/scene-reload");
    let session = lifecycle.begin_project_session(root).session;
    lifecycle
        .activate_scene(session, root, "res://scenes/main.scene.toml")
        .unwrap();
    let expected = lifecycle.active_scene_identity(root).unwrap();
    let coordinator = SceneDocumentReloadCoordinator::new(&lifecycle);
    let mut reloader = RecordingReloader::default();

    assert_eq!(
        coordinator.reload(&expected, &mut reloader).unwrap(),
        SceneDocumentReloadOutcome::Reloaded {
            document: expected.document()
        }
    );
    assert_eq!(reloader.prepare_count, 1);
    assert_eq!(reloader.install_count, 1);

    lifecycle
        .activate_scene(session, root, "res://scenes/secondary.scene.toml")
        .unwrap();
    assert_eq!(
        coordinator.reload(&expected, &mut reloader).unwrap(),
        SceneDocumentReloadOutcome::Superseded
    );
    assert_eq!(reloader.prepare_count, 1);
    assert_eq!(reloader.install_count, 1);

    lifecycle
        .activate_scene(session, root, "res://scenes/main.scene.toml")
        .unwrap();
    assert_eq!(
        coordinator.reload(&expected, &mut reloader).unwrap(),
        SceneDocumentReloadOutcome::Superseded
    );
    assert_eq!(reloader.prepare_count, 1);
    assert_eq!(reloader.install_count, 1);
}

#[test]
fn dirty_reload_rejection_never_reaches_installation() {
    let lifecycle = DocumentLifecycleAuthority::default();
    let root = Path::new("E:/projects/dirty-scene-reload");
    let session = lifecycle.begin_project_session(root).session;
    lifecycle
        .activate_scene(session, root, "res://scenes/main.scene.toml")
        .unwrap();
    let expected = lifecycle.active_scene_identity(root).unwrap();
    let mut reloader = RecordingReloader {
        reject_prepare: true,
        ..RecordingReloader::default()
    };

    let error = SceneDocumentReloadCoordinator::new(&lifecycle)
        .reload(&expected, &mut reloader)
        .unwrap_err();

    assert!(matches!(
        error,
        SceneDocumentReloadError::Transition("dirty scene requires a conflict decision")
    ));
    assert_eq!(reloader.prepare_count, 1);
    assert_eq!(reloader.install_count, 0);
}
