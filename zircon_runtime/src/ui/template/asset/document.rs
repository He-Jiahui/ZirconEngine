use std::collections::BTreeMap;

use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiChildMount, UiNodeDefinition, UiSelector, UiStyleRule,
    UiStyleSheet,
};

pub trait UiAssetDocumentRuntimeExt {
    fn validate_tree_authority(&self) -> Result<(), UiAssetError>;
    fn root_node_id(&self) -> Option<&str>;
    fn contains_node(&self, node_id: &str) -> bool;
    fn node(&self, node_id: &str) -> Option<&UiNodeDefinition>;
    fn node_mut(&mut self, node_id: &str) -> Option<&mut UiNodeDefinition>;
    fn style_rule(&self, rule_id: &str) -> Option<&UiStyleRule>;
    fn style_rule_mut(&mut self, rule_id: &str) -> Option<&mut UiStyleRule>;
    fn style_rule_position(&self, rule_id: &str) -> Option<UiStyleRulePosition<'_>>;
    fn rename_style_rule(&mut self, current_id: &str, new_id: &str) -> Result<bool, UiAssetError>;
    fn remove_style_rule(&mut self, rule_id: &str) -> Option<UiStyleRule>;
    fn insert_style_rule(
        &mut self,
        stylesheet_id: &str,
        index: usize,
        rule: UiStyleRule,
    ) -> Result<bool, UiAssetError>;
    fn replace_style_rule(
        &mut self,
        rule_id: &str,
        replacement: UiStyleRule,
    ) -> Result<Option<UiStyleRule>, UiAssetError>;
    fn move_style_rule(
        &mut self,
        rule_id: &str,
        target_stylesheet_id: &str,
        target_index: usize,
    ) -> Result<bool, UiAssetError>;
    fn style_sheet(&self, stylesheet_id: &str) -> Option<&UiStyleSheet>;
    fn style_sheet_mut(&mut self, stylesheet_id: &str) -> Option<&mut UiStyleSheet>;
    fn style_sheet_index(&self, stylesheet_id: &str) -> Option<usize>;
    fn rename_style_sheet(&mut self, current_id: &str, new_id: &str) -> Result<bool, UiAssetError>;
    fn remove_style_sheet(&mut self, stylesheet_id: &str) -> Option<UiStyleSheet>;
    fn set_style_sheets(&mut self, stylesheets: Vec<UiStyleSheet>) -> Result<bool, UiAssetError>;
    fn insert_style_sheet(
        &mut self,
        index: usize,
        stylesheet: UiStyleSheet,
    ) -> Result<usize, UiAssetError>;
    fn replace_style_sheet(
        &mut self,
        stylesheet_id: &str,
        replacement: UiStyleSheet,
    ) -> Result<Option<UiStyleSheet>, UiAssetError>;
    fn move_style_sheet(&mut self, stylesheet_id: &str, target_index: usize) -> Option<usize>;
    fn component_root_node_id(&self, component_name: &str) -> Option<&str>;
    fn child_index_in_parent(&self, child_id: &str) -> Option<(String, usize)>;
    fn child_mount(&self, child_id: &str) -> Option<&UiChildMount>;
    fn child_mount_mut(&mut self, child_id: &str) -> Option<&mut UiChildMount>;
    fn parent_of(&self, child_id: &str) -> Option<UiNodeParent<'_>>;
    fn iter_nodes(&self) -> UiAssetNodeIter<'_>;
    fn node_map(&self) -> BTreeMap<String, UiNodeDefinition>;
    fn replace_node(&mut self, node_id: &str, replacement: UiNodeDefinition) -> bool;
    fn remove_node(&mut self, node_id: &str) -> Option<UiNodeDefinition>;
    fn insert_child(&mut self, parent_id: &str, index: usize, child: UiChildMount) -> bool;
    fn push_child(&mut self, parent_id: &str, child: UiChildMount) -> bool;
    fn swap_children(&mut self, parent_id: &str, left: usize, right: usize) -> bool;
}

