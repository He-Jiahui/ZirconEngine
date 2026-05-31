use std::collections::BTreeMap;
use std::fs;

use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::metadata::{is_control_id_identifier, string_token_metadata_offender};
use super::support::{collect_zui_files, editor_asset_root, runtime_asset_root};

#[test]
fn production_zui_control_ids_are_unique_within_each_component_asset() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_control_ids = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));
            let mut control_ids = BTreeMap::<String, Vec<String>>::new();

            for (node_id, node) in &document.nodes {
                let Some(control_id) = node.control_id.as_deref() else {
                    continue;
                };
                if let Some(invalid_control_id) =
                    string_token_metadata_offender(control_id, "control_id")
                {
                    offenders.push(format!(
                        "{} node `{}` declares {invalid_control_id}",
                        path.display(),
                        node_id
                    ));
                    continue;
                }
                if !is_control_id_identifier(control_id) {
                    offenders.push(format!(
                        "{} node `{}` declares control_id `{control_id}` outside selector-safe identifier form",
                        path.display(),
                        node_id
                    ));
                    continue;
                }
                checked_control_ids += 1;
                control_ids
                    .entry(control_id.trim().to_string())
                    .or_default()
                    .push(node_id.clone());
            }

            for (control_id, node_ids) in control_ids {
                if node_ids.len() > 1 {
                    offenders.push(format!(
                        "{} declares duplicate control_id `{}` on nodes {node_ids:?}",
                        path.display(),
                        control_id
                    ));
                }
            }
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        checked_control_ids > 0,
        "production .zui assets should expose author-facing control ids"
    );
    assert!(
        offenders.is_empty(),
        "production .zui control_id values must be non-empty, trimmed, whitespace-free, selector-safe, and unique within each component asset: {offenders:#?}"
    );
}
