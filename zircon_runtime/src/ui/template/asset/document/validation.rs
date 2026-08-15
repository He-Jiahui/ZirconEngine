use std::collections::BTreeMap;

use zircon_runtime_interface::ui::template::{
    UiAssetError, UiNodeDefinition, UiSelector, UiStyleSheet,
};

pub(super) fn validate_node_tree<'a>(
    asset_id: &str,
    scope: &str,
    node: &'a UiNodeDefinition,
    seen: &mut BTreeMap<&'a str, &'a UiNodeDefinition>,
) -> Result<(), UiAssetError> {
    if node.node_id.trim().is_empty() {
        return Err(UiAssetError::InvalidDocument {
            asset_id: asset_id.to_string(),
            detail: format!("{scope} contains a node with an empty node_id"),
        });
    }
    if let Some(existing) = seen.get(node.node_id.as_str()) {
        if *existing == node {
            return Ok(());
        }
        return Err(UiAssetError::InvalidDocument {
            asset_id: asset_id.to_string(),
            detail: format!(
                "duplicate node_id {} resolves to conflicting subtrees",
                node.node_id
            ),
        });
    }
    let _ = seen.insert(node.node_id.as_str(), node);
    validate_action_refs(asset_id, node)?;
    for child in &node.children {
        validate_node_tree(asset_id, scope, &child.node, seen)?;
    }
    Ok(())
}

fn validate_action_refs(asset_id: &str, node: &UiNodeDefinition) -> Result<(), UiAssetError> {
    for binding in &node.bindings {
        let Some(action_ref) = binding.action.as_ref() else {
            continue;
        };
        let route = action_ref
            .route
            .as_deref()
            .map(str::trim)
            .filter(|route| !route.is_empty());
        let action = action_ref
            .action
            .as_deref()
            .map(str::trim)
            .filter(|action| !action.is_empty());
        if route.is_none() && action.is_none() {
            return Err(UiAssetError::InvalidDocument {
                asset_id: asset_id.to_string(),
                detail: format!(
                    "binding {} on node {} action ref must declare exactly one non-empty target",
                    binding.id, node.node_id
                ),
            });
        }
        if route.is_some() && action.is_some() {
            return Err(UiAssetError::InvalidDocument {
                asset_id: asset_id.to_string(),
                detail: format!(
                    "binding {} on node {} must declare exactly one action target: route or action",
                    binding.id, node.node_id
                ),
            });
        }
        if action.is_some() && !action_ref.payload.is_empty() {
            return Err(UiAssetError::InvalidDocument {
                asset_id: asset_id.to_string(),
                detail: format!(
                    "binding {} on node {} command action must not carry payload",
                    binding.id, node.node_id
                ),
            });
        }
    }
    Ok(())
}

pub(super) fn validate_stylesheet_ids(
    asset_id: &str,
    stylesheets: &[UiStyleSheet],
) -> Result<(), UiAssetError> {
    let mut seen = BTreeMap::new();
    for (stylesheet_index, stylesheet) in stylesheets.iter().enumerate() {
        if stylesheet.id.trim().is_empty() {
            return Err(UiAssetError::InvalidDocument {
                asset_id: asset_id.to_string(),
                detail: "stylesheet id cannot be empty".to_string(),
            });
        }
        if let Some(first_index) = seen.insert(stylesheet.id.as_str(), stylesheet_index) {
            return Err(UiAssetError::InvalidDocument {
                asset_id: asset_id.to_string(),
                detail: format!(
                    "duplicate stylesheet id {} appears at indexes {first_index} and {}",
                    stylesheet.id, stylesheet_index
                ),
            });
        }
    }
    Ok(())
}

pub(super) fn validate_style_rule_ids(
    asset_id: &str,
    stylesheets: &[UiStyleSheet],
) -> Result<(), UiAssetError> {
    let mut seen = BTreeMap::new();
    for stylesheet in stylesheets {
        for rule in &stylesheet.rules {
            let Some(rule_id) = rule.id.as_deref() else {
                continue;
            };
            if rule_id.trim().is_empty() {
                return Err(UiAssetError::InvalidDocument {
                    asset_id: asset_id.to_string(),
                    detail: format!(
                        "stylesheet {} contains a style rule with an empty id",
                        stylesheet.id
                    ),
                });
            }
            if let Some(first_stylesheet) = seen.insert(rule_id, stylesheet.id.as_str()) {
                return Err(UiAssetError::InvalidDocument {
                    asset_id: asset_id.to_string(),
                    detail: format!(
                        "duplicate style rule id {rule_id} appears in stylesheets {first_stylesheet} and {}",
                        stylesheet.id
                    ),
                });
            }
        }
    }
    Ok(())
}

pub(super) fn validate_style_rule_selectors(
    stylesheets: &[UiStyleSheet],
) -> Result<(), UiAssetError> {
    for stylesheet in stylesheets {
        for rule in &stylesheet.rules {
            UiSelector::parse(&rule.selector)?;
        }
    }
    Ok(())
}
