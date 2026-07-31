use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::v2::UiV2AssetDocument;

pub(super) struct MaterialPrototypeFixture {
    pub(super) path: PathBuf,
    pub(super) source: String,
    pub(super) document: UiV2AssetDocument,
}

static MATERIAL_PROTOTYPE_FILES: OnceLock<Vec<PathBuf>> = OnceLock::new();
static MATERIAL_PROTOTYPE_FIXTURES: OnceLock<BTreeMap<String, MaterialPrototypeFixture>> =
    OnceLock::new();
static EDITOR_MATERIAL_THEME_SOURCE: OnceLock<String> = OnceLock::new();
static EDITOR_MATERIAL_THEME_SELECTORS: OnceLock<BTreeSet<String>> = OnceLock::new();

pub(super) const MUI_X_PROTOTYPES: &[&str] = &[
    "mui_x_tree_view",
    "mui_x_data_grid",
    "mui_x_date_time_pickers",
    "mui_x_charts",
    "mui_x_line_chart",
    "mui_x_bar_chart",
    "mui_x_pie_chart",
    "mui_x_sparkline",
    "mui_x_gauge",
    "mui_x_agent_chat",
    "mui_x_chat_composer",
];

pub(super) const MATERIAL_COMPONENT_DOMAINS: &[&str] = &[
    "data_display",
    "feedback",
    "inputs",
    "layout",
    "mui_x",
    "navigation",
    "surfaces",
    "utils_lab",
];

pub(super) fn material_prototype_domain(key: &str) -> &'static str {
    match key {
        "avatars" | "badges" | "chips" | "dividers" | "icons" | "image_list" | "lists"
        | "material_icons" | "table" | "timeline" | "typography" => "data_display",
        "alert" | "backdrop" | "dialogs" | "modal" | "popover" | "popper" | "progress"
        | "skeleton" | "snackbars" | "speed_dial" | "tooltips" => "feedback",
        "autocomplete"
        | "button_group"
        | "buttons"
        | "checkboxes"
        | "floating_action_button"
        | "number_field"
        | "radio_buttons"
        | "rating"
        | "selects"
        | "slider"
        | "switches"
        | "text_fields"
        | "textarea_autosize"
        | "toggle_button" => "inputs",
        "box" | "container" | "grid" | "masonry" | "stack" => "layout",
        "mui_x_agent_chat"
        | "mui_x_bar_chart"
        | "mui_x_charts"
        | "mui_x_chat_composer"
        | "mui_x_data_grid"
        | "mui_x_date_time_pickers"
        | "mui_x_gauge"
        | "mui_x_line_chart"
        | "mui_x_pie_chart"
        | "mui_x_sparkline"
        | "mui_x_tree_view" => "mui_x",
        "bottom_navigation" | "breadcrumbs" | "links" | "menubar" | "menus" | "pagination"
        | "steppers" | "tabs" | "transfer_list" => "navigation",
        "accordion" | "app_bar" | "cards" | "drawers" | "paper" => "surfaces",
        "about_the_lab"
        | "click_away_listener"
        | "css_baseline"
        | "init_color_scheme_script"
        | "no_ssr"
        | "portal"
        | "transitions"
        | "use_media_query" => "utils_lab",
        _ => panic!("unknown Material component prototype key `{key}`"),
    }
}

pub(super) fn material_prototype_path(key: &str) -> PathBuf {
    editor_asset(&format!(
        "assets/ui/editor/material_components/{}/material_{key}.zui",
        material_prototype_domain(key)
    ))
}

pub(super) fn material_prototype_res_path(path: &Path, component_name: &str) -> String {
    let editor_root = editor_asset("assets/ui/editor");
    let relative = path
        .strip_prefix(&editor_root)
        .unwrap_or_else(|error| {
            panic!(
                "{} should live under {}: {error}",
                path.display(),
                editor_root.display()
            )
        })
        .to_string_lossy()
        .replace('\\', "/");
    format!("res://ui/editor/{relative}#{component_name}")
}

