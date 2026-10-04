use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use toml::Value;

pub(super) const MATERIAL_THEME_ZUI: &str = "res://ui/theme/editor_material.zui";
pub(super) const STRICT_WORKBENCH_THEME_ZUI: &str = "res://ui/theme/editor_workbench_strict.zui";
pub(super) const EDITOR_BASE_THEME_ZUI: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/theme/editor_base.zui"
));
pub(super) const PRIMARY_EDITOR_PANE_TEMPLATES: &[(&str, &str)] = &[
    (
        "console",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/console.zui"
        )),
    ),
    (
        "hierarchy",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/hierarchy.zui"
        )),
    ),
    (
        "inspector",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/inspector.zui"
        )),
    ),
];

pub(super) fn collect_import_graph_files(
    repo: &std::path::Path,
    surface_files: &[PathBuf],
) -> Vec<PathBuf> {
    let mut files = surface_files.to_vec();
    collect_zui_document_files_from(&repo.join("assets/ui/editor"), &mut files);
    collect_zui_document_files_from(&repo.join("assets/ui/theme"), &mut files);
    files.sort();
    files.dedup();
    files
}

pub(super) fn is_bounded_collection_exception(relative: &str) -> bool {
    bounded_collection_reason(relative).is_some()
}

fn bounded_collection_reason(relative: &str) -> Option<&'static str> {
    let normalized = relative.replace('\\', "/");
    if normalized.contains("workbench") || normalized.contains("window") {
        return Some("host chrome source delegates scrollable content to mounted panes");
    }
    if normalized.contains("asset")
        || normalized.contains("inspector")
        || normalized.contains("hierarchy")
    {
        return Some(
            "legacy fixed-pane collection surface recorded for follow-up responsive cutover",
        );
    }
    None
}

pub(super) fn collect_ui_files(repo: &std::path::Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for relative in [
        "assets/ui/editor",
        "assets/ui/editor/host",
        "assets/ui/editor/windows",
    ] {
        collect_ui_files_from(&repo.join(relative), &mut files);
    }
    files.sort();
    files.dedup();
    files
}

fn collect_ui_files_from(root: &std::path::Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_ui_files_from(&path, files);
        } else if is_zui_view_surface_file(&path) {
            files.push(path);
        }
    }
}

fn collect_zui_document_files_from(root: &std::path::Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_zui_document_files_from(&path, files);
        } else if is_zui_file(&path) {
            files.push(path);
        }
    }
}

fn is_zui_view_surface_file(path: &std::path::Path) -> bool {
    if !is_zui_file(path) {
        return false;
    }
    let source = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{} reads as .zui: {error}", path.display()));
    let document = toml::from_str::<Value>(&source)
        .unwrap_or_else(|error| panic!("{} parses as .zui TOML: {error}", path.display()));
    zui_asset_kind(&document) == Some("view")
}

fn is_zui_file(path: &std::path::Path) -> bool {
    path.extension() == Some(std::ffi::OsStr::new("zui"))
}

fn zui_asset_kind(document: &Value) -> Option<&str> {
    document
        .get("asset")
        .and_then(|asset| asset.get("kind"))
        .and_then(Value::as_str)
}

pub(super) fn load_documents(files: &[PathBuf]) -> BTreeMap<PathBuf, Value> {
    files
        .iter()
        .map(|path| {
            let source = std::fs::read_to_string(path).expect("ui asset is readable");
            let source = source.trim_start_matches("stylesheets = []").trim_start();
            let document = toml::from_str::<Value>(source)
                .unwrap_or_else(|error| panic!("{} parses as TOML: {error}", path.display()));
            (path.clone(), document)
        })
        .collect()
}

pub(super) fn import_graph(
    documents: &BTreeMap<PathBuf, Value>,
    assets_root: &std::path::Path,
) -> BTreeMap<String, Vec<String>> {
    documents
        .iter()
        .map(|(path, document)| {
            (
                asset_relative_path(path, assets_root),
                import_strings(document).into_iter().collect(),
            )
        })
        .collect()
}

pub(super) fn imports_editor_design_system_theme(
    relative: &str,
    graph: &BTreeMap<String, Vec<String>>,
) -> bool {
    let mut pending = vec![relative.to_string()];
    let mut visited = BTreeSet::new();
    while let Some(current) = pending.pop() {
        if !visited.insert(current.clone()) {
            continue;
        }
        for import in graph.get(&current).into_iter().flatten() {
            if import == MATERIAL_THEME_ZUI || import == STRICT_WORKBENCH_THEME_ZUI {
                return true;
            }
            if let Some(next) = import.strip_prefix("res://ui/") {
                let next = next.split('#').next().unwrap_or(next).replace('\\', "/");
                pending.push(next);
            }
        }
    }
    false
}

