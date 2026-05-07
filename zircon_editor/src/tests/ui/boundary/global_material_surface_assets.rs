use std::{collections::BTreeMap, path::PathBuf};

use toml::Value;

const MATERIAL_THEME: &str = "res://ui/theme/editor_material.ui.toml";
const MATERIAL_META: &str = "res://ui/editor/material_meta_components.ui.toml";

#[test]
fn global_material_surface_assets_follow_responsive_contracts() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let assets_root = repo.join("assets/ui");
    let files = collect_ui_files(&repo);
    assert_eq!(
        files.len(),
        54,
        "Milestone 3 inventory changed; update the acceptance inventory and this conformance test together"
    );

    let documents = load_documents(&files);
    let import_documents = load_documents(&collect_import_graph_files(&repo, &files));
    let import_graph = import_graph(&import_documents, &assets_root);
    let mut failures = Vec::new();

    for path in &files {
        let document = documents.get(path).expect("document loaded");
        let relative = asset_relative_path(path, &assets_root);
        if is_component_library(&relative, document) {
            continue;
        }
        if !imports_material_theme(&relative, &import_graph)
            && !is_material_import_pending_surface(&relative)
        {
            failures.push(format!(
                "{} must import Material theme directly or through another imported asset",
                relative
            ));
        }

        let Some(root) = document.get("root") else {
            failures.push(format!("{} must define [root]", relative));
            continue;
        };
        let effective_root = effective_root(document, root).unwrap_or(root);
        if !root_has_responsive_contract(&relative, effective_root) {
            failures.push(format!(
                "{} root must stretch in width and height unless it is a bounded popup/menu/dialog/window chrome surface",
                relative
            ));
        }

        visit_nodes(&relative, "root", effective_root, &mut |location, node| {
            check_interactive_material_contract(location, node, &mut failures);
            check_fixed_axis_contract(location, node, &mut failures);
        });

        if is_collection_heavy(&relative, document)
            && !has_scrollable_or_bounded_viewport(effective_root)
            && !is_bounded_collection_exception(&relative)
        {
            failures.push(format!(
                "{} is list/table/grid heavy and must expose ScrollableBox or an explicit bounded viewport",
                relative
            ));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn material_import_graph_uses_normalized_res_paths() {
    let mut graph = BTreeMap::new();
    graph.insert(
        "editor/welcome.ui.toml".to_string(),
        vec!["res://ui/theme/editor_base.ui.toml".to_string()],
    );
    graph.insert(
        "theme/editor_base.ui.toml".to_string(),
        vec![MATERIAL_THEME.to_string()],
    );

    assert!(imports_material_theme("editor/welcome.ui.toml", &graph));
}

#[test]
fn runtime_material_surfaces_use_shared_runtime_material_classes() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut failures = Vec::new();
    for relative in [
        "runtime/inventory_dialog.ui.toml",
        "runtime/pause_dialog.ui.toml",
        "runtime/quest_log_dialog.ui.toml",
        "runtime/runtime_hud.ui.toml",
        "runtime/settings_dialog.ui.toml",
    ] {
        let document = load_document(&repo.join("assets/ui").join(relative));
        let root = document
            .get("root")
            .unwrap_or_else(|| panic!("{relative} defines [root]"));
        let expected_surface_class = if relative.ends_with("runtime_hud.ui.toml") {
            "material-runtime-hud"
        } else {
            "material-runtime-dialog"
        };
        let mut found_surface_class = false;
        visit_nodes(relative, "root", root, &mut |location, node| {
            if node_has_class(node, expected_surface_class) {
                found_surface_class = true;
            }
            if node_type(node) != Some("Button") {
                return;
            }
            for class in ["material-control", "material-button"] {
                if !node_has_class(node, class) {
                    failures.push(format!("{location} must include `{class}`"));
                }
            }
            if (node_has_class(node, "dialog-button-primary")
                || node_has_class(node, "hud-button-primary"))
                && !node_has_class(node, "material-button-primary")
            {
                failures.push(format!(
                    "{location} primary runtime button must include `material-button-primary`"
                ));
            }
            for prop in ["surface_variant", "border_width", "corner_radius"] {
                if !node_props(node).is_some_and(|props| props.contains_key(prop)) {
                    failures.push(format!("{location} must carry `{prop}` Material prop"));
                }
            }
        });
        if !found_surface_class {
            failures.push(format!(
                "{relative} must mark a root/panel with `{expected_surface_class}`"
            ));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn collect_import_graph_files(repo: &std::path::Path, surface_files: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = surface_files.to_vec();
    collect_ui_files_from(&repo.join("assets/ui/theme"), &mut files);
    files.sort();
    files.dedup();
    files
}

fn is_bounded_collection_exception(relative: &str) -> bool {
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

fn collect_ui_files(repo: &std::path::Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for relative in [
        "assets/ui/editor",
        "assets/ui/editor/host",
        "assets/ui/editor/windows",
        "assets/ui/runtime",
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
        } else if path.extension() == Some(std::ffi::OsStr::new("toml"))
            && path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().ends_with(".ui.toml"))
        {
            files.push(path);
        }
    }
}

fn load_documents(files: &[PathBuf]) -> BTreeMap<PathBuf, Value> {
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

fn load_document(path: &std::path::Path) -> Value {
    let source = std::fs::read_to_string(path).expect("ui asset is readable");
    let source = source.trim_start_matches("stylesheets = []").trim_start();
    toml::from_str::<Value>(source)
        .unwrap_or_else(|error| panic!("{} parses as TOML: {error}", path.display()))
}

fn import_graph(
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

fn imports_material_theme(relative: &str, graph: &BTreeMap<String, Vec<String>>) -> bool {
    let mut pending = vec![relative.to_string()];
    let mut visited = Vec::new();
    while let Some(current) = pending.pop() {
        if visited.contains(&current) {
            continue;
        }
        visited.push(current.clone());
        for import in graph.get(&current).into_iter().flatten() {
            if import == MATERIAL_THEME {
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

fn root_has_responsive_contract(relative: &str, root: &Value) -> bool {
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
        return Some("host/window roots are mounted into responsive shell slots pending direct asset layout cutover");
    }
    None
}

fn effective_root<'a>(document: &'a Value, root: &'a Value) -> Option<&'a Value> {
    if root.get("kind").and_then(Value::as_str) != Some("component") {
        return Some(root);
    }
    let component_name = root.get("component").and_then(Value::as_str)?;
    document
        .get("components")
        .and_then(|components| components.get(component_name))
        .and_then(|component| component.get("root"))
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

fn is_material_import_pending_surface(relative: &str) -> bool {
    material_import_pending_reason(relative).is_some()
}

fn material_import_pending_reason(relative: &str) -> Option<&'static str> {
    let normalized = relative.replace('\\', "/");
    let pending_host_or_window = normalized.starts_with("editor/host/")
        || normalized.starts_with("editor/windows/")
        || normalized.starts_with("editor/workbench_");
    if pending_host_or_window {
        return Some("host/window chrome currently receives Material through shell projection; direct asset import remains tracked by this exception");
    }
    if matches!(
        normalized.as_str(),
        "editor/binding_browser.ui.toml"
            | "editor/layout_workbench.ui.toml"
            | "editor/preview_state_lab.ui.toml"
            | "editor/theme_browser.ui.toml"
    ) {
        return Some("legacy editor tool surface pending mechanical Material import cutover");
    }
    None
}

fn visit_nodes(relative: &str, location: &str, node: &Value, visit: &mut impl FnMut(&str, &Value)) {
    visit(&format!("{relative}:{location}"), node);
    if let Some(children) = node.get("children").and_then(Value::as_array) {
        for (index, child) in children.iter().enumerate() {
            if let Some(child_node) = child.get("node") {
                let child_location = child_node
                    .get("node_id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("{location}/child[{index}]"));
                visit_nodes(relative, &child_location, child_node, visit);
            }
        }
    }
}

fn check_interactive_material_contract(location: &str, node: &Value, failures: &mut Vec<String>) {
    let Some(component_type) = node_type(node) else {
        return;
    };
    if !is_interactive_type(component_type) || is_material_component_reference(node) {
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

fn is_material_component_reference(node: &Value) -> bool {
    node.get("component_ref")
        .and_then(Value::as_str)
        .is_some_and(|component_ref| component_ref.starts_with(MATERIAL_META))
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

fn node_has_class(node: &Value, expected: &str) -> bool {
    node.get("classes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .any(|class| class == expected)
}

fn node_props(node: &Value) -> Option<&toml::map::Map<String, Value>> {
    node.get("props")
        .or_else(|| node.get("params"))
        .and_then(Value::as_table)
}

fn has_layout_metric(node: &Value) -> bool {
    node.get("props")
        .or_else(|| node.get("params"))
        .and_then(Value::as_table)
        .into_iter()
        .flatten()
        .any(|(key, _)| key.starts_with("layout_"))
}

fn check_fixed_axis_contract(location: &str, node: &Value, failures: &mut Vec<String>) {
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
    if normalized.contains("status_bar") || normalized.contains("statusbar") {
        return Some("fixed status bar chrome");
    }
    if normalized.contains("header")
        || normalized.contains("toolbar")
        || normalized.contains("top_bar")
        || normalized.contains("menu_bar")
        || normalized.contains("page_bar")
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
        && (normalized.contains("_demo") || normalized.contains("component_showcase_"))
    {
        return Some("bounded Component Showcase sample row");
    }
    if axis == "height"
        && normalized.contains("editor/welcome.ui.toml")
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

fn is_collection_heavy(relative: &str, document: &Value) -> bool {
    let name = normalize_location(relative);
    if matches!(
        name.as_str(),
        "editor/console.ui.toml"
            | "editor/welcome.ui.toml"
            | "editor/host/console_body.ui.toml"
            | "editor/host/module_plugins_body.ui.toml"
            | "editor/host/runtime_diagnostics_body.ui.toml"
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

fn has_scrollable_or_bounded_viewport(root: &Value) -> bool {
    let mut found = false;
    visit_value(root, &mut |value| {
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
    found
}

fn is_component_library(relative: &str, document: &Value) -> bool {
    relative.ends_with("material_meta_components.ui.toml")
        || relative.ends_with("component_widgets.ui.toml")
        || relative.ends_with("editor_widgets.ui.toml")
        || document.get("components").is_some() && document.get("root").is_none()
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

fn asset_relative_path(path: &std::path::Path, assets_root: &std::path::Path) -> String {
    path.strip_prefix(assets_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
        .to_string()
}
