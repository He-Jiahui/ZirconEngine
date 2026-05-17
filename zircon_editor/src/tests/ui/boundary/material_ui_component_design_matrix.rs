use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

const DESIGN_MATRIX: &str =
    include_str!("../../../../../docs/ui-and-layout/material-ui-component-design-matrix.md");

const REQUIRED_INTERACTIVE_ROWS: &[&str] = &[
    "accordion",
    "alert",
    "autocomplete",
    "backdrop",
    "bottom-navigation",
    "breadcrumbs",
    "button-group",
    "buttons",
    "cards",
    "checkboxes",
    "chips",
    "dialogs",
    "drawers",
    "floating-action-button",
    "links",
    "lists",
    "menubar",
    "menus",
    "number-field",
    "pagination",
    "popover",
    "radio-buttons",
    "rating",
    "selects",
    "slider",
    "snackbars",
    "speed-dial",
    "steppers",
    "switches",
    "table",
    "tabs",
    "text-fields",
    "textarea-autosize",
    "toggle-button",
    "tooltips",
    "transfer-list",
];

const REQUIRED_UTILITY_ROWS: &[&str] = &[
    "about-the-lab",
    "box",
    "click-away-listener",
    "container",
    "css-baseline",
    "grid",
    "init-color-scheme-script",
    "modal",
    "no-ssr",
    "portal",
    "stack",
    "transitions",
    "use-media-query",
];

const REQUIRED_NEEDS_SUPPORT_ROWS: &[&str] = &["masonry", "popper"];

const REQUIRED_MUI_X_ROWS: &[&str] = &[
    "Tree View",
    "Data Grid",
    "Charts",
    "Line Chart",
    "Bar Chart",
    "Pie Chart",
    "Sparkline",
    "Gauge",
    "Date and Time Pickers",
    "Chat / AgentChat",
    "Chat Composer",
];

#[test]
fn material_ui_component_design_matrix_covers_public_mui_docs_inventory() {
    let docs_inventory = collect_mui_docs_components();
    assert!(
        docs_inventory.len() >= 60,
        "MUI public docs inventory unexpectedly small; check dev/material-ui checkout"
    );

    for component_key in &docs_inventory {
        assert!(
            matrix_row(component_key).is_some(),
            "Material design matrix must include public MUI docs component `{component_key}`"
        );
    }

    let matrix_keys = collect_matrix_keys();
    assert_eq!(
        docs_inventory, matrix_keys,
        "Material design matrix rows must match dev/material-ui/docs/data/material/components exactly"
    );
}

#[test]
fn material_ui_component_design_matrix_freezes_state_and_validation_contracts() {
    for key in REQUIRED_INTERACTIVE_ROWS {
        let row = matrix_row(key).unwrap_or_else(|| panic!("missing interactive row `{key}`"));
        assert!(
            row.contains("visual + interaction"),
            "`{key}` must require visual + interaction validation"
        );
        assert!(
            row.contains("click")
                || row.contains("toggle")
                || row.contains("select")
                || row.contains("open")
                || row.contains("drag")
                || row.contains("focus"),
            "`{key}` must name an interaction feedback path"
        );
    }

    for key in REQUIRED_UTILITY_ROWS {
        let row = matrix_row(key).unwrap_or_else(|| panic!("missing utility row `{key}`"));
        assert!(
            row.contains("behavior/utility"),
            "`{key}` must be classified as behavior/utility instead of a forced visual control"
        );
    }

    for key in REQUIRED_NEEDS_SUPPORT_ROWS {
        let row = matrix_row(key).unwrap_or_else(|| panic!("missing needs-support row `{key}`"));
        assert!(
            row.contains("needs support"),
            "`{key}` must keep missing Zircon capability explicit"
        );
    }

    for token in [
        "button_variant",
        "surface_variant",
        "validation_level",
        "hovered",
        "pressed",
        "focused",
        "selected",
        "checked",
        "disabled",
        "popup_open",
        "corner_radius",
        "border_width",
    ] {
        assert!(
            DESIGN_MATRIX.contains(token),
            "shared Zircon interface field `{token}` must stay documented"
        );
    }
}

