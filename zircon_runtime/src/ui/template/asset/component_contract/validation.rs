use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiComponentApiVersion, UiComponentContractDiagnostic,
    UiComponentContractDiagnosticCode, UiComponentDefinition, UiNodeDefinition, UiRootClassPolicy,
    UiSelector, UiSelectorToken, UiStyleSheet,
};

pub(crate) fn validate_document_component_contracts(
    document: &UiAssetDocument,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
    style_imports: &BTreeMap<String, UiAssetDocument>,
) -> Result<(), UiAssetError> {
    if let Some(diagnostic) =
        component_contract_diagnostic(document, widget_imports, style_imports)?
    {
        return Err(diagnostic.into_asset_error(document.asset.id.clone()));
    }
    Ok(())
}

pub fn component_contract_diagnostic(
    document: &UiAssetDocument,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
    style_imports: &BTreeMap<String, UiAssetDocument>,
) -> Result<Option<UiComponentContractDiagnostic>, UiAssetError> {
    for (component_name, component) in &document.components {
        if let Some(diagnostic) = validate_component_contract(component_name, component) {
            return Ok(Some(diagnostic));
        }
    }
    if let Some(diagnostic) = validate_component_instances(document, widget_imports)? {
        return Ok(Some(diagnostic));
    }
    validate_reference_privacy(document, widget_imports, style_imports)
}

fn validate_component_contract(
    component_name: &str,
    component: &UiComponentDefinition,
) -> Option<UiComponentContractDiagnostic> {
    let tree = ComponentTreeIndex::new(&component.root);
    for (part_name, part) in &component.contract.public_parts {
        if part_name.trim().is_empty() {
            return Some(invalid_contract(
                component_name,
                UiComponentContractDiagnosticCode::InvalidPublicPart,
                "public part names cannot be empty",
                format!("components.{component_name}.contract.public_parts"),
                None,
            ));
        }
        if part.node_id.trim().is_empty() || !tree.node_ids.contains(part.node_id.as_str()) {
            return Some(invalid_contract(
                component_name,
                UiComponentContractDiagnosticCode::InvalidPublicPart,
                &format!(
                    "public part {part_name} references missing node {}",
                    part.node_id
                ),
                format!("components.{component_name}.contract.public_parts.{part_name}.node_id"),
                Some(TargetRef::node(part.node_id.clone())),
            ));
        }
        if let Some(control_id) = &part.control_id {
            match tree.node_control_ids.get(part.node_id.as_str()).copied() {
                Some(actual) if actual == control_id.as_str() => {}
                Some(_) => {
                    return Some(invalid_contract(
                        component_name,
                        UiComponentContractDiagnosticCode::InvalidPublicPart,
                        &format!(
                            "public part {part_name} control {control_id} does not belong to node {}",
                            part.node_id
                        ),
                        format!(
                            "components.{component_name}.contract.public_parts.{part_name}.control_id"
                        ),
                        Some(TargetRef::node_and_control(
                            part.node_id.clone(),
                            control_id.clone(),
                        )),
                    ));
                }
                None if tree.control_ids.contains(control_id.as_str()) => {
                    return Some(invalid_contract(
                        component_name,
                        UiComponentContractDiagnosticCode::InvalidPublicPart,
                        &format!(
                            "public part {part_name} control {control_id} does not belong to node {}",
                            part.node_id
                        ),
                        format!(
                            "components.{component_name}.contract.public_parts.{part_name}.control_id"
                        ),
                        Some(TargetRef::node_and_control(
                            part.node_id.clone(),
                            control_id.clone(),
                        )),
                    ));
                }
                None => {
                    return Some(invalid_contract(
                        component_name,
                        UiComponentContractDiagnosticCode::InvalidPublicPart,
                        &format!("public part {part_name} references missing control {control_id}"),
                        format!(
                            "components.{component_name}.contract.public_parts.{part_name}.control_id"
                        ),
                        Some(TargetRef::control(control_id.clone())),
                    ));
                }
            }
        }
    }

    if let Some(initial_focus) = &component.contract.focus.initial_focus {
        if let Some(diagnostic) = validate_public_target(
            component_name,
            component,
            &tree,
            initial_focus,
            UiComponentContractDiagnosticCode::PrivateFocusTarget,
            format!("components.{component_name}.contract.focus.initial_focus"),
        ) {
            return Some(diagnostic);
        }
    }
    for (target_name, target) in &component.contract.focus.public_targets {
        if let Some(diagnostic) = validate_public_target(
            component_name,
            component,
            &tree,
            target,
            UiComponentContractDiagnosticCode::PrivateFocusTarget,
            format!("components.{component_name}.contract.focus.public_targets.{target_name}"),
        ) {
            return Some(diagnostic);
        }
    }
    for (action_name, route) in &component.contract.bindings.public_actions {
        if let Some(target) = &route.target {
            if let Some(diagnostic) = validate_public_target(
                component_name,
                component,
                &tree,
                target,
                UiComponentContractDiagnosticCode::PrivateBindingTarget,
                format!(
                    "components.{component_name}.contract.bindings.public_actions.{action_name}.target"
                ),
            ) {
                return Some(diagnostic);
            }
        }
    }
    None
}

