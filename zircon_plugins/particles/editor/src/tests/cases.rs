use std::sync::Arc;

use super::*;
use zircon_editor::core::commands::{
    CommandEvalCtx, EditorCommandRegistry, MenuBarModel, MenuItemModel,
};
use zircon_editor::core::editor_extension::EditorExtensionRegistry;
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_editor::core::i18n::EditorI18nService;
use zircon_editor::core::plugin::{
    EditorPluginHandle, EditorPluginLoadingPhase, EditorPluginManager, EditorPluginState,
};
use zircon_editor::EditorPlugin;
use zircon_plugin_particles_runtime::PARTICLE_SYSTEM_COMPONENT_TYPE;

const CPU_SPRITE_TEMPLATE: &str = include_str!("../../../templates/cpu_sprite_system.toml");
const AUTHORING_UI_TEMPLATE: &str = include_str!("../../authoring.zui");
const PREVIEW_UI_TEMPLATE: &str = include_str!("../../preview.zui");
const COMPONENT_DRAWER_UI_TEMPLATE: &str = include_str!("../../particle_system.drawer.zui");

fn operation(path: &str) -> EditorOperationPath {
    EditorOperationPath::parse(path).expect("valid particles test operation path")
}

#[test]
fn particles_editor_plugin_contributes_authoring_extensions() {
    let registration = plugin_registration();
    let mut registry = EditorExtensionRegistry::default();
    editor_plugin()
        .register_editor_extensions(&mut registry)
        .expect("particles authoring registration");
    let commands = active_command_registry();
    // Menu enablement belongs to the command projection, not raw extension rows.
    let context = CommandEvalCtx::interactive();
    let i18n = EditorI18nService::default();
    let locale = i18n.active_locale();
    let menus = commands.menu_bar_model(&i18n, &locale, &context);
    let create_asset = operation("particles.authoring.create_cpu_sprite_asset");

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert!(registration
        .capabilities
        .contains(&PARTICLES_AUTHORING_CAPABILITY.to_string()));
    assert!(registry
        .views()
        .iter()
        .any(|view| view.id() == PARTICLES_AUTHORING_VIEW_ID));
    assert!(registry
        .views()
        .iter()
        .any(|view| view.id() == PARTICLES_PREVIEW_VIEW_ID));
    assert!(registry
        .drawers()
        .iter()
        .any(|drawer| drawer.id() == PARTICLES_DRAWER_ID));
    assert!(registry
        .ui_templates()
        .iter()
        .any(|template| template.id() == PARTICLES_TEMPLATE_ID));
    assert!(registry
        .ui_templates()
        .iter()
        .any(|template| template.id() == PARTICLES_PREVIEW_TEMPLATE_ID));
    assert!(registry
        .inspector_customizations()
        .iter()
        .any(|customization| {
            customization.target_type() == PARTICLE_SYSTEM_COMPONENT_TYPE
                && customization.surface().controller() == PARTICLES_COMPONENT_DRAWER_ID
        }));
    let particle_type = registry
        .asset_type_contributions()
        .into_iter()
        .find(|contribution| contribution.asset_type().as_str() == PARTICLES_SYSTEM_ASSET_KIND)
        .expect("particle system asset type contribution");
    assert_eq!(
        particle_type.toolkit().unwrap().view_id(),
        PARTICLES_AUTHORING_VIEW_ID
    );
    assert!(particle_type.creation_templates().iter().any(|template| {
        template.id() == PARTICLES_CPU_SPRITE_TEMPLATE_ID
            && template.default_document() == Some(PARTICLES_CPU_SPRITE_TEMPLATE_DOCUMENT)
            && template.operation() == &create_asset
    }));
    assert!(menu_item(&menus, &operation("view.particles.authoring.open")).is_some());
    assert!(menu_item(&menus, &operation("view.particles.preview.open")).is_some());
    assert!(commands
        .commands()
        .any(|operation| operation.id().as_str() == "view.particles.authoring.open"));
    assert!(commands
        .commands()
        .any(|operation| operation.id().as_str() == "view.particles.preview.open"));

    let create_asset_descriptor = commands
        .command(&create_asset)
        .expect("create CPU sprite asset operation should be registered");
    assert_eq!(
        create_asset_descriptor.payload_schema_id(),
        Some("particles.create_cpu_sprite_asset.v1")
    );
    assert!(!create_asset_descriptor.callable_from_remote());
    assert!(create_asset_descriptor
        .required_capabilities()
        .iter()
        .any(|capability| capability == PARTICLES_AUTHORING_CAPABILITY));
    assert_eq!(
        create_asset_descriptor
            .menu_path()
            .expect("create command menu path")
            .stable_path(),
        "plugins/particles/particles.authoring.create_cpu_sprite_asset"
    );

    assert!(registry.menu_items().is_empty());

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
        "particles.authoring.add_component",
        "particles.authoring.open_asset",
        "particles.authoring.add_emitter",
        "particles.authoring.add_module",
        "particles.authoring.edit_curve",
        "particles.authoring.validate_asset",
        "particles.preview.play",
        "particles.preview.pause",
        "particles.preview.stop",
        "particles.preview.rewind",
        "particles.preview.warmup",
    ] {
        let operation = operation(path);
        let descriptor = commands
            .command(&operation)
            .unwrap_or_else(|| panic!("operation {path} should be registered"));
        assert!(!descriptor.callable_from_remote());
        assert!(descriptor
            .required_capabilities()
            .iter()
            .any(|capability| capability == PARTICLES_AUTHORING_CAPABILITY));

        let menu = menu_item(&menus, &operation)
            .unwrap_or_else(|| panic!("operation {path} should have a menu row"));
        assert!(
            !menu.enabled,
            "menu row for {path} requires the particles authoring capability"
        );
    }
    let authorized = context.with_capabilities([PARTICLES_AUTHORING_CAPABILITY]);
    let authorized_menus = commands.menu_bar_model(&i18n, &locale, &authorized);
    assert!(
        menu_item(&authorized_menus, &operation("particles.preview.play"))
            .expect("authorized preview play menu")
            .enabled
    );
}