pub(super) const INTERACTIVE_PROTOTYPES: &[&str] = &[
    "accordion",
    "autocomplete",
    "backdrop",
    "bottom_navigation",
    "breadcrumbs",
    "buttons",
    "cards",
    "checkboxes",
    "chips",
    "dialogs",
    "drawers",
    "floating_action_button",
    "image_list",
    "links",
    "lists",
    "menubar",
    "menus",
    "modal",
    "number_field",
    "pagination",
    "popover",
    "popper",
    "radio_buttons",
    "rating",
    "selects",
    "slider",
    "snackbars",
    "speed_dial",
    "steppers",
    "switches",
    "table",
    "tabs",
    "text_fields",
    "textarea_autosize",
    "toggle_button",
    "tooltips",
    "transfer_list",
    "mui_x_tree_view",
    "mui_x_data_grid",
    "mui_x_date_time_pickers",
    "mui_x_charts",
    "mui_x_line_chart",
    "mui_x_bar_chart",
    "mui_x_pie_chart",
    "mui_x_sparkline",
    "mui_x_gauge",
    "mui_x_agent_chat",
    "mui_x_chat_composer",
];

pub(super) fn child_nodes<'a>(document: &'a UiV2AssetDocument, node_id: &str) -> Vec<&'a str> {
    document
        .nodes
        .get(node_id)
        .unwrap_or_else(|| panic!("Material Lab should contain node `{node_id}`"))
        .children
        .iter()
        .map(|child| child.node.as_str())
        .collect()
}

pub(super) fn assert_component(
    document: &UiV2AssetDocument,
    node_id: &str,
    expected_component: &str,
) {
    let component = document
        .nodes
        .get(node_id)
        .unwrap_or_else(|| panic!("Material Lab should contain node `{node_id}`"))
        .component
        .as_str();
    assert_eq!(
        component, expected_component,
        "Material Lab node `{node_id}` should keep its expected component"
    );
}

pub(super) fn assert_node_class(document: &UiV2AssetDocument, node_id: &str, expected_class: &str) {
    let node = document
        .nodes
        .get(node_id)
        .unwrap_or_else(|| panic!("Material Lab should contain node `{node_id}`"));
    assert!(
        node.classes.iter().any(|class| class == expected_class),
        "Material Lab node `{node_id}` should keep class `{expected_class}`"
    );
}

pub(super) fn numeric_prop(value: Option<&toml::Value>) -> Option<f64> {
    value.and_then(|value| {
        value
            .as_float()
            .or_else(|| value.as_integer().map(|integer| integer as f64))
    })
}

pub(super) fn material_prototype_files() -> &'static [PathBuf] {
    MATERIAL_PROTOTYPE_FILES
        .get_or_init(|| {
            let dir = editor_asset("assets/ui/editor/material_components");
            let mut files = Vec::new();
            for domain in MATERIAL_COMPONENT_DOMAINS {
                let domain_dir = dir.join(domain);
                files.extend(
                    fs::read_dir(&domain_dir)
                        .unwrap_or_else(|error| {
                            panic!("{} should be readable: {error}", domain_dir.display())
                        })
                        .map(|entry| {
                            entry
                                .expect("prototype dir entry should be readable")
                                .path()
                        })
                        .filter(|path| {
                            path.file_name()
                                .and_then(|name| name.to_str())
                                .is_some_and(|name| {
                                    name.starts_with("material_") && name.ends_with(".zui")
                                })
                        }),
                );
            }
            files.sort();
            files
        })
        .as_slice()
}