impl UiAssetDocumentRuntimeExt for UiAssetDocument {
    fn validate_tree_authority(&self) -> Result<(), UiAssetError> {
        let mut seen = BTreeMap::new();
        if let Some(root) = &self.root {
            validate_node_tree(&self.asset.id, "root", root, &mut seen)?;
        }
        for (component_name, component) in &self.components {
            validate_node_tree(
                &self.asset.id,
                &format!("component {component_name}"),
                &component.root,
                &mut seen,
            )?;
        }
        validate_stylesheet_ids(&self.asset.id, &self.stylesheets)?;
        validate_style_rule_ids(&self.asset.id, &self.stylesheets)?;
        validate_style_rule_selectors(&self.stylesheets)?;
        Ok(())
    }

    fn root_node_id(&self) -> Option<&str> {
        self.root.as_ref().map(|node| node.node_id.as_str())
    }

    fn contains_node(&self, node_id: &str) -> bool {
        self.node(node_id).is_some()
    }

    fn node(&self, node_id: &str) -> Option<&UiNodeDefinition> {
        if let Some(root) = &self.root {
            if let Some(node) = find_node(root, node_id) {
                return Some(node);
            }
        }
        self.components
            .values()
            .find_map(|component| find_node(&component.root, node_id))
    }

    fn node_mut(&mut self, node_id: &str) -> Option<&mut UiNodeDefinition> {
        if let Some(root) = &mut self.root {
            if let Some(node) = find_node_mut(root, node_id) {
                return Some(node);
            }
        }
        self.components
            .values_mut()
            .find_map(|component| find_node_mut(&mut component.root, node_id))
    }

    fn style_rule(&self, rule_id: &str) -> Option<&UiStyleRule> {
        self.stylesheets
            .iter()
            .flat_map(|stylesheet| stylesheet.rules.iter())
            .find(|rule| rule.id.as_deref() == Some(rule_id))
    }

    fn style_rule_mut(&mut self, rule_id: &str) -> Option<&mut UiStyleRule> {
        self.stylesheets
            .iter_mut()
            .flat_map(|stylesheet| stylesheet.rules.iter_mut())
            .find(|rule| rule.id.as_deref() == Some(rule_id))
    }