#[test]
fn material_ui_component_design_matrix_defines_zui_prototype_contract_and_mui_x_rows() {
    for heading in [
        "Detailed Prototype Contract",
        "MUI X Extension Matrix",
        "Response mechanism",
        "响应机制",
        "Appearance variants",
        "外观变体",
        "Layout modes",
        "布局模式",
        "`.zui` mapping",
        "Validation",
    ] {
        assert!(
            DESIGN_MATRIX.contains(heading),
            "design matrix should document `{heading}`"
        );
    }

    for group in [
        "Inputs",
        "Data Display",
        "Feedback",
        "Surfaces",
        "Navigation",
        "Layout",
    ] {
        assert!(
            DESIGN_MATRIX.contains(group),
            "design matrix should include the `{group}` prototype group"
        );
    }

    for row in REQUIRED_MUI_X_ROWS {
        assert!(
            DESIGN_MATRIX.contains(row),
            "design matrix should include MUI X row `{row}`"
        );
    }

    let date_time_pickers_row = mui_x_row("Date and Time Pickers")
        .expect("design matrix should include the MUI X Date and Time Pickers row");
    for token in [
        "<https://mui.com/x/react-date-pickers/>",
        "dev/material-ui/packages/mui-lab/src/*Date*Picker*",
        "dev/material-ui/packages/mui-lab/src/*Time*Picker*",
        "DateTimePickers",
        "material_mui_x_date_time_pickers.zui",
    ] {
        assert!(
            date_time_pickers_row.contains(token),
            "Date and Time Pickers row should document `{token}`"
        );
    }

    for prototype in [
        "material_buttons.zui",
        "material_text_fields.zui",
        "material_selects.zui",
        "material_table.zui",
        "material_dialogs.zui",
        "material_drawers.zui",
        "material_tabs.zui",
        "material_mui_x_data_grid.zui",
        "material_mui_x_date_time_pickers.zui",
        "material_mui_x_chat_composer.zui",
    ] {
        assert!(
            DESIGN_MATRIX.contains(prototype),
            "design matrix should name prototype mapping `{prototype}`"
        );
    }
}

#[test]
fn material_ui_component_design_matrix_names_existing_zui_prototypes() {
    let references = collect_material_zui_references();
    assert!(
        references.len() >= 60,
        "design matrix should name the concrete Material prototype files, found only {}",
        references.len()
    );

    let prototype_dir = workspace_root().join("zircon_editor/assets/ui/editor/material_components");
    let prototype_files = fs::read_dir(&prototype_dir)
        .unwrap_or_else(|error| {
            panic!(
                "Material prototype directory is readable at {}: {error}",
                prototype_dir.display()
            )
        })
        .map(|entry| {
            entry
                .expect("prototype dir entry is readable")
                .file_name()
                .to_string_lossy()
                .to_string()
        })
        .collect::<BTreeSet<_>>();

    for reference in references {
        assert!(
            prototype_files.contains(&reference),
            "design matrix references `{reference}`, but no matching Material prototype file exists"
        );
    }
}

fn collect_mui_docs_components() -> BTreeSet<String> {
    let root = workspace_root().join("dev/material-ui/docs/data/material/components");
    let mut components = BTreeSet::new();
    for entry in fs::read_dir(&root).unwrap_or_else(|error| {
        panic!(
            "MUI docs component inventory is readable at {}: {error}",
            root.display()
        )
    }) {
        let path = entry.expect("component dir entry is readable").path();
        if path.is_dir() {
            let key = path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("component dir name is UTF-8")
                .to_string();
            components.insert(key);
        }
    }
    components
}

fn collect_material_zui_references() -> BTreeSet<String> {
    DESIGN_MATRIX
        .split('`')
        .filter(|segment| {
            segment.starts_with("material_")
                && segment.ends_with(".zui")
                && !segment.contains("<component>")
                && !segment.contains('*')
        })
        .map(str::to_string)
        .collect()
}

fn collect_matrix_keys() -> BTreeSet<String> {
    DESIGN_MATRIX
        .lines()
        .filter_map(|line| {
            line.strip_prefix("| `")
                .and_then(|rest| rest.split_once("` |"))
                .map(|(key, _)| key.to_string())
        })
        .filter(|key| key != "MUI key")
        .collect()
}

fn matrix_row(component_key: &str) -> Option<&'static str> {
    let prefix = format!("| `{component_key}` |");
    DESIGN_MATRIX.lines().find(|line| line.starts_with(&prefix))
}

fn mui_x_row(component_name: &str) -> Option<&'static str> {
    let prefix = format!("| {component_name} |");
    DESIGN_MATRIX.lines().find(|line| line.starts_with(&prefix))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_editor lives directly under the workspace root")
        .to_path_buf()
}
