use std::{fs, path::PathBuf};

use toml::Value;

const MIGRATION_DOC: &str = "docs/ui-and-layout/slint-material-retained-editor-migration.md";
const MIGRATION_SPEC: &str =
    "docs/superpowers/specs/2026-05-20-slint-material-retained-editor-migration-design.md";
const MIGRATION_PLAN: &str =
    "docs/superpowers/plans/2026-05-20-slint-material-retained-editor-migration.md";
const MATERIAL_EXPORTS: &str = "dev/material-rust-template/material-1.0/material.slint";

const REQUIRED_SOURCE_REFERENCES: &[&str] = &[
    "dev/material-rust-template/material-1.0/material.slint",
    "dev/material-rust-template/material-1.0/ui/styling/material_palette.slint",
    "dev/material-rust-template/material-1.0/ui/styling/material_schemes.slint",
    "dev/material-rust-template/material-1.0/ui/styling/material_style_metrics.slint",
    "dev/material-rust-template/material-1.0/ui/styling/material_typography.slint",
    "dev/material-rust-template/material-1.0/ui/styling/material_animations.slint",
    "dev/material-rust-template/material-1.0/ui/components/state_layer.slint",
    "dev/material-rust-template/material-1.0/ui/components/elevation.slint",
    "dev/material-rust-template/material-1.0/ui/components/base_button.slint",
    "dev/material-rust-template/material-1.0/ui/components/text_field.slint",
    "dev/material-rust-template/material-1.0/ui/components/check_box.slint",
    "dev/material-rust-template/material-1.0/ui/components/radio_button.slint",
    "dev/material-rust-template/material-1.0/ui/components/switch.slint",
    "dev/material-rust-template/material-1.0/ui/components/slider.slint",
    "dev/material-rust-template/material-1.0/ui/components/menu.slint",
    "dev/material-rust-template/material-1.0/ui/components/dialog.slint",
    "dev/material-rust-template/material-1.0/ui/components/drawer.slint",
    "dev/material-rust-template/material-1.0/ui/components/navigation_bar.slint",
    "dev/material-rust-template/material-1.0/ui/components/tab_bar.slint",
];

const REQUIRED_FOUNDATION_TOKENS: &[&str] = &[
    "slint_material_primary",
    "slint_material_on_primary",
    "slint_material_primary_container",
    "slint_material_on_primary_container",
    "slint_material_secondary",
    "slint_material_tertiary",
    "slint_material_error",
    "slint_material_surface",
    "slint_material_on_surface",
    "slint_material_surface_container_lowest",
    "slint_material_surface_container_low",
    "slint_material_surface_container",
    "slint_material_surface_container_high",
    "slint_material_surface_container_highest",
    "slint_material_outline",
    "slint_material_shadow",
    "slint_material_scrim",
    "slint_material_inverse_surface",
    "slint_material_primary_fixed",
    "slint_material_shadow_15",
    "slint_material_shadow_30",
    "slint_material_background_modal",
    "slint_material_state_layer_opacity_hover",
    "slint_material_state_layer_opacity_focus",
    "slint_material_state_layer_opacity_press",
    "slint_material_state_layer_opacity_disabled",
    "slint_material_state_layer_opacity_drag",
    "slint_material_disabled_opacity",
    "slint_material_size_32",
    "slint_material_size_90",
    "slint_material_icon_size_24",
    "slint_material_icon_size_90",
    "slint_material_padding_16",
    "slint_material_spacing_8",
    "slint_material_border_radius_28",
    "slint_material_typography_regular_weight",
    "slint_material_typography_medium_weight",
    "slint_material_typography_semibold_weight",
    "slint_material_typography_display_large_size",
    "slint_material_typography_headline_large_size",
    "slint_material_typography_title_medium_size",
    "slint_material_typography_label_medium_prominent_weight",
    "slint_material_typography_body_small_size",
    "slint_material_emphasized_easing",
    "slint_material_emphasized_duration_ms",
    "slint_material_standard_easing",
    "slint_material_standard_fast_duration_ms",
    "slint_material_ripple_easing",
    "slint_material_ripple_duration_ms",
    "slint_material_opacity_easing",
    "slint_material_opacity_duration_ms",
    "slint_material_elevation_level1_light_outer_offset_y",
    "slint_material_elevation_level1_light_outer_blur",
    "slint_material_elevation_level2_light_inner_blur",
    "slint_material_elevation_level5_light_inner_offset_y",
    "slint_material_elevation_level1_dark_outer_blur",
    "slint_material_elevation_level2_dark_outer_offset_y",
    "slint_material_elevation_level5_dark_inner_blur",
];