    fn style_rule_position(&self, rule_id: &str) -> Option<UiStyleRulePosition<'_>> {
        self.stylesheets
            .iter()
            .enumerate()
            .find_map(|(stylesheet_index, stylesheet)| {
                stylesheet
                    .rules
                    .iter()
                    .position(|rule| rule.id.as_deref() == Some(rule_id))
                    .map(|rule_index| UiStyleRulePosition {
                        stylesheet_id: stylesheet.id.as_str(),
                        stylesheet_index,
                        rule_index,
                    })
            })
    }

    fn rename_style_rule(&mut self, current_id: &str, new_id: &str) -> Result<bool, UiAssetError> {
        let new_id = new_id.trim();
        if new_id.is_empty() {
            return Err(UiAssetError::InvalidDocument {
                asset_id: self.asset.id.clone(),
                detail: "style rule id cannot be empty".to_string(),
            });
        }

        let Some((stylesheet_index, rule_index)) =
            self.stylesheets
                .iter()
                .enumerate()
                .find_map(|(stylesheet_index, stylesheet)| {
                    stylesheet
                        .rules
                        .iter()
                        .position(|rule| rule.id.as_deref() == Some(current_id))
                        .map(|rule_index| (stylesheet_index, rule_index))
                })
        else {
            return Ok(false);
        };

        let duplicate = self
            .stylesheets
            .iter()
            .enumerate()
            .flat_map(|(candidate_stylesheet_index, stylesheet)| {
                stylesheet
                    .rules
                    .iter()
                    .enumerate()
                    .map(move |(candidate_rule_index, rule)| {
                        (candidate_stylesheet_index, candidate_rule_index, rule)
                    })
            })
            .any(|(candidate_stylesheet_index, candidate_rule_index, rule)| {
                (candidate_stylesheet_index, candidate_rule_index) != (stylesheet_index, rule_index)
                    && rule.id.as_deref() == Some(new_id)
            });
        if duplicate {
            return Err(UiAssetError::InvalidDocument {
                asset_id: self.asset.id.clone(),
                detail: format!("duplicate style rule id {new_id}"),
            });
        }

        self.stylesheets[stylesheet_index].rules[rule_index].id = Some(new_id.to_string());
        Ok(true)
    }

    fn remove_style_rule(&mut self, rule_id: &str) -> Option<UiStyleRule> {
        let (stylesheet_index, rule_index) =
            self.stylesheets
                .iter()
                .enumerate()
                .find_map(|(stylesheet_index, stylesheet)| {
                    stylesheet
                        .rules
                        .iter()
                        .position(|rule| rule.id.as_deref() == Some(rule_id))
                        .map(|rule_index| (stylesheet_index, rule_index))
                })?;
        Some(self.stylesheets[stylesheet_index].rules.remove(rule_index))
    }

    fn insert_style_rule(
        &mut self,
        stylesheet_id: &str,
        index: usize,
        rule: UiStyleRule,
    ) -> Result<bool, UiAssetError> {
        let Some(stylesheet_index) = self
            .stylesheets
            .iter()
            .position(|stylesheet| stylesheet.id == stylesheet_id)
        else {
            return Ok(false);
        };

        let mut stylesheets = self.stylesheets.clone();
        let insert_index = index.min(stylesheets[stylesheet_index].rules.len());
        stylesheets[stylesheet_index]
            .rules
            .insert(insert_index, rule);
        validate_style_rule_ids(&self.asset.id, &stylesheets)?;
        validate_style_rule_selectors(&stylesheets)?;
        self.stylesheets = stylesheets;
        Ok(true)
    }

    fn replace_style_rule(
        &mut self,
        rule_id: &str,
        replacement: UiStyleRule,
    ) -> Result<Option<UiStyleRule>, UiAssetError> {
        let Some((stylesheet_index, rule_index)) =
            self.stylesheets
                .iter()
                .enumerate()
                .find_map(|(stylesheet_index, stylesheet)| {
                    stylesheet
                        .rules
                        .iter()
                        .position(|rule| rule.id.as_deref() == Some(rule_id))
                        .map(|rule_index| (stylesheet_index, rule_index))
                })
        else {
            return Ok(None);
        };

        let mut stylesheets = self.stylesheets.clone();
        let previous = std::mem::replace(
            &mut stylesheets[stylesheet_index].rules[rule_index],
            replacement,
        );
        validate_style_rule_ids(&self.asset.id, &stylesheets)?;
        validate_style_rule_selectors(&stylesheets)?;
        self.stylesheets = stylesheets;
        Ok(Some(previous))
    }

    fn move_style_rule(
        &mut self,
        rule_id: &str,
        target_stylesheet_id: &str,
        target_index: usize,
    ) -> Result<bool, UiAssetError> {
        let Some((source_stylesheet_index, source_rule_index)) = self
            .stylesheets
            .iter()
            .enumerate()
            .find_map(|(stylesheet_index, stylesheet)| {
                stylesheet
                    .rules
                    .iter()
                    .position(|rule| rule.id.as_deref() == Some(rule_id))
                    .map(|rule_index| (stylesheet_index, rule_index))
            })
        else {
            return Ok(false);
        };
        if self.style_sheet(target_stylesheet_id).is_none() {
            return Ok(false);
        }

        let mut stylesheets = self.stylesheets.clone();
        let rule = stylesheets[source_stylesheet_index]
            .rules
            .remove(source_rule_index);
        let target_stylesheet_index = stylesheets
            .iter()
            .position(|stylesheet| stylesheet.id == target_stylesheet_id)
            .expect("target stylesheet checked above");
        let insert_index = target_index.min(stylesheets[target_stylesheet_index].rules.len());
        stylesheets[target_stylesheet_index]
            .rules
            .insert(insert_index, rule);
        validate_style_rule_ids(&self.asset.id, &stylesheets)?;
        validate_style_rule_selectors(&stylesheets)?;
        self.stylesheets = stylesheets;
        Ok(true)
    }

    fn style_sheet(&self, stylesheet_id: &str) -> Option<&UiStyleSheet> {
        self.stylesheets
            .iter()
            .find(|stylesheet| stylesheet.id == stylesheet_id)
    }

    fn style_sheet_mut(&mut self, stylesheet_id: &str) -> Option<&mut UiStyleSheet> {
        self.stylesheets
            .iter_mut()
            .find(|stylesheet| stylesheet.id == stylesheet_id)
    }

    fn style_sheet_index(&self, stylesheet_id: &str) -> Option<usize> {
        self.stylesheets
            .iter()
            .position(|stylesheet| stylesheet.id == stylesheet_id)
    }

    fn rename_style_sheet(&mut self, current_id: &str, new_id: &str) -> Result<bool, UiAssetError> {
        let new_id = new_id.trim();
        if new_id.is_empty() {
            return Err(UiAssetError::InvalidDocument {
                asset_id: self.asset.id.clone(),
                detail: "stylesheet id cannot be empty".to_string(),
            });
        }

        let Some(stylesheet_index) = self
            .stylesheets
            .iter()
            .position(|stylesheet| stylesheet.id == current_id)
        else {
            return Ok(false);
        };

        if self
            .stylesheets
            .iter()
            .enumerate()
            .any(|(candidate_index, stylesheet)| {
                candidate_index != stylesheet_index && stylesheet.id == new_id
            })
        {
            return Err(UiAssetError::InvalidDocument {
                asset_id: self.asset.id.clone(),
                detail: format!("duplicate stylesheet id {new_id}"),
            });
        }

        self.stylesheets[stylesheet_index].id = new_id.to_string();
        Ok(true)
    }

    fn remove_style_sheet(&mut self, stylesheet_id: &str) -> Option<UiStyleSheet> {
        let stylesheet_index = self
            .stylesheets
            .iter()
            .position(|stylesheet| stylesheet.id == stylesheet_id)?;
        Some(self.stylesheets.remove(stylesheet_index))
    }

    fn set_style_sheets(&mut self, stylesheets: Vec<UiStyleSheet>) -> Result<bool, UiAssetError> {
        if self.stylesheets == stylesheets {
            return Ok(false);
        }
        validate_stylesheet_ids(&self.asset.id, &stylesheets)?;
        validate_style_rule_ids(&self.asset.id, &stylesheets)?;
        validate_style_rule_selectors(&stylesheets)?;
        self.stylesheets = stylesheets;
        Ok(true)
    }

    fn insert_style_sheet(
        &mut self,
        index: usize,
        stylesheet: UiStyleSheet,
    ) -> Result<usize, UiAssetError> {
        let mut stylesheets = self.stylesheets.clone();
        let insert_index = index.min(stylesheets.len());
        stylesheets.insert(insert_index, stylesheet);
        validate_stylesheet_ids(&self.asset.id, &stylesheets)?;
        validate_style_rule_ids(&self.asset.id, &stylesheets)?;
        validate_style_rule_selectors(&stylesheets)?;
        self.stylesheets = stylesheets;
        Ok(insert_index)
    }

    fn replace_style_sheet(
        &mut self,
        stylesheet_id: &str,
        replacement: UiStyleSheet,
    ) -> Result<Option<UiStyleSheet>, UiAssetError> {
        let Some(stylesheet_index) = self
            .stylesheets
            .iter()
            .position(|stylesheet| stylesheet.id == stylesheet_id)
        else {
            return Ok(None);
        };

        let mut stylesheets = self.stylesheets.clone();
        let previous = std::mem::replace(&mut stylesheets[stylesheet_index], replacement);
        validate_stylesheet_ids(&self.asset.id, &stylesheets)?;
        validate_style_rule_ids(&self.asset.id, &stylesheets)?;
        validate_style_rule_selectors(&stylesheets)?;
        self.stylesheets = stylesheets;
        Ok(Some(previous))
    }

    fn move_style_sheet(&mut self, stylesheet_id: &str, target_index: usize) -> Option<usize> {
        let source_index = self
            .stylesheets
            .iter()
            .position(|stylesheet| stylesheet.id == stylesheet_id)?;
        let stylesheet = self.stylesheets.remove(source_index);
        let insert_index = target_index.min(self.stylesheets.len());
        self.stylesheets.insert(insert_index, stylesheet);
        Some(insert_index)
    }

    fn component_root_node_id(&self, component_name: &str) -> Option<&str> {
        self.components
            .get(component_name)
            .map(|component| component.root.node_id.as_str())
    }

    fn child_index_in_parent(&self, child_id: &str) -> Option<(String, usize)> {
        self.parent_of(child_id)
            .map(|parent| (parent.parent_node_id.to_string(), parent.child_index))
    }

    fn child_mount(&self, child_id: &str) -> Option<&UiChildMount> {
        if let Some(root) = &self.root {
            if let Some(mount) = find_child_mount(root, child_id) {
                return Some(mount);
            }
        }
        self.components
            .values()
            .find_map(|component| find_child_mount(&component.root, child_id))
    }

    fn child_mount_mut(&mut self, child_id: &str) -> Option<&mut UiChildMount> {
        if let Some(root) = &mut self.root {
            if let Some(mount) = find_child_mount_mut(root, child_id) {
                return Some(mount);
            }
        }
        self.components
            .values_mut()
            .find_map(|component| find_child_mount_mut(&mut component.root, child_id))
    }

    fn parent_of(&self, child_id: &str) -> Option<UiNodeParent<'_>> {
        if let Some(root) = &self.root {
            if let Some(parent) = find_parent(root, child_id) {
                return Some(parent);
            }
        }
        self.components
            .values()
            .find_map(|component| find_parent(&component.root, child_id))
    }

    fn iter_nodes(&self) -> UiAssetNodeIter<'_> {
        let mut stack = Vec::new();
        for component in self.components.values().rev() {
            stack.push(&component.root);
        }
        if let Some(root) = &self.root {
            stack.push(root);
        }
        UiAssetNodeIter { stack }
    }

    fn node_map(&self) -> BTreeMap<String, UiNodeDefinition> {
        self.iter_nodes()
            .map(|node| (node.node_id.clone(), node.clone()))
            .collect()
    }

    fn replace_node(&mut self, node_id: &str, replacement: UiNodeDefinition) -> bool {
        if self
            .root
            .as_ref()
            .is_some_and(|root| root.node_id == node_id)
        {
            if self.root.as_ref() == Some(&replacement) {
                return false;
            }
            self.root = Some(replacement);
            return true;
        }
        if let Some(root) = &mut self.root {
            if replace_node_in_tree(root, node_id, &replacement) {
                return true;
            }
        }
        for component in self.components.values_mut() {
            if component.root.node_id == node_id {
                if component.root == replacement {
                    return false;
                }
                component.root = replacement;
                return true;
            }
            if replace_node_in_tree(&mut component.root, node_id, &replacement) {
                return true;
            }
        }
        false
    }

    fn remove_node(&mut self, node_id: &str) -> Option<UiNodeDefinition> {
        if self
            .root
            .as_ref()
            .is_some_and(|root| root.node_id == node_id)
        {
            return self.root.take();
        }
        if let Some(root) = &mut self.root {
            if let Some(removed) = remove_node_from_tree(root, node_id) {
                return Some(removed);
            }
        }
        for component in self.components.values_mut() {
            if let Some(removed) = remove_node_from_tree(&mut component.root, node_id) {
                return Some(removed);
            }
        }
        None
    }

    fn insert_child(&mut self, parent_id: &str, index: usize, child: UiChildMount) -> bool {
        let Some(parent) = self.node_mut(parent_id) else {
            return false;
        };
        let insert_index = index.min(parent.children.len());
        parent.children.insert(insert_index, child);
        true
    }

    fn push_child(&mut self, parent_id: &str, child: UiChildMount) -> bool {
        let Some(parent) = self.node_mut(parent_id) else {
            return false;
        };
        parent.children.push(child);
        true
    }

    fn swap_children(&mut self, parent_id: &str, left: usize, right: usize) -> bool {
        let Some(parent) = self.node_mut(parent_id) else {
            return false;
        };
        if left >= parent.children.len() || right >= parent.children.len() || left == right {
            return false;
        }
        parent.children.swap(left, right);
        true
    }
}

