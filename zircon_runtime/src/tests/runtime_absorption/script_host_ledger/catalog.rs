pub(super) const FIXED_HOST_FUNCTIONS: &[(&str, &str)] = &[
    ("zr.zircon.foundation", "time_unix_millis"),
    ("zr.zircon.foundation", "log_info"),
    ("zr.zircon.foundation", "event_publish"),
    ("zr.zircon.asset", "locator_identity"),
    ("zr.zircon.asset", "status"),
    ("zr.zircon.asset", "revision"),
    ("zr.zircon.scene", "default_world_handle"),
    ("zr.zircon.scene", "handle_is_valid"),
    ("zr.zircon.scene", "summary"),
    ("zr.zircon.render", "backend_name"),
    ("zr.zircon.render", "frame_index"),
    ("zr.zircon.math", "vec3_length"),
    ("zr.zircon.math", "vec3_dot"),
    ("zr.zircon.gameplay", "delta_seconds"),
    ("zr.zircon.gameplay", "entity"),
    ("zr.zircon.gameplay", "key_pressed"),
    ("zr.zircon.gameplay", "position_json"),
    ("zr.zircon.gameplay", "position_x"),
    ("zr.zircon.gameplay", "position_y"),
    ("zr.zircon.gameplay", "position_z"),
    ("zr.zircon.gameplay", "set_position_json"),
    ("zr.zircon.gameplay", "set_position"),
    ("zr.zircon.gameplay", "translate_json"),
    ("zr.zircon.gameplay", "translate"),
    ("zr.zircon.gameplay", "face_direction"),
    ("zr.zircon.gameplay", "set_scale"),
    ("zr.zircon.gameplay", "follow_position"),
    ("zr.zircon.gameplay", "move_towards_entity"),
    ("zr.zircon.gameplay", "camera_follow"),
    ("zr.zircon.gameplay", "component_json"),
    ("zr.zircon.gameplay", "component_string"),
    ("zr.zircon.gameplay", "set_component_json"),
    ("zr.zircon.gameplay", "find_by_component"),
    ("zr.zircon.gameplay", "entity_exists"),
    ("zr.zircon.gameplay", "nearest_by_script_property"),
    ("zr.zircon.gameplay", "count_by_script_property"),
    ("zr.zircon.gameplay", "script_property_matches"),
    ("zr.zircon.gameplay", "script_number"),
    ("zr.zircon.gameplay", "script_number_at_most"),
    ("zr.zircon.gameplay", "set_animation_bool"),
    ("zr.zircon.gameplay", "damage_entity"),
    ("zr.zircon.gameplay", "heal_entity"),
    ("zr.zircon.gameplay", "current_hp"),
    ("zr.zircon.gameplay", "damage_entity_report"),
    ("zr.zircon.gameplay", "spawn_empty"),
    ("zr.zircon.gameplay", "spawn_model"),
    ("zr.zircon.gameplay", "set_hud_text"),
    ("zr.zircon.gameplay", "set_particle_sprites"),
    ("zr.zircon.gameplay", "set_world_hud_bar"),
    ("zr.zircon.gameplay", "despawn"),
    ("zr.zircon.gameplay", "nav_next_point_json"),
    ("zr.zircon.gameplay", "nav_move_towards_entity"),
];

pub(super) const FIXED_HOST_MODULES: &[&str] = &[
    "zr.zircon.foundation",
    "zr.zircon.asset",
    "zr.zircon.scene",
    "zr.zircon.render",
    "zr.zircon.math",
    "zr.zircon.gameplay",
];

pub(super) const HOST_CAPABILITIES: &[&str] = &[
    "foundation.log",
    "foundation.time",
    "foundation.event",
    "asset.query",
    "scene.query",
    "scene.handle",
    "render.query",
    "gameplay.input",
    "gameplay.entity",
    "gameplay.navigation",
    "bridge.call",
];

pub(super) fn fixed_sources_contain_function(
    builtin_source: &str,
    gameplay_source: &str,
    function: &str,
) -> bool {
    builtin_source.contains(&format!("HostExportFunction::new(\"{function}\""))
        || gameplay_source.contains(&format!("HostExportFunction::new(\"{function}\""))
        || builtin_source.contains(&format!("name = \"{function}\""))
}

pub(super) fn combined_fixed_sources_contain_module(
    builtin_source: &str,
    gameplay_source: &str,
    module: &str,
) -> bool {
    builtin_source.contains(module) || gameplay_source.contains(module)
}

pub(super) fn missing_documented_functions(ledger: &str) -> Vec<String> {
    FIXED_HOST_FUNCTIONS
        .iter()
        .filter_map(|(module, function)| {
            let documented = ledger.contains(&format!("| `{function}` |"))
                || ledger.contains(&format!("| Type `{function}` |"));
            (!documented).then(|| format!("{module}.{function}"))
        })
        .collect()
}

pub(super) fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}
