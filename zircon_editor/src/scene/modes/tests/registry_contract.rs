use super::*;

#[test]
fn mode_registry_creates_the_factory_bound_to_a_scene_mode_descriptor() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let factory_events = events.clone();
    let mut registry = SceneModeRegistry::default();
    registry
        .register(SceneModeRegistration::new(
            scene_mode_descriptor("scene.navigation"),
            move || {
                Box::new(RecordingMode::new(
                    "scene.navigation",
                    InputOutcome::PassThrough,
                    factory_events.clone(),
                )) as Box<dyn EditorSceneMode>
            },
        ))
        .unwrap();

    let mode_id = SceneModeId::new("scene.navigation");
    let mode = registry.create(&mode_id).unwrap();

    assert_eq!(mode.id(), &mode_id);
    assert_eq!(
        registry.descriptor(&mode_id).unwrap().display_name(),
        "Navigation"
    );
}

#[test]
fn mode_registry_rejects_duplicate_unknown_and_mismatched_factories() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut registry = SceneModeRegistry::default();
    registry
        .register(SceneModeRegistration::new(
            scene_mode_descriptor("scene.navigation"),
            recording_factory("scene.navigation", events.clone()),
        ))
        .unwrap();

    let duplicate = registry
        .register(SceneModeRegistration::new(
            scene_mode_descriptor("scene.navigation"),
            recording_factory("scene.navigation", events.clone()),
        ))
        .unwrap_err();
    assert!(matches!(
        duplicate,
        SceneModeRegistryError::DuplicateMode { mode_id }
            if mode_id.as_str() == "scene.navigation"
    ));

    let unknown = registry
        .create(&SceneModeId::new("scene.unknown"))
        .err()
        .expect("unknown mode should be rejected");
    assert!(matches!(
        unknown,
        SceneModeRegistryError::UnknownMode { mode_id }
            if mode_id.as_str() == "scene.unknown"
    ));

    registry
        .register(SceneModeRegistration::new(
            scene_mode_descriptor("scene.mismatch"),
            recording_factory("scene.other", events),
        ))
        .unwrap();
    let mismatch = registry
        .create(&SceneModeId::new("scene.mismatch"))
        .err()
        .expect("factory id mismatch should be rejected");
    assert!(matches!(
        mismatch,
        SceneModeRegistryError::FactoryModeIdMismatch {
            registered_mode_id,
            produced_mode_id,
        } if registered_mode_id.as_str() == "scene.mismatch"
            && produced_mode_id.as_str() == "scene.other"
    ));
}

#[test]
fn builtin_registry_maps_scene_mode_activations_to_their_registered_modes() {
    let registry = builtin_scene_mode_registry();

    assert_eq!(registry.len(), 2);
    for (activation, expected_mode) in [
        (SceneModeActivation::Select, "scene.select"),
        (
            SceneModeActivation::Transform(TransformHandleKind::Move),
            "scene.transform",
        ),
        (
            SceneModeActivation::Transform(TransformHandleKind::Rotate),
            "scene.transform",
        ),
        (
            SceneModeActivation::Transform(TransformHandleKind::Scale),
            "scene.transform",
        ),
    ] {
        let mode_id = activation.mode_id();
        let mode = registry
            .create(&mode_id)
            .expect("every built-in scene mode must resolve through the registry");

        assert_eq!(mode.id().as_str(), expected_mode);
    }

    assert_eq!(
        registry
            .descriptor(&SceneModeId::new("scene.select"))
            .expect("the select mode must have a descriptor")
            .display_name(),
        "Select"
    );
    assert_eq!(
        registry
            .descriptor(&SceneModeId::new("scene.transform"))
            .expect("the transform mode must have a descriptor")
            .display_name(),
        "Transform"
    );
}