const REQUIRED_M2_METADATA_TOKENS: &[&str] = &[
    "slint_material_state_layer_priority",
    "slint_material_state_layer_disabled_uses_focus_opacity",
    "slint_material_hover_disable_token",
    "slint_material_ripple_pressed_x_attr",
    "slint_material_ripple_pressed_y_attr",
    "slint_material_ripple_clip_attr",
    "slint_material_ripple_static_pulse_contract",
    "slint_material_elevation_retained_painter_contract",
];

const EXPECTED_STRING_TOKENS: &[(&str, &str)] = &[
    ("slint_material_scheme", "dark"),
    ("slint_material_primary", "#adc6ff"),
    ("slint_material_on_primary", "#112f60"),
    ("slint_material_surface", "#111318"),
    ("slint_material_surface_container_highest", "#33353a"),
    ("slint_material_outline", "#8e9099"),
    ("slint_material_shadow_15", "#00000026"),
    ("slint_material_shadow_30", "#0000004d"),
    ("slint_material_background_modal", "#00000080"),
    (
        "slint_material_emphasized_easing",
        "cubic-bezier(0.05, 0.7, 0.1, 1.0)",
    ),
    ("slint_material_standard_easing", "cubic-bezier(0, 0, 0, 1)"),
    ("slint_material_ripple_easing", "ease_out"),
    ("slint_material_opacity_easing", "ease"),
];

const EXPECTED_NUMBER_TOKENS: &[(&str, f64)] = &[
    ("slint_material_state_layer_opacity_hover", 0.08),
    ("slint_material_state_layer_opacity_focus", 0.10),
    ("slint_material_state_layer_opacity_press", 0.10),
    ("slint_material_state_layer_opacity_disabled", 0.12),
    ("slint_material_state_layer_opacity_drag", 0.16),
    ("slint_material_disabled_opacity", 0.38),
    ("slint_material_size_32", 32.0),
    ("slint_material_size_90", 96.0),
    ("slint_material_icon_size_90", 27.0),
    ("slint_material_padding_16", 16.0),
    ("slint_material_spacing_8", 8.0),
    ("slint_material_border_radius_28", 28.0),
    ("slint_material_typography_regular_weight", 300.0),
    ("slint_material_typography_medium_weight", 600.0),
    ("slint_material_typography_semibold_weight", 900.0),
    ("slint_material_typography_display_large_size", 57.0),
    ("slint_material_typography_title_medium_size", 16.0),
    ("slint_material_typography_body_small_size", 12.0),
    ("slint_material_emphasized_duration_ms", 500.0),
    ("slint_material_standard_fast_duration_ms", 150.0),
    ("slint_material_ripple_duration_ms", 2000.0),
    ("slint_material_opacity_duration_ms", 250.0),
    ("slint_material_elevation_level1_light_outer_offset_y", 1.0),
    ("slint_material_elevation_level2_light_inner_blur", 6.0),
    ("slint_material_elevation_level5_light_inner_offset_y", 4.0),
    ("slint_material_elevation_level1_dark_outer_blur", 3.0),
    ("slint_material_elevation_level2_dark_outer_offset_y", 2.0),
    ("slint_material_elevation_level5_dark_inner_blur", 4.0),
];

