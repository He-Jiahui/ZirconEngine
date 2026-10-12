use std::path::{Path, PathBuf};

use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::v2::{UiV2AssetDocument, UiV2AssetKind};

use crate::ui::template_runtime::EditorUiHostRuntime;

use super::contract::{contained, verify_review_host, Entry, ReviewCase};

pub(super) fn register(
    runtime: &mut EditorUiHostRuntime,
    repo: &Path,
    entry: &Entry,
    case: &ReviewCase,
    directory: &Path,
) -> Result<(), String> {
    validate_workbench_state_source(repo, entry, case)?;
    let source = contained(repo, &entry.source_path)?;
    let document = UiZuiAssetLoader::load_zui_file(&source).map_err(|error| error.to_string())?;
    let host = verify_review_host(&repo.join("docs/layout"), entry, case)?;
    let root = match host {
        Some(path) => {
            if case.data.get("componentInput").is_some() {
                return Err("component input cannot replace an explicit slot host".into());
            }
            let host = UiZuiAssetLoader::load_zui_file(&path).map_err(|error| error.to_string())?;
            if host.asset.kind != UiV2AssetKind::View || host.root_node_id().is_none() {
                return Err("Editor review host must be a native view with a root".into());
            }
            path
        }
        None if document.asset.kind == UiV2AssetKind::Component => {
            mount_component(&document, case, directory)?
        }
        None if document.asset.kind == UiV2AssetKind::View => source.clone(),
        None => {
            return Err(
                "style/theme assets require a fingerprinted component-state specimen".into(),
            );
        }
    };
    let mut paths = vec![root];
    for path in std::iter::once(source).chain(entry.zui_dependencies(repo)?) {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    // A generated editor host lives under `docs/layout`, so the V2 file-cache
    // cannot infer its `res://` asset root from the host path.  The catalog
    // writer records the authored consumers used to assemble that host in a
    // sibling source map; include those source files explicitly so the real
    // prototype store expands every mounted shell/window component instead of
    // silently retaining only the host root.
    if case.host == "editor" {
        let map_relative = format!(
            "{}/{}/evidence/editor-host-sources.json",
            entry.category, entry.name
        );
        let map_path = output_root(repo, &map_relative)?;
        if map_path.is_file() {
            let map: serde_json::Value = serde_json::from_slice(
                &std::fs::read(&map_path).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
            if let Some(consumers) = map.as_array() {
                for consumer in consumers {
                    let Some(relative) =
                        consumer.get("sourcePath").and_then(|value| value.as_str())
                    else {
                        return Err("editor host source map contains a missing sourcePath".into());
                    };
                    let path = contained(repo, relative)?;
                    if path.extension().and_then(|value| value.to_str()) == Some("zui")
                        && !paths.contains(&path)
                    {
                        paths.push(path);
                    }
                }
            } else {
                return Err("editor host source map must be an array".into());
            }
        }
    }
    if let Some(theme) = &case.theme_source_path {
        if theme != &entry.source_path
            && !entry
                .dependency_fingerprints
                .iter()
                .any(|item| item["sourcePath"].as_str() == Some(theme))
        {
            return Err("Editor host theme is not a fingerprinted dependency".into());
        }
        let path = contained(repo, theme)?;
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    runtime
        .register_v2_template_document_files("layout-review", paths)
        .map_err(|error| error.to_string())
}

fn validate_workbench_state_source(
    repo: &Path,
    entry: &Entry,
    case: &ReviewCase,
) -> Result<(), String> {
    let Some(selector) = super::state::workbench_state_selector(&case.data)? else {
        return Ok(());
    };
    if case.host != "editor" {
        return Err("Workbench state selectors require the editor host".into());
    }
    validate_fingerprinted_source(repo, entry, &selector.source_path)?;
    validate_authored_control(
        repo,
        &selector.source_path,
        &selector.control_id,
        selector.source_node_id.as_deref(),
        false,
    )?;
    if let Some(scroll_target) = &selector.scroll_target {
        validate_fingerprinted_source(repo, entry, &scroll_target.source_path)?;
        validate_authored_control(
            repo,
            &scroll_target.source_path,
            &scroll_target.control_id,
            scroll_target.source_node_id.as_deref(),
            false,
        )?;
    }
    for override_value in &selector.text_overrides {
        validate_fingerprinted_source(repo, entry, &override_value.source_path)?;
        validate_authored_control(
            repo,
            &override_value.source_path,
            &override_value.control_id,
            Some(&override_value.source_node_id),
            true,
        )?;
    }
    Ok(())
}

fn validate_fingerprinted_source(
    repo: &Path,
    entry: &Entry,
    source_path: &str,
) -> Result<PathBuf, String> {
    let is_fingerprinted_source = source_path == entry.source_path
        || entry.dependency_fingerprints.iter().any(|fingerprint| {
            fingerprint
                .get("sourcePath")
                .and_then(serde_json::Value::as_str)
                == Some(source_path)
        });
    if !is_fingerprinted_source || !source_path.ends_with(".zui") {
        return Err(
            "Workbench state sourcePath must be the case source or a fingerprinted ZUI dependency"
                .into(),
        );
    }
    contained(repo, source_path)
}

fn validate_authored_control(
    repo: &Path,
    source_path: &str,
    control_id: &str,
    source_node_id: Option<&str>,
    require_text: bool,
) -> Result<(), String> {
    let source_path = contained(repo, source_path)?;
    let source = std::fs::read_to_string(&source_path).map_err(|error| error.to_string())?;
    let document: toml::Value = toml::from_str(&source).map_err(|error| error.to_string())?;
    let nodes = document
        .get("nodes")
        .and_then(toml::Value::as_table)
        .ok_or("Workbench state source has no authored node table")?;
    let matches_control = |node: &toml::Value| {
        node.get("control_id").and_then(toml::Value::as_str) == Some(control_id)
    };
    let node = if let Some(source_node_id) = source_node_id {
        let node = nodes
            .get(source_node_id)
            .ok_or("Workbench state sourceNodeId is not in its source")?;
        if !matches_control(node) {
            return Err("Workbench state sourceNodeId does not match controlId".into());
        }
        node
    } else {
        let matches = nodes
            .values()
            .filter(|node| matches_control(node))
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [node] => *node,
            _ => {
                return Err(format!(
                    "Workbench state controlId must be unique in its source, found {}",
                    matches.len()
                ));
            }
        }
    };
    if require_text
        && node
            .get("props")
            .and_then(toml::Value::as_table)
            .and_then(|props| props.get("text"))
            .and_then(toml::Value::as_str)
            .is_none()
    {
        return Err("Workbench text override must target an authored string props.text".into());
    }
    Ok(())
}

fn output_root(repo: &Path, relative: &str) -> Result<PathBuf, String> {
    contained(&repo.join("docs/layout"), relative)
}

fn mount_component(
    document: &UiV2AssetDocument,
    case: &ReviewCase,
    directory: &Path,
) -> Result<PathBuf, String> {
    if document.components.len() != 1 {
        return Err(
            "component capture requires one declared export and an explicit slot specimen".into(),
        );
    }
    let component = document
        .components
        .keys()
        .next()
        .ok_or("missing component export")?;
    // Real instancing applies defaults, slots, bindings and required-slot checks.
    let source = toml::to_string(&serde_json::json!({
        "asset": {"kind": "view", "id": format!("{}#editor-review", document.asset.id), "version": 2},
        "imports": {"widgets": [document.asset.id]},
        "root": {"node": "review_mount"},
        "nodes": {"review_mount": {
            "component": format!("{}#{component}", document.asset.id),
            "props": super::component_input::props(document, &case.data)?,
        }},
    })).map_err(|error| error.to_string())?;
    let path = directory.join(format!("editor-host-{}.zui", case.id));
    std::fs::write(&path, source).map_err(|error| error.to_string())?;
    Ok(path)
}