fn validate_component_instances(
    document: &UiAssetDocument,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
) -> Result<Option<UiComponentContractDiagnostic>, UiAssetError> {
    if let Some(root) = &document.root {
        if let Some(diagnostic) =
            validate_component_instances_from_node(document, widget_imports, root)?
        {
            return Ok(Some(diagnostic));
        }
    }
    for component in document.components.values() {
        if let Some(diagnostic) =
            validate_component_instances_from_node(document, widget_imports, &component.root)?
        {
            return Ok(Some(diagnostic));
        }
    }
    Ok(None)
}

fn validate_component_instances_from_node(
    document: &UiAssetDocument,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
    node: &UiNodeDefinition,
) -> Result<Option<UiComponentContractDiagnostic>, UiAssetError> {
    if let Some(component_name) = &node.component {
        let component = document.components.get(component_name).ok_or_else(|| {
            UiAssetError::UnknownComponent {
                asset_id: document.asset.id.clone(),
                component: component_name.clone(),
            }
        })?;
        if let Some(diagnostic) = validate_instance_contract(component_name, component, node) {
            return Ok(Some(diagnostic));
        }
    }

    if let Some(reference) = &node.component_ref {
        if let Some((_, component_name)) = reference.split_once('#') {
            let imported =
                widget_imports
                    .get(reference)
                    .ok_or_else(|| UiAssetError::UnknownImport {
                        reference: reference.clone(),
                    })?;
            let component = imported.components.get(component_name).ok_or_else(|| {
                UiAssetError::UnknownComponent {
                    asset_id: imported.asset.id.clone(),
                    component: component_name.to_string(),
                }
            })?;
            if let Some(diagnostic) = validate_instance_contract(component_name, component, node) {
                return Ok(Some(diagnostic));
            }
        }
    }

    for child in &node.children {
        if let Some(diagnostic) =
            validate_component_instances_from_node(document, widget_imports, &child.node)?
        {
            return Ok(Some(diagnostic));
        }
    }
    Ok(None)
}

fn validate_instance_contract(
    component_name: &str,
    component: &UiComponentDefinition,
    instance_node: &UiNodeDefinition,
) -> Option<UiComponentContractDiagnostic> {
    if matches!(
        component.contract.root_class_policy,
        UiRootClassPolicy::Closed
    ) && !instance_node.classes.is_empty()
    {
        return Some(
            UiComponentContractDiagnostic::new(
                UiComponentContractDiagnosticCode::ClosedRootClass,
                format!(
                    "component {component_name} root class policy is closed; instance {} cannot append root classes",
                    instance_node.node_id
                ),
                format!("nodes.{}.classes", instance_node.node_id),
            )
            .with_target_node_id(instance_node.node_id.clone()),
        );
    }

    if let Some(required) = instance_node.component_api_version {
        let actual = component.contract.api_version;
        if !actual.is_compatible_with(required) {
            return Some(
                UiComponentContractDiagnostic::new(
                    UiComponentContractDiagnosticCode::ApiMismatch,
                    format!(
                        "component instance {} requires component API {required}, but {component_name} exports {actual}",
                        instance_node.node_id
                    ),
                    format!("nodes.{}.component_api_version", instance_node.node_id),
                )
                .with_target_node_id(instance_node.node_id.clone()),
            );
        }
    }
    None
}

