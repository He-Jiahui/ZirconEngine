use super::*;

fn activate(lifecycle: &DocumentLifecycleAuthority) -> ActiveSceneDocumentIdentity {
    let root = std::path::Path::new("revision-lifecycle-fixture");
    let session = lifecycle.begin_project_session(root).session;
    lifecycle
        .activate_scene(session, root, "res://scene.scene.toml")
        .unwrap();
    lifecycle.active_scene_identity(root).unwrap()
}

#[test]
fn overwrite_lease_is_bound_to_prompt_observation_and_consumed_once() {
    let lifecycle = DocumentLifecycleAuthority::default();
    let identity = activate(&lifecycle);
    let lease = OverwriteLease::for_observed_source(
        identity.clone(),
        zircon_runtime::scene::world::SceneSourceRevision::Missing,
    );
    let result =
        lifecycle.save_active_scene(identity.project_root(), Some(lease), |_, expected| {
            assert_eq!(
                expected,
                &zircon_runtime::scene::world::SceneSourceRevision::Missing
            );
            Err::<SceneSaveOutcome, _>("publication refused")
        });
    assert!(matches!(
        result,
        Err(SceneSaveError::Persistence("publication refused"))
    ));
    // Admission consumed the lease. Failure never installs its observation as the normal base.
    assert_eq!(
        lifecycle
            .active_scene_identity(identity.project_root())
            .unwrap(),
        identity
    );
    let ordinary = lifecycle.save_active_scene::<&str>(identity.project_root(), None, |_, _| {
        panic!("ordinary Save must not acquire the prompt's expected revision");
    });
    assert!(matches!(
        ordinary,
        Err(SceneSaveError::MissingSourceRevision)
    ));
}

#[test]
fn stale_activation_rejects_overwrite_lease() {
    let lifecycle = DocumentLifecycleAuthority::default();
    let identity = activate(&lifecycle);
    let lease = OverwriteLease::for_observed_source(
        identity.clone(),
        zircon_runtime::scene::world::SceneSourceRevision::Missing,
    );
    let session = lifecycle.project_session(identity.project_root()).unwrap();
    lifecycle
        .activate_scene(session, identity.project_root(), identity.scene_uri())
        .unwrap();
    let result =
        lifecycle.save_active_scene::<&str>(identity.project_root(), Some(lease), |_, _| {
            panic!("a stale activation cannot reach persistence");
        });
    assert!(matches!(result, Err(SceneSaveError::StaleOverwriteLease)));
}
