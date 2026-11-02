use zircon_runtime::builtin::RuntimePluginId;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    PluginSelectionResolutionStatus, ProjectPluginManifest, ProjectPluginSelection,
};

use super::first_party_editor_plugin_registrations_for_manifest;

#[test]
fn editor_catalog_preallocates_manifest_projection_storage() {
    let source = include_str!("../catalog.rs");
    let projection = source
        .split("pub fn first_party_editor_plugin_registrations_for_manifest")
        .nth(1)
        .and_then(|text| {
            text.split("pub fn first_party_registration_for_editor_plugin")
                .next()
        })
        .expect("read editor catalog manifest projection");

    assert!(projection.contains("resolve_plugin_selections(target_mode, manifest"));
}

#[test]
fn non_editor_target_reports_each_selection_without_loading_editor_providers() {
    let mut selected =
        ProjectPluginSelection::runtime_plugin(RuntimePluginId::Navigation, true, true)
            .with_target_modes([RuntimeTargetMode::ClientRuntime]);
    selected.editor_crate = Some("zircon_plugin_navigation_editor".to_owned());
    let editor_only =
        ProjectPluginSelection::runtime_plugin(RuntimePluginId::Navigation, true, true)
            .with_target_modes([RuntimeTargetMode::EditorHost]);
    let manifest = ProjectPluginManifest {
        selections: vec![selected, editor_only],
    };

    let report = first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::ClientRuntime,
        &manifest,
    );
    assert!(report.is_empty());
    assert_eq!(
        report
            .outcomes()
            .iter()
            .map(|outcome| outcome.status)
            .collect::<Vec<_>>(),
        [
            PluginSelectionResolutionStatus::Unsupported,
            PluginSelectionResolutionStatus::SkippedTargetMismatch,
        ]
    );
    assert_eq!(
        report
            .into_registrations_if_required_resolved_where(|selection| {
                selection.editor_crate.is_some()
            })
            .expect_err("an explicit required editor provider must not silently disappear")
            .failures()
            .len(),
        1
    );
}

#[cfg(feature = "navigation-editor-plugin")]
#[test]
fn selected_navigation_provider_projects_typed_runtime_consumer() {
    let manifest = manifest_with_navigation(true, RuntimeTargetMode::EditorHost);

    let registrations = first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &manifest,
    );

    assert_eq!(registrations.len(), 1);
    let registration = &registrations[0];
    assert_eq!(registration.package_manifest.id, "navigation");
    assert!(registration.is_success());
    assert_eq!(registration.runtime_event_consumers.manifests().len(), 1);
}

#[cfg(feature = "navigation-editor-plugin")]
#[test]
fn disabled_or_wrong_target_navigation_provider_is_not_projected() {
    let disabled = manifest_with_navigation(false, RuntimeTargetMode::EditorHost);
    assert!(first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &disabled,
    )
    .is_empty());

    let runtime_only = manifest_with_navigation(true, RuntimeTargetMode::ClientRuntime);
    assert!(first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &runtime_only,
    )
    .is_empty());
}

