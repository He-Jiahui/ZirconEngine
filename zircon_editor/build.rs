fn main() {
    emit_rerun_if_changed_recursive("assets").expect("track editor asset resources recursively");
    write_viewport_gizmo_icon_manifest().expect("generate viewport gizmo icon manifest");
    write_editor_plugin_catalog_manifest().expect("generate editor plugin catalog manifest");
}

fn emit_rerun_if_changed_recursive(root: &str) -> Result<(), Box<dyn std::error::Error>> {
    visit_rerun_if_changed(&std::path::PathBuf::from(root))
}

fn visit_rerun_if_changed(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed={}", path.display());
    if !path.is_dir() {
        return Ok(());
    }

    let mut entries = std::fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        visit_rerun_if_changed(&entry.path())?;
    }
    Ok(())
}

fn write_viewport_gizmo_icon_manifest() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let manifest_path = out_dir.join("viewport_gizmo_icon_manifest.rs");
    let icons = [
        ("Camera", "camera.pbm"),
        ("DirectionalLight", "directional_light.pbm"),
    ];

    println!("cargo:rerun-if-changed=assets/viewport_gizmos");
    for (_, filename) in icons {
        println!("cargo:rerun-if-changed=assets/viewport_gizmos/{filename}");
    }

    let mut generated = String::from(
        "pub(crate) fn viewport_gizmo_icon_bytes(id: ViewportIconId) -> Option<&'static [u8]> {\n    match id {\n",
    );
    for (variant, filename) in icons {
        generated.push_str(&format!(
            "        ViewportIconId::{variant} => Some(include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/assets/viewport_gizmos/{filename}\"))),\n"
        ));
    }
    generated.push_str("    }\n}\n");
    fs::write(manifest_path, generated)?;
    Ok(())
}

fn write_editor_plugin_catalog_manifest() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let plugins_root = manifest_dir
        .parent()
        .ok_or("editor crate should have repository parent")?
        .join("zircon_plugins");
    println!("cargo:rerun-if-changed={}", plugins_root.display());

    let mut entries = Vec::new();
    let mut plugin_dirs = fs::read_dir(&plugins_root)?.collect::<Result<Vec<_>, _>>()?;
    plugin_dirs.sort_by_key(|entry| entry.path());
    for plugin_dir in plugin_dirs {
        let plugin_manifest = plugin_dir.path().join("plugin.toml");
        if !plugin_manifest.is_file() {
            continue;
        }
        println!("cargo:rerun-if-changed={}", plugin_manifest.display());
        if let Some(entry) = editor_catalog_entry_from_manifest(&plugin_manifest)? {
            entries.push(entry);
        }
    }
    entries.sort_by(|left, right| left.package_id.cmp(&right.package_id));

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let manifest_path = out_dir.join("editor_plugin_catalog_gen.rs");
    let mut generated = String::from(
        "pub(crate) const GENERATED_EDITOR_PLUGIN_CATALOG: &[GeneratedEditorPluginCatalogEntry] = &[\n",
    );
    for entry in entries {
        generated.push_str("    GeneratedEditorPluginCatalogEntry {\n");
        generated.push_str(&format!("        package_id: {:?},\n", entry.package_id));
        generated.push_str(&format!(
            "        display_name: {:?},\n",
            entry.display_name
        ));
        generated.push_str(&format!("        crate_name: {:?},\n", entry.crate_name));
        generated.push_str(&format!("        category: {:?},\n", entry.category));
        generated.push_str("        capabilities: &[\n");
        for capability in entry.capabilities {
            generated.push_str(&format!("            {:?},\n", capability));
        }
        generated.push_str("        ],\n");
        generated.push_str("    },\n");
    }
    generated.push_str("];\n");
    fs::write(manifest_path, generated)?;
    Ok(())
}

fn editor_catalog_entry_from_manifest(
    path: &std::path::Path,
) -> Result<Option<EditorCatalogEntry>, Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path)?;
    let manifest = source.parse::<toml::Table>()?;
    let Some(editor_module) = manifest
        .get("modules")
        .and_then(toml::Value::as_array)
        .and_then(|modules| {
            modules.iter().find(|module| {
                module
                    .get("kind")
                    .and_then(toml::Value::as_str)
                    .is_some_and(|kind| kind == "editor")
            })
        })
    else {
        return Ok(None);
    };

    let package_id = required_manifest_str(manifest.get("id"), "id", path)?.to_string();
    let display_name =
        required_manifest_str(manifest.get("display_name"), "display_name", path)?.to_string();
    let category = manifest
        .get("category")
        .and_then(toml::Value::as_str)
        .unwrap_or("uncategorized")
        .to_string();
    let crate_name =
        required_manifest_str(editor_module.get("crate_name"), "crate_name", path)?.to_string();
    let capabilities = editor_module
        .get("capabilities")
        .and_then(toml::Value::as_array)
        .map(|capabilities| {
            capabilities
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(Some(EditorCatalogEntry {
        package_id,
        display_name,
        crate_name,
        category,
        capabilities,
    }))
}

fn required_manifest_str<'a>(
    value: Option<&'a toml::Value>,
    field: &str,
    path: &std::path::Path,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    value
        .and_then(toml::Value::as_str)
        .ok_or_else(|| format!("{} missing required `{field}`", path.display()).into())
}

struct EditorCatalogEntry {
    package_id: String,
    display_name: String,
    crate_name: String,
    category: String,
    capabilities: Vec<String>,
}