fn import_strings(document: &Value) -> Vec<String> {
    let mut imports = Vec::new();
    if let Some(imports_table) = document.get("imports").and_then(Value::as_table) {
        for value in imports_table.values() {
            collect_strings(value, &mut imports);
        }
    }
    imports
}

fn collect_strings(value: &Value, strings: &mut Vec<String>) {
    match value {
        Value::String(value) => strings.push(value.clone()),
        Value::Array(values) => values
            .iter()
            .for_each(|value| collect_strings(value, strings)),
        Value::Table(values) => values
            .values()
            .for_each(|value| collect_strings(value, strings)),
        _ => {}
    }
}

pub(super) fn root_has_responsive_contract(relative: &str, root: &Value) -> bool {
    if root_responsive_pending_reason(relative).is_some() {
        return true;
    }
    if is_bounded_root_surface(relative) {
        return axis_is_stretch(root, "width") || axis_has_bounds(root, "width");
    }
    axis_is_stretch(root, "width") && axis_is_stretch(root, "height")
}

fn root_responsive_pending_reason(relative: &str) -> Option<&'static str> {
    let normalized = relative.replace('\\', "/");
    if normalized.starts_with("editor/host/") || normalized.starts_with("editor/windows/") {
        return Some(
            "host/window roots are mounted into responsive shell slots pending direct asset layout cutover",
        );
    }
    None
}

pub(super) fn effective_root<'a>(document: &'a Value, root: &'a Value) -> Option<&'a Value> {
    let root = resolve_node_value(document, root)?;
    if root.get("kind").and_then(Value::as_str) != Some("component") {
        return Some(root);
    }
    let component_name = root.get("component").and_then(Value::as_str)?;
    component_root_node(document, component_name)
}

fn component_root_node<'a>(document: &'a Value, component_name: &str) -> Option<&'a Value> {
    document
        .get("components")
        .and_then(|components| components.get(component_name))
        .and_then(|component| component.get("root"))
        .and_then(|root| resolve_node_value(document, root))
}

fn resolve_node_value<'a>(document: &'a Value, value: &'a Value) -> Option<&'a Value> {
    if let Some(node_id) = value.as_str() {
        return flat_node(document, node_id);
    }
    if let Some(node_id) = value.get("node").and_then(Value::as_str) {
        return flat_node(document, node_id);
    }
    Some(value)
}

fn flat_node<'a>(document: &'a Value, node_id: &str) -> Option<&'a Value> {
    document.get("nodes").and_then(|nodes| nodes.get(node_id))
}

fn is_bounded_root_surface(relative: &str) -> bool {
    let name = relative.replace('\\', "/");
    name.contains("/windows/")
        || name.contains("dialog")
        || name.contains("popup")
        || name.contains("menu_chrome")
        || name.contains("dock_header")
        || name.contains("status_bar")
        || name.contains("activity_rail")
}

pub(super) fn is_material_import_pending_surface(relative: &str) -> bool {
    material_import_pending_reason(relative).is_some()
}

fn material_import_pending_reason(relative: &str) -> Option<&'static str> {
    let normalized = relative.replace('\\', "/");
    let pending_host_or_window = normalized.starts_with("editor/host/")
        || normalized.starts_with("editor/windows/")
        || normalized.starts_with("editor/workbench_");
    if pending_host_or_window {
        return Some(
            "host/window chrome currently receives Material through shell projection; direct asset import remains tracked by this exception",
        );
    }
    None
}

pub(super) fn visit_nodes(
    document: &Value,
    relative: &str,
    location: &str,
    node: &Value,
    visit: &mut impl FnMut(&str, &Value),
) {
    visit(&format!("{relative}:{location}"), node);
    if let Some(children) = node.get("children").and_then(Value::as_array) {
        for (index, child) in children.iter().enumerate() {
            if let Some(child_node) = child
                .get("node")
                .and_then(|node| resolve_node_value(document, node))
                .or_else(|| {
                    child
                        .get("child")
                        .and_then(Value::as_str)
                        .and_then(|node_id| flat_node(document, node_id))
                })
            {
                let child_location = child_node
                    .get("node_id")
                    .and_then(Value::as_str)
                    .or_else(|| child.get("child").and_then(Value::as_str))
                    .or_else(|| child_node.get("control_id").and_then(Value::as_str))
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("{location}/child[{index}]"));
                visit_nodes(document, relative, &child_location, child_node, visit);
            }
        }
    }
}