#[cfg(feature = "sound-editor-plugin")]
#[test]
fn selected_sound_provider_makes_audio_source_customization_available() {
    use std::sync::Arc;

    use zircon_editor::core::extension::{
        InspectTarget, InspectTargetType, InspectorCustomizationChain,
    };
    use zircon_editor::core::plugin::{
        EditorPluginHandle, EditorPluginLoadingPhase, EditorPluginManager,
    };
    use zircon_runtime::core::framework::sound::{
        AUDIO_LISTENER_COMPONENT_TYPE, AUDIO_SOURCE_COMPONENT_TYPE, AUDIO_VOLUME_COMPONENT_TYPE,
    };
    use zircon_runtime::plugin::PluginModuleKind;
    use zircon_runtime::scene::{NodeKind, World};

    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )
        .with_target_modes([RuntimeTargetMode::EditorHost])],
    };
    let registrations = first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &manifest,
    );
    assert_eq!(registrations.len(), 1);
    let registration = &registrations[0];
    assert_eq!(registration.package_manifest.id, "sound");
    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert!(registration
        .capabilities
        .iter()
        .any(|capability| capability == "editor.extension.sound_authoring"));

    let mut component_types = registration
        .package_manifest
        .components
        .iter()
        .map(|descriptor| descriptor.type_id.as_str())
        .collect::<Vec<_>>();
    component_types.sort_unstable();
    assert_eq!(
        component_types,
        [
            AUDIO_LISTENER_COMPONENT_TYPE,
            AUDIO_SOURCE_COMPONENT_TYPE,
            AUDIO_VOLUME_COMPONENT_TYPE,
        ]
    );
    let mut world = World::empty();
    for descriptor in &registration.package_manifest.components {
        world
            .register_component_type(descriptor.clone())
            .expect("sound component descriptor should register");
    }
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    // Match the SoundSourceDescriptor::clip defaults used by sound runtime tests.
    let payload = r#"{
        "id": null, "input": {"Clip": 1}, "output_track": 1, "sends": [],
        "position": [0.0, 0.0, 0.0], "forward": [0.0, 0.0, 1.0],
        "velocity": [0.0, 0.0, 0.0], "gain": 1.0, "speed": 1.0,
        "looped": false, "playing": true, "muted": false,
        "completion_action": "None", "start_seconds": null, "duration_seconds": null,
        "spatial": {
            "spatial_blend": 0.0, "spatial_scale": null,
            "min_distance": 1.0, "max_distance": 50.0, "attenuation": "InverseDistance",
            "cone_inner_degrees": 360.0, "cone_outer_degrees": 360.0,
            "doppler_factor": 1.0, "occlusion_enabled": false, "convolution_send": null
        },
        "parameter_bindings": [], "gameplay_emitter": null
    }"#
    .parse()
    .expect("default sound source payload should be valid JSON");
    world
        .set_dynamic_component(entity, AUDIO_SOURCE_COMPONENT_TYPE, payload)
        .expect("default sound source should attach");
    let schema = world
        .reflect_schema(AUDIO_SOURCE_COMPONENT_TYPE)
        .expect("AudioSource must have a reflection schema");
    assert!(schema
        .type_info
        .fields
        .iter()
        .any(|field| field.name == "gain" && field.editable));

    // Raw registration extensions are private; read the same provider through its public
    // active manager snapshot. Remove the attached editor module before rebuilding it.
    let mut runtime_manifest = registration.package_manifest.clone();
    runtime_manifest
        .modules
        .retain(|module| module.kind != PluginModuleKind::Editor);
    let plugin: EditorPluginHandle = Arc::new(zircon_plugin_sound_editor::editor_plugin());
    let manager = EditorPluginManager::from_plugins([(plugin, runtime_manifest)]).unwrap();
    manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .unwrap();
    let snapshot = manager.apply_project_manifest(&manifest).unwrap();
    let extensions = snapshot.active_extensions();
    assert!(extensions.is_success(), "{:?}", extensions.diagnostics);
    let customizations = extensions.registry.inspector_customizations();
    assert_eq!(customizations.len(), 3);
    let mut chain = InspectorCustomizationChain::default();
    for descriptor in &customizations {
        chain.register(Arc::new((*descriptor).clone())).unwrap();
    }
    let target = InspectTarget::new(
        InspectTargetType::new(AUDIO_SOURCE_COMPONENT_TYPE).unwrap(),
        "component:sound.Component.AudioSource",
    )
    .unwrap();
    // Gate parity with the inspector's schema + customization check; G5 capability
    // filtering remains covered by editor plugin_contributions tests, not a GUI receipt.
    assert!(chain.matching(&target).is_some());

    let root = crate::catalog::sound_ui_template_root()
        .canonicalize()
        .unwrap();
    let templates = extensions.registry.ui_templates();
    assert_eq!(templates.len(), 2);
    for uri in templates
        .iter()
        .map(|template| template.ui_document())
        .chain(
            customizations
                .iter()
                .map(|descriptor| descriptor.surface().ui_document()),
        )
    {
        let relative = uri.strip_prefix("plugins://sound/").unwrap();
        let document = root.join(relative).canonicalize().unwrap();
        assert!(document.starts_with(&root) && document.is_file(), "{uri}");
    }

    // Cached discovery must still return independently mutable reports through
    // both the direct provider and manifest projection APIs.
    let mut caller_report =
        crate::first_party_registration_for_editor_plugin(RuntimePluginId::Sound)
            .expect("sound provider should be linked");
    caller_report.package_manifest.components.clear();
    caller_report.capabilities.clear();
    caller_report
        .diagnostics
        .push("caller-local diagnostic".to_owned());
    let fresh = first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &manifest,
    );
    assert_eq!(fresh.len(), 1);
    assert!(fresh[0].is_success());
    assert_eq!(fresh[0].package_manifest, registration.package_manifest);
    assert_eq!(fresh[0].capabilities, registration.capabilities);
}