fn validate_reference_privacy(
    document: &UiAssetDocument,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
    style_imports: &BTreeMap<String, UiAssetDocument>,
) -> Result<Option<UiComponentContractDiagnostic>, UiAssetError> {
    let referenced = collect_reference_components(document);
    if referenced.is_empty() {
        return Ok(None);
    }

    let referenced_component_names = referenced
        .iter()
        .map(|reference| reference.component_name.as_str())
        .collect();
    let selector_targets =
        collect_document_selector_targets(document, style_imports, &referenced_component_names)?;
    for reference in referenced {
        let imported = widget_imports.get(&reference.reference).ok_or_else(|| {
            UiAssetError::UnknownImport {
                reference: reference.reference.clone(),
            }
        })?;
        let component = imported
            .components
            .get(&reference.component_name)
            .ok_or_else(|| UiAssetError::UnknownComponent {
                asset_id: imported.asset.id.clone(),
                component: reference.component_name.clone(),
            })?;
        if let Some(diagnostic) = validate_component_contract(&reference.component_name, component)
        {
            return Ok(Some(diagnostic));
        }
        if let Some(required) = reference.required_api_version {
            let actual = component.contract.api_version;
            if !actual.is_compatible_with(required) {
                return Ok(Some(
                    UiComponentContractDiagnostic::new(
                        UiComponentContractDiagnosticCode::ApiMismatch,
                        format!(
                            "reference {} requires component API {required}, but {} exports {actual}",
                            reference.reference, reference.component_name
                        ),
                        reference.path.clone(),
                    )
                    .with_target_node_id(reference.node_id.clone()),
                ));
            }
        }

        let privacy = ComponentPrivacyIndex::new(component);
        for id in &selector_targets.ids {
            if id.applies_to(&reference.component_name)
                && privacy.private_targets.contains(id.name.as_str())
            {
                return Ok(Some(
                    UiComponentContractDiagnostic::new(
                        UiComponentContractDiagnosticCode::PrivateSelector,
                        format!(
                            "selector targets private component internals {} of {}",
                            id.name, reference.reference
                        ),
                        id.path.clone(),
                    )
                    .with_target_control_id(id.name.clone()),
                ));
            }
        }
        for part in &selector_targets.parts {
            if part.applies_to(&reference.component_name)
                && !component.contract.public_parts.contains_key(&part.name)
            {
                return Ok(Some(UiComponentContractDiagnostic::new(
                    UiComponentContractDiagnosticCode::PrivateSelector,
                    format!(
                        "selector references unknown public part {} of {}",
                        part.name, reference.reference
                    ),
                    part.path.clone(),
                )));
            }
        }
    }
    Ok(None)
}

fn validate_public_target(
    component_name: &str,
    component: &UiComponentDefinition,
    tree: &ComponentTreeIndex<'_>,
    target: &str,
    code: UiComponentContractDiagnosticCode,
    path: String,
) -> Option<UiComponentContractDiagnostic> {
    if target == component.root.node_id
        || component.root.control_id.as_deref() == Some(target)
        || component.contract.public_parts.contains_key(target)
        || component
            .contract
            .public_parts
            .values()
            .any(|part| part.node_id == target || part.control_id.as_deref() == Some(target))
    {
        return None;
    }

    if tree.node_ids.contains(target) || tree.control_ids.contains(target) {
        let target_ref = tree.target_ref(target);
        return Some(invalid_contract(
            component_name,
            code,
            &format!("target {target} is private and must be exported as a public part"),
            path,
            Some(target_ref),
        ));
    }

    Some(invalid_contract(
        component_name,
        code,
        &format!("target {target} does not exist in the component tree"),
        path,
        Some(TargetRef::node(target.to_string())),
    ))
}

