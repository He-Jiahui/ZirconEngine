//! Static Hub UI guardrail contracts that scan every Slint file.

use std::{
    fs,
    path::{Path, PathBuf},
};

fn ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
}

fn normalize_newlines(source: String) -> String {
    source.replace("\r\n", "\n")
}

fn read_ui_file(name: &str) -> String {
    normalize_newlines(
        fs::read_to_string(ui_dir().join(name)).unwrap_or_else(|error| {
            panic!("failed to read Hub UI file {name}: {error}");
        }),
    )
}

fn slint_files() -> Vec<PathBuf> {
    let mut files = fs::read_dir(ui_dir())
        .expect("failed to read Hub UI directory")
        .map(|entry| entry.expect("failed to read Hub UI entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "slint")
        })
        .collect::<Vec<_>>();
    files.sort();
    files
}

#[test]
fn hub_ui_files_route_typography_through_material_wrappers() {
    let mut violations = Vec::new();

    for path in slint_files() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            if uses_raw_text_or_direct_font_binding(trimmed) {
                violations.push(format!(
                    "{}:{}: {}",
                    display_path(&path),
                    index + 1,
                    trimmed
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Hub UI files must route visible typography through MaterialText/shared wrappers instead of raw Text/font bindings:\n{}",
        violations.join("\n")
    );
}

#[test]
fn hub_ui_files_do_not_use_character_icon_literals() {
    let mut violations = Vec::new();

    for path in slint_files() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            if let Some(literal) = character_icon_literal(trimmed) {
                violations.push(format!(
                    "{}:{}: {} uses {literal:?}",
                    display_path(&path),
                    index + 1,
                    trimmed
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Hub UI controls must use SVG/Material icon slots instead of single-character text glyphs:\n{}",
        violations.join("\n")
    );
}

#[test]
fn hub_ui_direct_touch_area_is_reserved_for_window_dragging() {
    let mut violations = Vec::new();

    for path in slint_files() {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("<unknown>");
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || !trimmed.contains("TouchArea") {
                continue;
            }
            if file_name == "shell_header_components.slint" && trimmed == "drag-area := TouchArea {"
            {
                continue;
            }
            violations.push(format!(
                "{}:{}: {}",
                display_path(&path),
                index + 1,
                trimmed
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "Hub UI interaction surfaces must use Material controls/ListTile/StateLayerArea; direct TouchArea is reserved for shell window dragging:\n{}",
        violations.join("\n")
    );
}

#[test]
fn input_and_navigation_state_owners_do_not_bypass_shared_primitives() {
    let mut violations = Vec::new();
    let allowed_state_owner_files = [
        "inputs.slint",
        "navigation.slint",
        "shared.slint",
        "shell_sidebar_components.slint",
        "app.slint",
        "project_browser_components.slint",
        "project_dashboard_components.slint",
        "settings_page_components.slint",
        "shell_header_components.slint",
    ];

    for path in slint_files() {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for forbidden in [
            "NavigationRail as MaterialNavigationRail",
            "material-field := TextField",
            "trigger := OutlineButton",
            "material-segment := SegmentedButton",
            "FilledIconButton {",
            "OutlineIconButton {",
            "MaterialIconButton {",
        ] {
            if source.contains(forbidden) && !allowed_state_owner_files.contains(&file_name) {
                violations.push(format!("{}: {forbidden}", display_path(&path)));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Hub input/navigation primitive state must stay in shared owner wrappers instead of page-local Material bypasses:\n{}",
        violations.join("\n")
    );
}

#[test]
fn hub_ui_files_do_not_use_percentage_sizing() {
    let mut violations = Vec::new();

    for path in slint_files() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            if percentage_size_binding(trimmed) {
                violations.push(format!(
                    "{}:{}: {}",
                    display_path(&path),
                    index + 1,
                    trimmed
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Hub UI sizing should use tokens, stretch, or explicit parent/content-width contracts instead of percent-based layout bindings:\n{}",
        violations.join("\n")
    );
}

#[test]
fn page_content_width_arithmetic_stays_in_layout_primitive() {
    let mut violations = Vec::new();

    for path in slint_files() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            if trimmed
                == "out property <length> content-width: max(1px, root.width - root.page-padding-x * 2);"
            {
                continue;
            }
            if path.file_name().and_then(|name| name.to_str())
                == Some("shell_header_popup_components.slint")
                && trimmed == "x: root.width - root.popup-width;"
            {
                continue;
            }
            if trimmed.contains("root.width -")
                || trimmed.contains("root.width /")
                || trimmed.contains("root.width *")
            {
                violations.push(format!(
                    "{}:{}: {}",
                    display_path(&path),
                    index + 1,
                    trimmed
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Hub pages and chrome should consume PageScrollSurface/content-width, stretch, or tokens instead of page-local root.width arithmetic:\n{}",
        violations.join("\n")
    );
}

#[test]
fn page_scroll_surface_is_owned_by_page_roots() {
    let mut violations = Vec::new();

    for path in slint_files() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            if trimmed.contains("page-surface := PageScrollSurface")
                || trimmed.contains("content-width: page-surface.content-width")
                || trimmed.contains("content-height: page-surface.content-height")
                || trimmed.contains("page-surface.content-height")
                || trimmed.contains("page-surface.viewport-height")
                || trimmed.contains("scroll-y <=> root.scroll-y;")
            {
                violations.push(format!(
                    "{}:{}: {}",
                    display_path(&path),
                    index + 1,
                    trimmed
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Hub pages should inherit PageScrollSurface or CatalogPage directly instead of nesting page-surface wrappers:\n{}",
        violations.join("\n")
    );
}

#[test]
fn page_compact_breakpoints_use_design_tokens() {
    let mut violations = Vec::new();
    let layout = read_ui_file("layout.slint");
    for snippet in [
        "export component ResponsiveState",
        "export component ResponsiveCollapse",
        "out property <bool> compact: root.viewport-width < HubTokens.breakpoint-compact;",
        "out property <bool> medium: root.viewport-width < HubTokens.breakpoint-medium;",
        "out property <bool> wide: root.viewport-width >= HubTokens.breakpoint-wide;",
        "out property <bool> short: root.viewport-height < HubTokens.breakpoint-short;",
        "out property <bool> collapsed: root.content-width < root.collapse-at;",
    ] {
        assert!(
            layout.contains(snippet),
            "layout.slint must centralize viewport breakpoints in ResponsiveState; missing {snippet}"
        );
    }

    for path in slint_files() {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            if file_name == "layout.slint"
                && (trimmed
                    == "out property <bool> compact: root.viewport-width < HubTokens.breakpoint-compact;"
                    || trimmed
                        == "out property <bool> medium: root.viewport-width < HubTokens.breakpoint-medium;"
                    || trimmed
                        == "out property <bool> wide: root.viewport-width >= HubTokens.breakpoint-wide;"
                    || trimmed
                        == "out property <bool> short: root.viewport-height < HubTokens.breakpoint-short;"
                    || trimmed
                        == "in property <length> page-padding: root.width < HubTokens.breakpoint-compact ? HubTokens.page-padding-compact : HubTokens.page-padding;")
                || trimmed == "out property <bool> collapsed: root.content-width < root.collapse-at;"
            {
                continue;
            }
            if trimmed.contains("root.width <")
                || trimmed.contains("root.height <")
                || trimmed.contains("root.viewport-width <")
                || trimmed.contains("root.viewport-width >=")
                || trimmed.contains("root.viewport-height <")
            {
                violations.push(format!("{}:{}: {trimmed}", display_path(&path), index + 1));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Hub UI viewport breakpoints must be centralized in layout.slint ResponsiveState:\n{}",
        violations.join("\n")
    );
}

#[test]
fn absolute_positioning_stays_out_of_page_layouts() {
    let mut violations = Vec::new();
    for path in slint_files() {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if matches!(
            file_name,
            "app.slint"
                | "inputs.slint"
                | "shell_header_components.slint"
                | "shell_header_popup_components.slint"
        ) {
            continue;
        }

        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("x:") || trimmed.starts_with("y:") {
                violations.push(format!(
                    "{}:{}: {}",
                    display_path(&path),
                    index + 1,
                    trimmed.trim()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "business pages should use layouts; only shell/input popup anchors may use x/y:\n{}",
        violations.join("\n")
    );
}

#[test]
fn hub_ui_layout_sizes_are_tokenized() {
    let mut violations = Vec::new();

    for path in slint_files() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            for value in raw_px_literals(trimmed) {
                if value > 1.0 {
                    violations.push(format!(
                        "{}:{}: {}",
                        display_path(&path),
                        index + 1,
                        trimmed
                    ));
                    break;
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Hub UI should derive layout sizes from MaterialStyleMetrics/HubTokens instead of raw px literals above 1px:\n{}",
        violations.join("\n")
    );
}

fn uses_raw_text_or_direct_font_binding(line: &str) -> bool {
    line == "Text {"
        || line.ends_with(": Text {")
        || line.ends_with(":= Text {")
        || line.contains("inherits Text")
        || line.contains("font-size:")
        || line.contains("font-weight:")
        || line.contains("font_size:")
        || line.contains("font_weight:")
}

fn character_icon_literal(line: &str) -> Option<&str> {
    for property in ["text:", "fallback-text:"] {
        if let Some(value) = line.strip_prefix(property) {
            let literal = value.trim().trim_end_matches(';').trim();
            if let Some(unquoted) = literal
                .strip_prefix('"')
                .and_then(|inner| inner.strip_suffix('"'))
            {
                if matches!(
                    unquoted,
                    "+" | ">" | "<" | "[]" | "::" | "==" | "v" | "!" | "?" | "..."
                ) {
                    return Some(unquoted);
                }
            }
        }
    }
    None
}

fn percentage_size_binding(line: &str) -> bool {
    [
        "width:",
        "height:",
        "min-width:",
        "min-height:",
        "max-width:",
        "max-height:",
        "preferred-width:",
        "preferred-height:",
    ]
    .iter()
    .any(|property| line.starts_with(property) && line.contains('%'))
}

fn raw_px_literals(line: &str) -> Vec<f32> {
    let mut values = Vec::new();

    for (unit_index, _) in line.match_indices("px") {
        let prefix = &line[..unit_index];
        let mut start = prefix.len();
        while start > 0 {
            let byte = prefix.as_bytes()[start - 1];
            if byte.is_ascii_digit() || byte == b'.' {
                start -= 1;
            } else {
                break;
            }
        }
        if start == prefix.len() {
            continue;
        }

        let literal = &prefix[start..];
        if let Ok(value) = literal.parse::<f32>() {
            values.push(value);
        }
    }

    values
}

fn display_path(path: &Path) -> String {
    path.strip_prefix(PathBuf::from(env!("CARGO_MANIFEST_DIR")))
        .unwrap_or(path)
        .display()
        .to_string()
}