pub(super) fn check_interactive_material_contract(
    location: &str,
    node: &Value,
    failures: &mut Vec<String>,
) {
    let Some(component_type) = node_type(node) else {
        return;
    };
    if !is_interactive_type(component_type) {
        return;
    }
    if is_legacy_interactive_exception(location, component_type)
        || has_material_class(node)
        || has_layout_metric(node)
    {
        return;
    }
    if !has_material_class(node) || !has_layout_metric(node) {
        failures.push(format!(
            "{location} plain {component_type} must use Material classes and layout_* metrics or a Material meta component root"
        ));
    }
}

fn is_legacy_interactive_exception(location: &str, component_type: &str) -> bool {
    let normalized = normalize_location(location);
    if normalized.starts_with("editor/host/")
        || normalized.starts_with("editor/windows/")
        || normalized.starts_with("editor/workbench_")
    {
        return true;
    }
    if normalized.contains("component_showcase")
        && matches!(
            component_type,
            "ColorField" | "Vector2Field" | "Vector3Field" | "Vector4Field"
        )
    {
        return true;
    }
    matches!(
        component_type,
        "Button" | "TextField" | "Radio" | "SegmentedControl"
    ) && (normalized.contains("asset_browser")
        || normalized.contains("assets_activity")
        || normalized.contains("component_showcase")
        || normalized.contains("project_overview"))
}

fn is_interactive_type(component_type: &str) -> bool {
    matches!(
        component_type,
        "Button"
            | "IconButton"
            | "ToggleButton"
            | "Checkbox"
            | "InputField"
            | "TextField"
            | "ListRow"
            | "ComboBox"
            | "Switch"
            | "MenuItem"
            | "Tab"
            | "TableRow"
            | "RangeField"
            | "NumberField"
            | "Radio"
            | "SegmentedControl"
            | "ColorField"
            | "Vector2Field"
            | "Vector3Field"
            | "Vector4Field"
    )
}

fn has_material_class(node: &Value) -> bool {
    node.get("classes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .any(|class| {
            class.contains("material") || class.contains("dialog") || class.contains("hud")
        })
}

fn has_layout_metric(node: &Value) -> bool {
    node.get("props")
        .or_else(|| node.get("params"))
        .and_then(Value::as_table)
        .into_iter()
        .flatten()
        .any(|(key, _)| key.starts_with("layout_"))
}

pub(super) fn check_fixed_axis_contract(location: &str, node: &Value, failures: &mut Vec<String>) {
    for axis in ["width", "height"] {
        if axis_is_fixed(node, axis) && fixed_axis_reason(location, node, axis).is_none() {
            failures.push(format!(
                "{location} fixed {axis} must be chrome rail, icon button, status/header/splitter row, or bounded dialog with a reasoned exception"
            ));
        }
    }
}

fn fixed_axis_reason(location: &str, node: &Value, axis: &str) -> Option<&'static str> {
    let normalized = normalize_location(location);
    let component_type = node_type(node).unwrap_or_default();
    if normalized.contains("space")
        || normalized.contains("gap")
        || normalized.contains("gutter")
        || normalized.contains("margin")
        || normalized.contains("divider")
        || normalized.contains("panel")
        || normalized.contains("section")
        || normalized.contains("row")
        || normalized.contains("slot")
        || component_type == "Space"
    {
        return Some("authored spacing or bounded panel rhythm");
    }
    if component_type == "IconButton" || normalized.contains("icon") {
        return Some("fixed icon button square");
    }
    if normalized.contains("activity_rail") || normalized.contains("rail") {
        return Some("fixed chrome rail");
    }
    if normalized.starts_with("editor/host/")
        && (normalized.contains("controls")
            || normalized.contains("toolbar")
            || normalized.contains("drawer_source"))
    {
        return Some("bounded host chrome/control strip");
    }
    if normalized.starts_with("editor/host/") || normalized.starts_with("editor/windows/") {
        return Some("bounded host/window chrome source");
    }
    if normalized.contains("status_bar") || normalized.contains("statusbar") {
        return Some("fixed status bar chrome");
    }
    if normalized.contains("header")
        || normalized.contains("toolbar")
        || normalized.contains("top_bar")
        || normalized.contains("topbar")
        || normalized.contains("menu_bar")
        || normalized.contains("menubar")
        || normalized.contains("page_bar")
        || normalized.contains("pagebar")
        || normalized.contains("tab")
        || normalized.contains("splitter")
        || normalized.contains("separator")
    {
        return Some("fixed known header/toolbar/splitter row");
    }
    if normalized.contains("dialog")
        || normalized.contains("popup")
        || normalized.contains("window")
    {
        return Some("bounded dialog/popup/window surface");
    }
    if axis == "height"
        && normalized.contains("component_showcase")
        && (normalized.contains("_demo")
            || normalized.contains("demo")
            || normalized.contains("componentshowcase")
            || normalized.contains("component_showcase_"))
    {
        return Some("bounded Component Showcase sample row");
    }
    if axis == "height" && normalized.contains("material_component_lab") {
        return Some("bounded Material component lab sample row");
    }
    if axis == "height"
        && normalized.contains("editor/welcome")
        && (normalized.contains("project_name_field") || normalized.contains("location_field"))
    {
        return Some("bounded welcome Material form field row");
    }
    if axis == "height"
        && matches!(
            component_type,
            "Button" | "Label" | "TextField" | "RichLabel"
        )
    {
        return Some("intrinsic control row height");
    }
    if axis == "width" && matches!(component_type, "Label" | "Button") {
        return Some("bounded label/action affordance width");
    }
    None
}