#[cfg(feature = "particles-editor-plugin")]
#[test]
fn selected_particles_provider_makes_particle_system_customization_available() {
    use std::sync::Arc;

    use zircon_editor::core::asset::AssetTypeId;
    use zircon_editor::core::extension::{
        InspectTarget, InspectTargetType, InspectorCustomizationChain,
    };
    use zircon_editor::core::plugin::{
        EditorPluginHandle, EditorPluginLoadingPhase, EditorPluginManager,
    };
    use zircon_plugin_particles_editor::PARTICLES_AUTHORING_CAPABILITY;
    use zircon_runtime::plugin::PluginModuleKind;
    use zircon_runtime::scene::{NodeKind, World};

    // Match the runtime descriptor's public type id without another direct dependency.
    const PARTICLE_SYSTEM_COMPONENT_TYPE: &str = "particles.Component.ParticleSystem";

    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Particles,
            true,
            false,
        )
        .with_target_modes([RuntimeTargetMode::EditorHost])],
    };
    let registrations = first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &manifest,
    );
    assert_eq!(registrations.len(), 1);
    let registration = &registrations[0];
    assert_eq!(registration.package_manifest.id, "particles");
    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert!(registration
        .capabilities
        .iter()
        .any(|capability| capability == PARTICLES_AUTHORING_CAPABILITY));
    assert_eq!(registration.package_manifest.components.len(), 1);
    assert_eq!(
        registration.package_manifest.components[0].type_id,
        PARTICLE_SYSTEM_COMPONENT_TYPE
    );

    let mut world = World::empty();
    for descriptor in &registration.package_manifest.components {
        world
            .register_component_type(descriptor.clone())
            .expect("particles component descriptor should register");
    }
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    let payload = r#"{
        "asset": "cpu_sprite_system", "playing": true, "backend": "Cpu",
        "time_scale": 1.0, "seed": 0
    }"#
    .parse()
    .expect("particle system payload should be valid JSON");
    world
        .set_dynamic_component(entity, PARTICLE_SYSTEM_COMPONENT_TYPE, payload)
        .expect("particle system should attach");
    let schema = world
        .reflect_schema(PARTICLE_SYSTEM_COMPONENT_TYPE)
        .expect("ParticleSystem must have a reflection schema");
    assert!(schema
        .type_info
        .fields
        .iter()
        .any(|field| field.name == "time_scale" && field.editable));

    // Read private registration extensions through the public active manager snapshot,
    // as in the settled sound test, rebuilding only the attached editor module.
    let mut runtime_manifest = registration.package_manifest.clone();
    runtime_manifest
        .modules
        .retain(|module| module.kind != PluginModuleKind::Editor);
    let plugin: EditorPluginHandle = Arc::new(zircon_plugin_particles_editor::editor_plugin());
    let manager = EditorPluginManager::from_plugins([(plugin, runtime_manifest)]).unwrap();
    manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .unwrap();
    let snapshot = manager.apply_project_manifest(&manifest).unwrap();
    let extensions = snapshot.active_extensions();
    assert!(extensions.is_success(), "{:?}", extensions.diagnostics);
    let customizations = extensions.registry.inspector_customizations();
    assert_eq!(customizations.len(), 1);
    let mut chain = InspectorCustomizationChain::default();
    for descriptor in &customizations {
        chain.register(Arc::new((*descriptor).clone())).unwrap();
    }
    let target = InspectTarget::new(
        InspectTargetType::new(PARTICLE_SYSTEM_COMPONENT_TYPE).unwrap(),
        "component:particles.Component.ParticleSystem",
    )
    .unwrap();
    // Schema/customization gate parity; G5 capability filtering remains covered by
    // editor plugin_contributions tests. This is not a GUI receipt.
    assert!(chain.matching(&target).is_some());

    let root = crate::catalog::particles_ui_template_root()
        .canonicalize()
        .unwrap();
    let templates = extensions.registry.ui_templates();
    assert_eq!(templates.len(), 2);
    // The manager materializes asset contributions into its separate type registry.
    let particle_asset_type = extensions
        .asset_types
        .get(
            &AssetTypeId::parse(zircon_plugin_particles_editor::PARTICLES_SYSTEM_ASSET_KIND)
                .unwrap(),
        )
        .expect("particle system asset type should materialize");
    let asset_documents = particle_asset_type
        .creation_templates()
        .iter()
        .filter_map(|template| template.default_document())
        .collect::<Vec<_>>();
    assert_eq!(asset_documents.len(), 1);
    let documents = templates
        .iter()
        .map(|template| template.ui_document())
        .chain(
            customizations
                .iter()
                .map(|descriptor| descriptor.surface().ui_document()),
        )
        .chain(asset_documents)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(documents.len(), 4);
    for uri in documents {
        let relative = uri.strip_prefix("plugins://particles/").unwrap();
        let document = root.join(relative).canonicalize().unwrap();
        assert!(document.starts_with(&root) && document.is_file(), "{uri}");
    }

    // Cache immutable discovery metadata without sharing caller-local report mutations.
    let mut caller_report =
        crate::first_party_registration_for_editor_plugin(RuntimePluginId::Particles)
            .expect("particles provider should be linked");
    caller_report.package_manifest.components.clear();
    caller_report.capabilities.clear();
    caller_report
        .diagnostics
        .push("caller-local diagnostic".to_owned());
    let fresh = first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &manifest,
    );
    assert_eq!(fresh.len(), 1);
    assert!(fresh[0].is_success());
    assert_eq!(fresh[0].package_manifest, registration.package_manifest);
    assert_eq!(fresh[0].capabilities, registration.capabilities);
}