const EXPECTED_M2_STRING_TOKENS: &[(&str, &str)] = &[
    (
        "slint_material_state_layer_priority",
        "disabled>focus>pressed>drag>hover>default",
    ),
    (
        "slint_material_state_layer_disabled_uses_focus_opacity",
        "true",
    ),
    ("slint_material_hover_disable_token", "disable_hover"),
    ("slint_material_ripple_pressed_x_attr", "pressed_x"),
    ("slint_material_ripple_pressed_y_attr", "pressed_y"),
    ("slint_material_ripple_clip_attr", "clip_ripple"),
    (
        "slint_material_ripple_static_pulse_contract",
        "pressed_or_enter_pressed_static_circle",
    ),
    (
        "slint_material_elevation_retained_painter_contract",
        "retained_template_elevation_shadow",
    ),
];

#[test]
fn slint_material_export_mapping_covers_every_material_export() {
    let material_exports = workspace_file(MATERIAL_EXPORTS);
    let migration_doc = workspace_file(MIGRATION_DOC);
    let exports = collect_material_exports(&material_exports);

    assert!(
        exports.len() >= 70,
        "material.slint export inventory unexpectedly small: {}",
        exports.len()
    );

    let missing_exports = exports
        .iter()
        .filter(|export| !migration_doc.contains(&format!("`{export}`")))
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        missing_exports.is_empty(),
        "Slint Material exports missing from retained migration mapping: {}",
        missing_exports.join(", ")
    );
}