fn collect_reference_components(document: &UiAssetDocument) -> Vec<ReferenceComponent> {
    let mut references = Vec::new();
    if let Some(root) = &document.root {
        collect_reference_components_from_node(root, &mut references);
    }
    for component in document.components.values() {
        collect_reference_components_from_node(&component.root, &mut references);
    }
    references
}

fn collect_reference_components_from_node(
    node: &UiNodeDefinition,
    references: &mut Vec<ReferenceComponent>,
) {
    if let Some(reference) = &node.component_ref {
        if let Some((_, component_name)) = reference.split_once('#') {
            references.push(ReferenceComponent {
                reference: reference.clone(),
                component_name: component_name.to_string(),
                required_api_version: node.component_api_version,
                node_id: node.node_id.clone(),
                path: format!("nodes.{}.component_ref", node.node_id),
            });
        }
    }
    for child in &node.children {
        collect_reference_components_from_node(&child.node, references);
    }
}

fn collect_document_selector_targets(
    document: &UiAssetDocument,
    style_imports: &BTreeMap<String, UiAssetDocument>,
    referenced_component_names: &BTreeSet<&str>,
) -> Result<SelectorTargetSet, UiAssetError> {
    let mut targets = SelectorTargetSet::default();
    collect_selector_targets_from_stylesheets(
        &document.stylesheets,
        &mut targets,
        referenced_component_names,
    )?;
    for reference in &document.imports.styles {
        let imported = style_imports
            .get(reference)
            .ok_or_else(|| UiAssetError::UnknownImport {
                reference: reference.clone(),
            })?;
        collect_selector_targets_from_stylesheets(
            &imported.stylesheets,
            &mut targets,
            referenced_component_names,
        )?;
    }
    Ok(targets)
}

fn collect_selector_targets_from_stylesheets(
    stylesheets: &[UiStyleSheet],
    targets: &mut SelectorTargetSet,
    referenced_component_names: &BTreeSet<&str>,
) -> Result<(), UiAssetError> {
    for stylesheet in stylesheets {
        for (rule_index, rule) in stylesheet.rules.iter().enumerate() {
            let selector = UiSelector::parse(&rule.selector)?;
            let component_scopes = selector_component_scopes(&selector, referenced_component_names);
            let path = rule
                .id
                .as_ref()
                .map(|rule_id| format!("stylesheets.{}.rules.{rule_id}.selector", stylesheet.id))
                .unwrap_or_else(|| {
                    format!("stylesheets.{}.rules.{rule_index}.selector", stylesheet.id)
                });
            for token in selector
                .segments
                .iter()
                .flat_map(|segment| segment.tokens.iter())
            {
                match token {
                    UiSelectorToken::Id(id) => {
                        targets.ids.push(ScopedSelectorTarget::new(
                            id.clone(),
                            path.clone(),
                            &component_scopes,
                        ));
                    }
                    UiSelectorToken::Part(part) => {
                        targets.parts.push(ScopedSelectorTarget::new(
                            part.clone(),
                            path.clone(),
                            &component_scopes,
                        ));
                    }
                    UiSelectorToken::Type(_)
                    | UiSelectorToken::Class(_)
                    | UiSelectorToken::State(_)
                    | UiSelectorToken::Host => {}
                }
            }
        }
    }
    Ok(())
}

fn selector_component_scopes(
    selector: &UiSelector,
    referenced_component_names: &BTreeSet<&str>,
) -> BTreeSet<String> {
    selector
        .segments
        .iter()
        .flat_map(|segment| segment.tokens.iter())
        .filter_map(|token| match token {
            UiSelectorToken::Type(component)
                if referenced_component_names.contains(component.as_str()) =>
            {
                Some(component.clone())
            }
            _ => None,
        })
        .collect()
}