pub struct UiAssetNodeIter<'a> {
    stack: Vec<&'a UiNodeDefinition>,
}

impl<'a> Iterator for UiAssetNodeIter<'a> {
    type Item = &'a UiNodeDefinition;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        for child in node.children.iter().rev() {
            self.stack.push(&child.node);
        }
        Some(node)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiNodeParent<'a> {
    pub parent_node_id: &'a str,
    pub child_index: usize,
    pub mount: Option<&'a str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiStyleRulePosition<'a> {
    pub stylesheet_id: &'a str,
    pub stylesheet_index: usize,
    pub rule_index: usize,
}

fn validate_node_tree(
    asset_id: &str,
    scope: &str,
    node: &UiNodeDefinition,
    seen: &mut BTreeMap<String, UiNodeDefinition>,
) -> Result<(), UiAssetError> {
    if node.node_id.trim().is_empty() {
        return Err(UiAssetError::InvalidDocument {
            asset_id: asset_id.to_string(),
            detail: format!("{scope} contains a node with an empty node_id"),
        });
    }
    if let Some(existing) = seen.get(&node.node_id) {
        if existing == node {
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
    let _ = seen.insert(node.node_id.clone(), node.clone());
    for child in &node.children {
        validate_node_tree(asset_id, scope, &child.node, seen)?;
    }
    Ok(())
}

fn validate_stylesheet_ids(
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

fn validate_style_rule_ids(
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

fn validate_style_rule_selectors(stylesheets: &[UiStyleSheet]) -> Result<(), UiAssetError> {
    for stylesheet in stylesheets {
        for rule in &stylesheet.rules {
            UiSelector::parse(&rule.selector)?;
        }
    }
    Ok(())
}

fn find_node<'a>(node: &'a UiNodeDefinition, target: &str) -> Option<&'a UiNodeDefinition> {
    if node.node_id == target {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_node(&child.node, target))
}

fn find_node_mut<'a>(
    node: &'a mut UiNodeDefinition,
    target: &str,
) -> Option<&'a mut UiNodeDefinition> {
    if node.node_id == target {
        return Some(node);
    }
    for child in &mut node.children {
        if let Some(found) = find_node_mut(&mut child.node, target) {
            return Some(found);
        }
    }
    None
}

