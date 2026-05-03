use super::*;
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_plugin_particles_runtime::PARTICLE_SYSTEM_COMPONENT_TYPE;

const CPU_SPRITE_TEMPLATE: &str = include_str!("../../templates/cpu_sprite_system.toml");
const AUTHORING_UI_TEMPLATE: &str = include_str!("../authoring.ui.toml");
const PREVIEW_UI_TEMPLATE: &str = include_str!("../preview.ui.toml");
const COMPONENT_DRAWER_UI_TEMPLATE: &str = include_str!("../particle_system.drawer.ui.toml");

fn operation(path: &str) -> EditorOperationPath {
    EditorOperationPath::parse(path).expect("valid particles test operation path")
}

#[test]
fn particles_editor_plugin_contributes_authoring_extensions() {
    let registration = plugin_registration();
    let create_asset = operation("Particles.Authoring.CreateCpuSpriteAsset");

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert!(registration
        .capabilities
        .contains(&PARTICLES_AUTHORING_CAPABILITY.to_string()));
    assert!(registration
        .extensions
        .views()
        .iter()
        .any(|view| view.id() == PARTICLES_AUTHORING_VIEW_ID));
    assert!(registration
        .extensions
        .views()
        .iter()
        .any(|view| view.id() == PARTICLES_PREVIEW_VIEW_ID));
    assert!(registration
        .extensions
        .drawers()
        .iter()
        .any(|drawer| drawer.id() == PARTICLES_DRAWER_ID));
    assert!(registration
        .extensions
        .ui_templates()
        .iter()
        .any(|template| template.id() == PARTICLES_TEMPLATE_ID));
    assert!(registration
        .extensions
        .ui_templates()
        .iter()
        .any(|template| template.id() == PARTICLES_PREVIEW_TEMPLATE_ID));
    assert!(registration
        .extensions
        .component_drawers()
        .iter()
        .any(|drawer| {
            drawer.component_type() == PARTICLE_SYSTEM_COMPONENT_TYPE
                && drawer.controller() == PARTICLES_COMPONENT_DRAWER_ID
        }));
    assert!(registration
        .extensions
        .asset_editors()
        .iter()
        .any(|editor| {
            editor.asset_kind() == PARTICLES_SYSTEM_ASSET_KIND
                && editor.view_id() == PARTICLES_AUTHORING_VIEW_ID
        }));
    assert!(registration
        .extensions
        .asset_creation_templates()
        .iter()
        .any(|template| {
            template.id() == PARTICLES_CPU_SPRITE_TEMPLATE_ID
                && template.asset_kind() == PARTICLES_SYSTEM_ASSET_KIND
                && template.default_document() == Some(PARTICLES_CPU_SPRITE_TEMPLATE_DOCUMENT)
                && template.operation() == &create_asset
        }));
    assert!(registration
        .extensions
        .menu_items()
        .iter()
        .any(|menu| menu.operation().as_str() == "View.particles.authoring.Open"));
    assert!(registration
        .extensions
        .menu_items()
        .iter()
        .any(|menu| menu.operation().as_str() == "View.particles.preview.Open"));
    assert!(registration
        .extensions
        .operations()
        .descriptors()
        .any(|operation| operation.path().as_str() == "View.particles.authoring.Open"));
    assert!(registration
        .extensions
        .operations()
        .descriptors()
        .any(|operation| operation.path().as_str() == "View.particles.preview.Open"));

    let create_asset_descriptor = registration
        .extensions
        .operations()
        .descriptor(&create_asset)
        .expect("create CPU sprite asset operation should be registered");
    assert_eq!(
        create_asset_descriptor.payload_schema_id(),
        Some("particles.create_cpu_sprite_asset.v1")
    );
    assert!(!create_asset_descriptor.callable_from_remote());
    assert_eq!(
        create_asset_descriptor.menu_path(),
        Some("Plugins/Particles/Create CPU Sprite Asset")
    );

    let particles_authoring_capability = [PARTICLES_AUTHORING_CAPABILITY.to_string()];
    assert!(registration.extensions.menu_items().iter().any(|menu| {
        menu.path() == "Plugins/Particles/Create CPU Sprite Asset"
            && menu.operation() == &create_asset
            && !menu.enabled()
            && menu.required_capabilities() == particles_authoring_capability.as_slice()
    }));

    assert!(CPU_SPRITE_TEMPLATE.contains("cpu_sprite_system"));
    assert_cpu_sprite_template_shape(CPU_SPRITE_TEMPLATE);
    assert_ui_template_shape(
        AUTHORING_UI_TEMPLATE,
        "particles.authoring",
        "ParticlesAuthoringRoot",
        &[
            "ParticlesEmitterList",
            "ParticlesModuleStack",
            "ParticlesCurveEditor",
            "ParticlesDiagnosticsPanel",
        ],
    );
    assert_ui_template_shape(
        PREVIEW_UI_TEMPLATE,
        "particles.preview",
        "ParticlesPreviewRoot",
        &[
            "ParticlesPreviewViewport",
            "ParticlesPreviewTransport",
            "ParticlesPreviewStats",
        ],
    );
    assert_ui_template_shape(
        COMPONENT_DRAWER_UI_TEMPLATE,
        "particles.particle_system.drawer",
        "ParticlesComponentDrawerRoot",
        &[
            "ParticlesComponentAssetRow",
            "ParticlesComponentPlaybackRow",
            "ParticlesComponentBackendRow",
            "ParticlesComponentDiagnosticsRow",
        ],
    );

    for path in [
        "Particles.Authoring.AddComponent",
        "Particles.Authoring.OpenAsset",
        "Particles.Authoring.AddEmitter",
        "Particles.Authoring.AddModule",
        "Particles.Authoring.EditCurve",
        "Particles.Authoring.ValidateAsset",
        "Particles.Preview.Play",
        "Particles.Preview.Pause",
        "Particles.Preview.Stop",
        "Particles.Preview.Rewind",
        "Particles.Preview.Warmup",
    ] {
        let operation = operation(path);
        let descriptor = registration
            .extensions
            .operations()
            .descriptor(&operation)
            .unwrap_or_else(|| panic!("operation {path} should be registered"));
        assert!(!descriptor.callable_from_remote());

        let menu = registration
            .extensions
            .menu_items()
            .into_iter()
            .find(|menu| menu.operation() == &operation)
            .unwrap_or_else(|| panic!("operation {path} should have a menu row"));
        assert!(!menu.enabled(), "menu row for {path} should be disabled");
    }
}

fn assert_cpu_sprite_template_shape(template: &str) {
    assert!(template.contains("[system]"));
    assert!(template.contains("id = \"cpu_sprite_system\""));
    assert!(template.contains("backend = \"Cpu\""));
    assert!(template.contains("looped = true"));
    assert!(template.contains("[[emitters]]"));
    assert!(template.contains("id = \"sprite_emitter\""));
    assert!(template.contains("max_particles = 256"));
    assert!(template.contains("spawn_rate_per_second = 32.0"));
    assert!(template.contains("[emitters.shape]"));
    assert!(template.contains("[[emitters.color_over_lifetime]]"));
    assert!(template.contains("[[emitters.size_over_lifetime]]"));
}

fn assert_ui_template_shape(template: &str, id: &str, root: &str, controls: &[&str]) {
    assert!(template.contains("[asset]"));
    assert!(template.contains(&format!("id = \"{id}\"")));
    assert!(template.contains("kind = \"layout\""));
    assert!(template.contains(&format!("control_id = \"{root}\"")));
    for control in controls {
        assert!(
            template.contains(&format!("control_id = \"{control}\"")),
            "template {id} should contain control {control}"
        );
    }
}
