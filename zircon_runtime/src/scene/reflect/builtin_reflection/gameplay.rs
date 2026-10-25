//! Runtime-owned JSON component identities; payload access stays with the dynamic component APIs.

use zircon_runtime_interface::reflect::{
    ReflectError, ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath,
    ReflectTypeRegistration,
};

use crate::scene::{RuntimeTypeRegistration, TypeRegistry};

pub(super) fn register(registry: &mut TypeRegistry) -> Result<(), ReflectError> {
    // Keep this list explicit: a gameplay namespace prefix must not admit unknown project ids.
    for (type_path, name, display_name) in [
        ("gameplay.hud_text", "hud_text", "Gameplay HUD text"),
        ("gameplay.menu_state", "menu_state", "Gameplay menu state"),
        (
            "gameplay.control_state",
            "control_state",
            "Gameplay control state",
        ),
        (
            "gameplay.particle_sprites",
            "particle_sprites",
            "Gameplay particle sprites",
        ),
        (
            "gameplay.world_hud_bars",
            "world_hud_bars",
            "Gameplay world HUD bars",
        ),
    ] {
        registry.register(RuntimeTypeRegistration::metadata(
            ReflectTypeRegistration::new(
                ReflectTypePath::new(type_path, name)?,
                display_name,
                ReflectTypeInfo::json_with_fields(Vec::new()),
                ReflectSerializationStrategy::Json,
            )
            .as_component()
            .with_editor_visible(false),
        ))?;
    }
    Ok(())
}