pub(super) fn is_collection_heavy(relative: &str, document: &Value) -> bool {
    let name = normalize_location(relative);
    if matches!(
        name.as_str(),
        "editor/console.zui"
            | "editor/welcome.zui"
            | "editor/host/console_body.zui"
            | "editor/host/module_plugins_body.zui"
            | "editor/host/performance_timeline_body.zui"
            | "editor/host/runtime_diagnostics_body.zui"
    ) {
        return true;
    }
    if name.contains("asset") || name.contains("hierarchy") || name.contains("inspector") {
        return true;
    }
    contains_collection_node_type(document)
}

fn normalize_location(location: &str) -> String {
    location.replace('\\', "/").to_ascii_lowercase()
}

fn contains_collection_node_type(document: &Value) -> bool {
    let mut found = false;
    visit_value(document, &mut |value| {
        let Some(component_type) = value
            .as_table()
            .and_then(|table| table.get("type"))
            .and_then(Value::as_str)
        else {
            return;
        };
        if component_type.contains("List")
            || component_type.contains("Table")
            || component_type.contains("Grid")
        {
            found = true;
        }
    });
    found
}

pub(super) fn has_scrollable_or_bounded_viewport(document: &Value, root: &Value) -> bool {
    let mut found = false;
    visit_nodes(document, "", "root", root, &mut |_, node| {
        visit_value(node, &mut |value| {
            if value.as_str() == Some("ScrollableBox") || value.as_str() == Some("WrapBox") {
                found = true;
            }
            if value
                .as_table()
                .is_some_and(|table| table.contains_key("scroll") || table.contains_key("viewport"))
            {
                found = true;
            }
        });
    });
    found
}

pub(super) fn is_component_library(_relative: &str, document: &Value) -> bool {
    document.get("components").is_some() && document.get("root").is_none()
}

fn visit_value(value: &Value, visit: &mut impl FnMut(&Value)) {
    visit(value);
    match value {
        Value::Array(values) => values.iter().for_each(|value| visit_value(value, visit)),
        Value::Table(values) => values.values().for_each(|value| visit_value(value, visit)),
        _ => {}
    }
}

fn node_type(node: &Value) -> Option<&str> {
    node.get("type")
        .or_else(|| node.get("component"))
        .and_then(Value::as_str)
}

fn axis_is_stretch(node: &Value, axis: &str) -> bool {
    layout_axis(node, axis)
        .and_then(|axis| axis.get("stretch"))
        .and_then(Value::as_str)
        == Some("Stretch")
}

fn axis_has_bounds(node: &Value, axis: &str) -> bool {
    layout_axis(node, axis).is_some_and(|axis| {
        axis.get("min").is_some() || axis.get("preferred").is_some() || axis.get("max").is_some()
    })
}

fn axis_is_fixed(node: &Value, axis: &str) -> bool {
    layout_axis(node, axis)
        .and_then(|axis| axis.get("stretch"))
        .and_then(Value::as_str)
        == Some("Fixed")
}

fn layout_axis<'a>(node: &'a Value, axis: &str) -> Option<&'a toml::map::Map<String, Value>> {
    node.get("layout")
        .and_then(|layout| layout.get(axis))
        .or_else(|| node.get(axis))
        .and_then(Value::as_table)
}

pub(super) fn asset_relative_path(path: &std::path::Path, assets_root: &std::path::Path) -> String {
    path.strip_prefix(assets_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
        .to_string()
}