fn invalid_contract(
    component_name: &str,
    code: UiComponentContractDiagnosticCode,
    detail: &str,
    path: String,
    target: Option<TargetRef>,
) -> UiComponentContractDiagnostic {
    let mut diagnostic = UiComponentContractDiagnostic::new(
        code,
        format!("component {component_name} public contract is invalid: {detail}"),
        path,
    );
    if let Some(target) = target {
        if let Some(node_id) = target.node_id {
            diagnostic = diagnostic.with_target_node_id(node_id);
        }
        if let Some(control_id) = target.control_id {
            diagnostic = diagnostic.with_target_control_id(control_id);
        }
    }
    diagnostic
}

#[derive(Clone)]
struct ReferenceComponent {
    reference: String,
    component_name: String,
    required_api_version: Option<UiComponentApiVersion>,
    node_id: String,
    path: String,
}

#[derive(Default)]
struct SelectorTargetSet {
    ids: Vec<ScopedSelectorTarget>,
    parts: Vec<ScopedSelectorTarget>,
}

struct ScopedSelectorTarget {
    name: String,
    path: String,
    component_scopes: BTreeSet<String>,
}

impl ScopedSelectorTarget {
    fn new(name: String, path: String, component_scopes: &BTreeSet<String>) -> Self {
        Self {
            name,
            path,
            component_scopes: component_scopes.clone(),
        }
    }

    fn applies_to(&self, component_name: &str) -> bool {
        self.component_scopes.is_empty() || self.component_scopes.contains(component_name)
    }
}

struct ComponentTreeIndex<'a> {
    node_ids: BTreeSet<&'a str>,
    control_ids: BTreeSet<&'a str>,
    node_control_ids: BTreeMap<&'a str, &'a str>,
}

impl<'a> ComponentTreeIndex<'a> {
    fn new(root: &'a UiNodeDefinition) -> Self {
        let mut index = Self {
            node_ids: BTreeSet::new(),
            control_ids: BTreeSet::new(),
            node_control_ids: BTreeMap::new(),
        };
        index.visit(root);
        index
    }

    fn visit(&mut self, node: &'a UiNodeDefinition) {
        let _ = self.node_ids.insert(node.node_id.as_str());
        if let Some(control_id) = &node.control_id {
            let _ = self.control_ids.insert(control_id.as_str());
            let _ = self
                .node_control_ids
                .insert(node.node_id.as_str(), control_id.as_str());
        }
        for child in &node.children {
            self.visit(&child.node);
        }
    }

    fn target_ref(&self, target: &str) -> TargetRef {
        if self.control_ids.contains(target) {
            TargetRef::control(target.to_string())
        } else {
            TargetRef::node(target.to_string())
        }
    }
}

struct ComponentPrivacyIndex<'a> {
    private_targets: BTreeSet<&'a str>,
}

impl<'a> ComponentPrivacyIndex<'a> {
    fn new(component: &'a UiComponentDefinition) -> Self {
        let tree = ComponentTreeIndex::new(&component.root);
        let mut public_targets = BTreeSet::new();
        let _ = public_targets.insert(component.root.node_id.as_str());
        if let Some(control_id) = &component.root.control_id {
            let _ = public_targets.insert(control_id.as_str());
        }
        for part in component.contract.public_parts.values() {
            let _ = public_targets.insert(part.node_id.as_str());
            if let Some(control_id) = &part.control_id {
                let _ = public_targets.insert(control_id.as_str());
            }
        }

        let private_targets = tree
            .node_ids
            .into_iter()
            .chain(tree.control_ids)
            .filter(|target| !public_targets.contains(*target))
            .collect();
        Self { private_targets }
    }
}

struct TargetRef {
    node_id: Option<String>,
    control_id: Option<String>,
}

impl TargetRef {
    fn node(node_id: String) -> Self {
        Self {
            node_id: Some(node_id),
            control_id: None,
        }
    }

    fn control(control_id: String) -> Self {
        Self {
            node_id: None,
            control_id: Some(control_id),
        }
    }

    fn node_and_control(node_id: String, control_id: String) -> Self {
        Self {
            node_id: Some(node_id),
            control_id: Some(control_id),
        }
    }
}