fn find_child_mount<'a>(node: &'a UiNodeDefinition, child_id: &str) -> Option<&'a UiChildMount> {
    for child in &node.children {
        if child.node.node_id == child_id {
            return Some(child);
        }
        if let Some(found) = find_child_mount(&child.node, child_id) {
            return Some(found);
        }
    }
    None
}

fn find_child_mount_mut<'a>(
    node: &'a mut UiNodeDefinition,
    child_id: &str,
) -> Option<&'a mut UiChildMount> {
    for child in &mut node.children {
        if child.node.node_id == child_id {
            return Some(child);
        }
        if let Some(found) = find_child_mount_mut(&mut child.node, child_id) {
            return Some(found);
        }
    }
    None
}

fn find_parent<'a>(node: &'a UiNodeDefinition, child_id: &str) -> Option<UiNodeParent<'a>> {
    for (index, child) in node.children.iter().enumerate() {
        if child.node.node_id == child_id {
            return Some(UiNodeParent {
                parent_node_id: node.node_id.as_str(),
                child_index: index,
                mount: child.mount.as_deref(),
            });
        }
        if let Some(parent) = find_parent(&child.node, child_id) {
            return Some(parent);
        }
    }
    None
}

fn replace_node_in_tree(
    node: &mut UiNodeDefinition,
    target: &str,
    replacement: &UiNodeDefinition,
) -> bool {
    for child in &mut node.children {
        if child.node.node_id == target {
            if child.node == *replacement {
                return false;
            }
            child.node = replacement.clone();
            return true;
        }
        if replace_node_in_tree(&mut child.node, target, replacement) {
            return true;
        }
    }
    false
}

fn remove_node_from_tree(node: &mut UiNodeDefinition, target: &str) -> Option<UiNodeDefinition> {
    if let Some(index) = node
        .children
        .iter()
        .position(|child| child.node.node_id == target)
    {
        return Some(node.children.remove(index).node);
    }
    for child in &mut node.children {
        if let Some(removed) = remove_node_from_tree(&mut child.node, target) {
            return Some(removed);
        }
    }
    None
}