#[test]
fn retained_editor_migration_docs_pin_sources_and_no_direct_slint_fence() {
    let migration_doc = workspace_file(MIGRATION_DOC);
    let migration_spec = workspace_file(MIGRATION_SPEC);
    let migration_plan = workspace_file(MIGRATION_PLAN);
    let editor_manifest = editor_file("Cargo.toml");
    let mut failures = Vec::new();

    for source in REQUIRED_SOURCE_REFERENCES {
        if !migration_doc.contains(source) {
            failures.push(format!("migration doc must reference `{source}`"));
        }
        if !migration_spec.contains(source) {
            failures.push(format!("migration spec must reference `{source}`"));
        }
    }

    for document in [&migration_doc, &migration_spec, &migration_plan] {
        if !document.contains("retained") {
            failures
                .push("migration control documents must name retained UI ownership".to_string());
        }
        if !names_no_direct_slint_fence(document) {
            failures
                .push("migration control documents must name the direct Slint fence".to_string());
        }
    }

    let former_generated_build_dependency = ["sli", "nt-build"].concat();
    for forbidden in [
        "slint",
        former_generated_build_dependency.as_str(),
        "i-slint",
    ] {
        if editor_manifest.contains(forbidden) {
            failures.push(format!(
                "zircon_editor/Cargo.toml must not declare `{forbidden}`"
            ));
        }
    }

    for relative in collect_editor_ui_files() {
        let source = fs::read_to_string(&relative)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", relative.display()));
        if source.contains("@material") || source.contains("material.slint") {
            failures.push(format!(
                "retained Editor UI asset must not import Slint Material: {}",
                relative.display()
            ));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn editor_material_theme_declares_source_aligned_slint_material_foundation_tokens() {
    let document = editor_toml("assets/ui/theme/editor_material.v2.ui.toml");
    let tokens = document
        .get("tokens")
        .and_then(Value::as_table)
        .expect("editor material theme declares [tokens]");
    let mut failures = Vec::new();

    for token in REQUIRED_FOUNDATION_TOKENS {
        if !tokens.contains_key(*token) {
            failures.push(format!("missing Slint Material foundation token `{token}`"));
        }
    }

    for (token, expected) in EXPECTED_STRING_TOKENS {
        match tokens.get(*token).and_then(Value::as_str) {
            Some(value) if value == *expected => {}
            Some(value) => failures.push(format!(
                "`{token}` must be `{expected}` from Slint Material, got `{value}`"
            )),
            None => failures.push(format!("`{token}` must be a string token")),
        }
    }

    for (token, expected) in EXPECTED_NUMBER_TOKENS {
        match tokens.get(*token).and_then(value_as_number) {
            Some(value) if (value - expected).abs() < f64::EPSILON => {}
            Some(value) => failures.push(format!(
                "`{token}` must be {expected} from Slint Material, got {value}"
            )),
            None => failures.push(format!("`{token}` must be a numeric token")),
        }
    }

    for token in [
        "material_density_compact_height",
        "material_icon_size_button",
        "material_font_size_body",
    ] {
        if !tokens.contains_key(token) {
            failures.push(format!(
                "existing compact Editor token `{token}` must remain while adding Slint tokens"
            ));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn editor_material_theme_declares_m2_behavior_metadata_contracts() {
    let document = editor_toml("assets/ui/theme/editor_material.v2.ui.toml");
    let tokens = document
        .get("tokens")
        .and_then(Value::as_table)
        .expect("editor material theme declares [tokens]");
    let migration_doc = workspace_file(MIGRATION_DOC);
    let mut failures = Vec::new();

    for token in REQUIRED_M2_METADATA_TOKENS {
        if !tokens.contains_key(*token) {
            failures.push(format!("missing M2 metadata token `{token}`"));
        }
        if !migration_doc.contains(token) {
            failures.push(format!("M2 metadata token `{token}` must be documented"));
        }
    }

    for (token, expected) in EXPECTED_M2_STRING_TOKENS {
        match tokens.get(*token).and_then(Value::as_str) {
            Some(value) if value == *expected => {}
            Some(value) => failures.push(format!(
                "`{token}` must be `{expected}` from Slint M2 behavior, got `{value}`"
            )),
            None => failures.push(format!("`{token}` must be a string token")),
        }
    }

    for source_symbol in [
        "root.state_layer_opacity: MaterialPalette.state_layer_opacity_focus",
        "root.pressed || root.enter_pressed",
        "pressed_x: root.pressed_x",
        "clip_ripple: root.clip_ripple",
        "drop_shadow_offset_y: 8px",
    ] {
        if !migration_doc.contains(source_symbol) {
            failures.push(format!(
                "migration doc must preserve source behavior `{source_symbol}`"
            ));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn collect_material_exports(source: &str) -> Vec<String> {
    let mut exports = Vec::new();
    for line in source.lines() {
        let Some(exports_start) = line.find("export {") else {
            continue;
        };
        let Some(exports_end) = line[exports_start..].find('}') else {
            continue;
        };
        let names = &line[exports_start + "export {".len()..exports_start + exports_end];
        for name in names
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            exports.push(name.to_string());
        }
    }
    exports.sort();
    exports.dedup();
    exports
}

fn names_no_direct_slint_fence(document: &str) -> bool {
    let lower = document.to_ascii_lowercase();
    lower.contains("direct") && lower.contains("slint")
}

fn collect_editor_ui_files() -> Vec<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/ui");
    let mut files = Vec::new();
    collect_files_with_extension(root, "toml", &mut files);
    collect_files_with_extension(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/ui"),
        "zui",
        &mut files,
    );
    files.sort();
    files.dedup();
    files
}

fn collect_files_with_extension(root: PathBuf, extension: &str, files: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }
    for entry in fs::read_dir(&root)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", root.display()))
    {
        let path = entry
            .unwrap_or_else(|error| {
                panic!("entry under {} should be readable: {error}", root.display())
            })
            .path();
        if path.is_dir() {
            collect_files_with_extension(path, extension, files);
        } else if path.extension().and_then(|value| value.to_str()) == Some(extension) {
            files.push(path);
        }
    }
}

fn editor_toml(relative: &str) -> Value {
    toml::from_str(&editor_file(relative))
        .unwrap_or_else(|error| panic!("{relative} parses: {error}"))
}

fn editor_file(relative: &str) -> String {
    read_file(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative))
}

fn workspace_file(relative: &str) -> String {
    read_file(workspace_root().join(relative))
}

fn read_file(path: PathBuf) -> String {
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()))
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_editor lives directly under the workspace root")
        .to_path_buf()
}

fn value_as_number(value: &Value) -> Option<f64> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|integer| integer as f64))
}