#[cfg(feature = "navigation-editor-plugin")]
#[test]
fn duplicate_project_selections_produce_one_editor_registration() {
    let selection =
        ProjectPluginSelection::runtime_plugin(RuntimePluginId::Navigation, true, false)
            .with_target_modes([RuntimeTargetMode::EditorHost]);
    let manifest = ProjectPluginManifest {
        selections: vec![selection.clone(), selection],
    };

    let registrations = first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &manifest,
    );

    assert_eq!(registrations.len(), 1);
    assert_eq!(
        registrations.outcomes()[1].status,
        zircon_runtime::core::framework::project::PluginSelectionResolutionStatus::Duplicate
    );
}

#[cfg(feature = "neural-editor-plugin")]
#[test]
fn selected_neural_provider_projects_real_onnx_authoring_contributions() {
    let manifest = manifest_with_neural(true, RuntimeTargetMode::EditorHost);

    let registrations = first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &manifest,
    );

    assert_eq!(registrations.len(), 1);
    let registration = &registrations[0];
    assert_eq!(registration.package_manifest.id, "neural");
    assert!(registration.is_success());
    assert!(registration
        .capabilities
        .iter()
        .any(|capability| capability == "editor.extension.neural_authoring"));
    assert!(registration.package_manifest.modules.iter().any(|module| {
        module.name == "neural.editor" && module.crate_name == "zircon_plugin_neural_editor"
    }));
}

#[cfg(feature = "neural-editor-plugin")]
#[test]
fn disabled_wrong_target_or_duplicate_neural_selections_project_correctly() {
    let disabled = manifest_with_neural(false, RuntimeTargetMode::EditorHost);
    assert!(first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &disabled,
    )
    .is_empty());

    let runtime_only = manifest_with_neural(true, RuntimeTargetMode::ClientRuntime);
    assert!(first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &runtime_only,
    )
    .is_empty());

    let selection =
        ProjectPluginSelection::runtime_plugin(RuntimePluginId::new("neural"), true, false)
            .with_target_modes([RuntimeTargetMode::EditorHost]);
    let duplicate = ProjectPluginManifest {
        selections: vec![selection.clone(), selection],
    };
    assert_eq!(
        first_party_editor_plugin_registrations_for_manifest(
            RuntimeTargetMode::EditorHost,
            &duplicate,
        )
        .len(),
        1
    );
}

#[cfg(feature = "navigation-editor-plugin")]
fn manifest_with_navigation(enabled: bool, target: RuntimeTargetMode) -> ProjectPluginManifest {
    ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Navigation,
            enabled,
            false,
        )
        .with_target_modes([target])],
    }
}

#[cfg(feature = "neural-editor-plugin")]
fn manifest_with_neural(enabled: bool, target: RuntimeTargetMode) -> ProjectPluginManifest {
    ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::new("neural"),
            enabled,
            false,
        )
        .with_target_modes([target])],
    }
}