pub(super) fn material_prototype_fixtures() -> &'static BTreeMap<String, MaterialPrototypeFixture> {
    MATERIAL_PROTOTYPE_FIXTURES.get_or_init(|| {
        material_prototype_files()
            .iter()
            .map(|path| {
                let key = material_prototype_key(path).to_string();
                let source = fs::read_to_string(path).unwrap_or_else(|error| {
                    panic!("{} should be readable: {error}", path.display())
                });
                let document = UiZuiAssetLoader::load_zui_str(&source).unwrap_or_else(|error| {
                    panic!("{} should load as .zui: {error}", path.display())
                });
                (
                    key,
                    MaterialPrototypeFixture {
                        path: path.clone(),
                        source,
                        document,
                    },
                )
            })
            .collect()
    })
}

pub(super) fn material_prototype_fixture(key: &str) -> &'static MaterialPrototypeFixture {
    material_prototype_fixtures()
        .get(key)
        .unwrap_or_else(|| panic!("Material prototype `{key}` should exist"))
}

fn material_prototype_key(path: &Path) -> &str {
    path.file_stem()
        .and_then(|name| name.to_str())
        .expect("prototype file stem is UTF-8")
        .strip_prefix("material_")
        .expect("prototype files use material_ prefix")
}

pub(super) fn material_prototype_component_names() -> BTreeSet<String> {
    material_prototype_components_by_key()
        .into_values()
        .collect()
}

pub(super) fn material_prototype_components_by_key() -> BTreeMap<String, String> {
    material_prototype_fixtures()
        .iter()
        .map(|(key, fixture)| {
            let component = fixture
                .document
                .components
                .keys()
                .next()
                .expect("prototype declares one component")
                .to_string();
            (key.clone(), component)
        })
        .collect()
}

pub(super) fn material_lab_section_ids() -> [&'static str; 8] {
    [
        "section_data_display",
        "section_feedback",
        "section_inputs",
        "section_layout",
        "section_mui_x",
        "section_navigation",
        "section_surfaces",
        "section_utils_lab",
    ]
}

pub(super) fn material_lab_event_ids() -> Vec<String> {
    material_lab_event_specs()
        .into_iter()
        .map(|(id, _)| id)
        .collect()
}

pub(super) fn material_lab_event_specs() -> BTreeSet<(String, String)> {
    let mut specs = BTreeSet::new();
    for fixture in material_prototype_fixtures().values() {
        for node in fixture.document.nodes.values() {
            for event in &node.events {
                if event.id.starts_with("MaterialLab/") {
                    specs.insert((event.id.clone(), format!("{:?}", event.event)));
                }
            }
        }
    }
    specs
}

pub(super) fn editor_material_theme_source() -> &'static str {
    EDITOR_MATERIAL_THEME_SOURCE
        .get_or_init(|| {
            let path = editor_asset("assets/ui/theme/editor_material.zui");
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()))
        })
        .as_str()
}

pub(super) fn editor_material_theme_selectors() -> &'static BTreeSet<String> {
    EDITOR_MATERIAL_THEME_SELECTORS.get_or_init(|| {
        toml::from_str::<toml::Value>(editor_material_theme_source())
            .unwrap_or_else(|error| panic!("Editor Material theme should parse as TOML: {error}"))
            .get("stylesheets")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
            .flat_map(|stylesheet| {
                stylesheet
                    .get("rules")
                    .and_then(toml::Value::as_array)
                    .into_iter()
                    .flatten()
            })
            .filter_map(|rule| rule.get("selector").and_then(toml::Value::as_str))
            .map(ToOwned::to_owned)
            .collect()
    })
}

pub(super) fn mui_docs_keys() -> BTreeSet<String> {
    let root = workspace_root().join("dev/material-ui/docs/data/material/components");
    fs::read_dir(&root)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", root.display()))
        .filter_map(|entry| {
            let entry = entry.expect("MUI docs component entry should be readable");
            entry
                .path()
                .is_dir()
                .then(|| entry.file_name().to_string_lossy().to_string())
        })
        .collect()
}

pub(super) fn editor_asset(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

pub(super) fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_editor lives directly under the workspace root")
        .to_path_buf()
}