fn active_command_registry() -> EditorCommandRegistry {
    let plugin: EditorPluginHandle = Arc::new(editor_plugin());
    let manager = EditorPluginManager::from_plugins([(
        plugin,
        zircon_plugin_particles_runtime::package_manifest(),
    )])
    .expect("particles plugin admission");
    assert!(manager
        .state_snapshot()
        .active_extensions()
        .registry
        .pending_commands()
        .next()
        .is_none());
    let active = manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("particles plugin activation");
    assert_eq!(
        active.entry(PLUGIN_ID).map(|entry| entry.state()),
        Some(EditorPluginState::Active)
    );
    let extensions = active.active_extensions();
    assert!(extensions.is_success(), "{:?}", extensions.diagnostics);
    EditorCommandRegistry::new(extensions.registry.pending_commands().cloned().collect())
        .expect("active particles command projection")
}

fn menu_item<'a>(
    menus: &'a MenuBarModel,
    operation: &EditorOperationPath,
) -> Option<&'a MenuItemModel> {
    let mut items = menus
        .menus
        .iter()
        .flat_map(|menu| &menu.items)
        .collect::<Vec<_>>();
    while let Some(item) = items.pop() {
        if item.operation_path.as_ref() == Some(operation) {
            return Some(item);
        }
        items.extend(&item.children);
    }
    None
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
    assert!(template.contains("kind = \"view\""));
    assert!(template.contains("version = 2"));
    assert!(template.contains(&format!("control_id = \"{root}\"")));
    for control in controls {
        assert!(
            template.contains(&format!("control_id = \"{control}\"")),
            "template {id} should contain control {control}"
        );
    }
}
