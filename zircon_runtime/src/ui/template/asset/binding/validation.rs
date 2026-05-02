use std::collections::BTreeMap;

use toml::Value;

use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{UiComponentDescriptor, UiValue, UiValueKind};
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiBindingDiagnostic, UiBindingDiagnosticCode,
    UiBindingDiagnosticSeverity, UiBindingExpression, UiBindingExpressionParseError, UiBindingRef,
    UiBindingReport, UiBindingTarget, UiBindingTargetAssignment, UiBindingTargetKind,
    UiComponentParamSchema, UiNodeDefinition,
};

pub fn collect_asset_binding_report(
    document: &UiAssetDocument,
    registry: &UiComponentDescriptorRegistry,
) -> UiBindingReport {
    let mut context = ValidationContext {
        registry,
        report: UiBindingReport::default(),
    };
    if let Some(root) = &document.root {
        context.validate_node("root", root, &BTreeMap::new());
    }
    for (component_name, component) in &document.components {
        context.validate_node(
            &format!("components.{component_name}.root"),
            &component.root,
            &component.params,
        );
    }
    context.report
}

pub fn validate_asset_bindings(
    document: &UiAssetDocument,
    registry: &UiComponentDescriptorRegistry,
) -> Result<(), UiAssetError> {
    let report = collect_asset_binding_report(document, registry);
    if let Some(diagnostic) = report.first_error() {
        return Err(UiAssetError::InvalidDocument {
            asset_id: document.asset.id.clone(),
            detail: diagnostic.message.clone(),
        });
    }
    Ok(())
}

struct ValidationContext<'a> {
    registry: &'a UiComponentDescriptorRegistry,
    report: UiBindingReport,
}

impl<'a> ValidationContext<'a> {
    fn validate_node(
        &mut self,
        path: &str,
        node: &UiNodeDefinition,
        params: &BTreeMap<String, UiComponentParamSchema>,
    ) {
        let descriptor = node
            .widget_type
            .as_deref()
            .and_then(|component_id| self.registry.descriptor(component_id));
        for (binding_index, binding) in node.bindings.iter().enumerate() {
            self.validate_binding(
                &format!("{path}.bindings[{binding_index}]"),
                node,
                descriptor,
                params,
                binding,
            );
        }
        for (child_index, child) in node.children.iter().enumerate() {
            self.validate_node(
                &format!("{path}.children[{child_index}].node"),
                &child.node,
                params,
            );
        }
    }

    fn validate_binding(
        &mut self,
        path: &str,
        node: &UiNodeDefinition,
        descriptor: Option<&UiComponentDescriptor>,
        params: &BTreeMap<String, UiComponentParamSchema>,
        binding: &UiBindingRef,
    ) {
        for (target_index, assignment) in binding.targets.iter().enumerate() {
            self.validate_assignment(
                &format!("{path}.targets[{target_index}]"),
                node,
                descriptor,
                params,
                binding,
                assignment,
            );
        }
        if let Some(action) = &binding.action {
            for (key, value) in &action.payload {
                self.validate_payload_expression(
                    &format!("{path}.action.payload.{key}"),
                    node,
                    descriptor,
                    params,
                    binding,
                    key,
                    value,
                );
            }
        }
    }

    fn validate_assignment(
        &mut self,
        path: &str,
        node: &UiNodeDefinition,
        descriptor: Option<&UiComponentDescriptor>,
        params: &BTreeMap<String, UiComponentParamSchema>,
        binding: &UiBindingRef,
        assignment: &UiBindingTargetAssignment,
    ) {
        let Some(expected_kind) =
            self.expected_kind_for_target(path, node, descriptor, binding, &assignment.target)
        else {
            return;
        };
        self.validate_expression_text(
            &format!("{path}.expression"),
            node,
            descriptor,
            params,
            binding,
            &assignment.expression,
            expected_kind,
        );
    }

    fn validate_payload_expression(
        &mut self,
        path: &str,
        node: &UiNodeDefinition,
        descriptor: Option<&UiComponentDescriptor>,
        params: &BTreeMap<String, UiComponentParamSchema>,
        binding: &UiBindingRef,
        payload_key: &str,
        value: &Value,
    ) {
        let Value::String(text) = value else {
            return;
        };
        if !text.trim_start().starts_with('=') {
            return;
        }
        if !is_m18_runtime_expression(text) {
            return;
        }
        self.validate_expression_text(
            path,
            node,
            descriptor,
            params,
            binding,
            text,
            payload_value_kind(payload_key, value),
        );
    }

    fn validate_expression_text(
        &mut self,
        path: &str,
        node: &UiNodeDefinition,
        descriptor: Option<&UiComponentDescriptor>,
        params: &BTreeMap<String, UiComponentParamSchema>,
        binding: &UiBindingRef,
        expression_text: &str,
        expected_kind: UiValueKind,
    ) {
        let expression = match UiBindingExpression::parse(expression_text) {
            Ok(expression) => expression,
            Err(error) => {
                let code = match error {
                    UiBindingExpressionParseError::UnsupportedOperator(_) => {
                        UiBindingDiagnosticCode::UnsupportedOperator
                    }
                    _ => UiBindingDiagnosticCode::InvalidValueKind,
                };
                self.push_error(
                    code,
                    path,
                    node,
                    binding,
                    format!("binding {} expression is invalid: {error}", binding.id),
                );
                return;
            }
        };
        let actual_kind =
            self.infer_expression_kind(path, node, descriptor, params, binding, &expression);
        let Some(actual_kind) = actual_kind else {
            return;
        };
        if !kind_matches(expected_kind, actual_kind) {
            self.push_error(
                UiBindingDiagnosticCode::InvalidValueKind,
                path,
                node,
                binding,
                format!(
                    "binding {} expression expected {:?} but resolved {:?}",
                    binding.id, expected_kind, actual_kind
                ),
            );
        }
    }

    fn expected_kind_for_target(
        &mut self,
        path: &str,
        node: &UiNodeDefinition,
        descriptor: Option<&UiComponentDescriptor>,
        binding: &UiBindingRef,
        target: &UiBindingTarget,
    ) -> Option<UiValueKind> {
        match target.kind {
            UiBindingTargetKind::Prop => {
                let Some(name) = target_name(target) else {
                    self.invalid_target(path, node, binding, "prop binding target requires name");
                    return None;
                };
                if let Some(schema) = descriptor.and_then(|descriptor| descriptor.prop(name)) {
                    return Some(schema.value_kind);
                }
                if descriptor.is_some() {
                    self.invalid_target(
                        path,
                        node,
                        binding,
                        format!("binding {} targets unknown prop {name}", binding.id),
                    );
                    return None;
                }
                if let Some(value) = node.props.get(name) {
                    return Some(UiValue::from_toml(value).kind());
                }
                self.invalid_target(
                    path,
                    node,
                    binding,
                    format!("binding {} targets unknown prop {name}", binding.id),
                );
                None
            }
            UiBindingTargetKind::Class => {
                self.named_bool_target(path, node, binding, target, "class")
            }
            UiBindingTargetKind::Visibility | UiBindingTargetKind::Enabled => {
                if target
                    .name
                    .as_deref()
                    .is_some_and(|name| !name.trim().is_empty())
                {
                    self.invalid_target(
                        path,
                        node,
                        binding,
                        format!("{:?} binding target must not declare name", target.kind),
                    );
                    return None;
                }
                Some(UiValueKind::Bool)
            }
            UiBindingTargetKind::ActionPayload => {
                self.named_payload_target(path, node, binding, target)
            }
        }
    }

    fn named_bool_target(
        &mut self,
        path: &str,
        node: &UiNodeDefinition,
        binding: &UiBindingRef,
        target: &UiBindingTarget,
        label: &str,
    ) -> Option<UiValueKind> {
        if target_name(target).is_none() {
            self.invalid_target(
                path,
                node,
                binding,
                format!("{label} binding target requires name"),
            );
            None
        } else {
            Some(UiValueKind::Bool)
        }
    }

    fn named_payload_target(
        &mut self,
        path: &str,
        node: &UiNodeDefinition,
        binding: &UiBindingRef,
        target: &UiBindingTarget,
    ) -> Option<UiValueKind> {
        let Some(name) = target_name(target) else {
            self.invalid_target(
                path,
                node,
                binding,
                "action payload binding target requires name",
            );
            return None;
        };
        if binding.action.is_none() {
            self.invalid_target(
                path,
                node,
                binding,
                format!(
                    "binding {} targets action payload {name} without an action payload",
                    binding.id
                ),
            );
            return None;
        }
        let Some(value) = binding
            .action
            .as_ref()
            .and_then(|action| action.payload.get(name))
        else {
            self.invalid_target(
                path,
                node,
                binding,
                format!(
                    "binding {} targets unknown action payload {name}",
                    binding.id
                ),
            );
            return None;
        };
        Some(UiValue::from_toml(value).kind())
    }

    fn infer_expression_kind(
        &mut self,
        path: &str,
        node: &UiNodeDefinition,
        descriptor: Option<&UiComponentDescriptor>,
        params: &BTreeMap<String, UiComponentParamSchema>,
        binding: &UiBindingRef,
        expression: &UiBindingExpression,
    ) -> Option<UiValueKind> {
        match expression {
            UiBindingExpression::Literal(value) => Some(value.kind()),
            UiBindingExpression::ParamRef(name) => params
                .get(name)
                .and_then(|schema| component_param_kind(schema.r#type.as_str()))
                .or_else(|| {
                    self.push_error(
                        UiBindingDiagnosticCode::UnresolvedRef,
                        path,
                        node,
                        binding,
                        format!("binding {} references unknown param {name}", binding.id),
                    );
                    None
                }),
            UiBindingExpression::PropRef(name) => {
                if let Some(descriptor) = descriptor {
                    return descriptor
                        .prop(name)
                        .map(|schema| schema.value_kind)
                        .or_else(|| {
                            self.push_error(
                                UiBindingDiagnosticCode::UnresolvedRef,
                                path,
                                node,
                                binding,
                                format!("binding {} references unknown prop {name}", binding.id),
                            );
                            None
                        });
                }
                node.props
                    .get(name)
                    .map(|value| UiValue::from_toml(value).kind())
                    .or_else(|| {
                        self.push_error(
                            UiBindingDiagnosticCode::UnresolvedRef,
                            path,
                            node,
                            binding,
                            format!("binding {} references unknown prop {name}", binding.id),
                        );
                        None
                    })
            }
            UiBindingExpression::Equals(lhs, rhs) | UiBindingExpression::NotEquals(lhs, rhs) => {
                let lhs_kind =
                    self.infer_expression_kind(path, node, descriptor, params, binding, lhs);
                let rhs_kind =
                    self.infer_expression_kind(path, node, descriptor, params, binding, rhs);
                if let (Some(lhs_kind), Some(rhs_kind)) = (lhs_kind, rhs_kind) {
                    if !kind_matches(lhs_kind, rhs_kind) && !kind_matches(rhs_kind, lhs_kind) {
                        self.push_error(
                            UiBindingDiagnosticCode::InvalidValueKind,
                            path,
                            node,
                            binding,
                            format!(
                                "binding {} compares incompatible kinds {:?} and {:?}",
                                binding.id, lhs_kind, rhs_kind
                            ),
                        );
                    }
                }
                Some(UiValueKind::Bool)
            }
            UiBindingExpression::And(lhs, rhs) | UiBindingExpression::Or(lhs, rhs) => {
                self.expect_bool_expression(path, node, descriptor, params, binding, lhs);
                self.expect_bool_expression(path, node, descriptor, params, binding, rhs);
                Some(UiValueKind::Bool)
            }
            UiBindingExpression::Not(nested) => {
                self.expect_bool_expression(path, node, descriptor, params, binding, nested);
                Some(UiValueKind::Bool)
            }
        }
    }

    fn expect_bool_expression(
        &mut self,
        path: &str,
        node: &UiNodeDefinition,
        descriptor: Option<&UiComponentDescriptor>,
        params: &BTreeMap<String, UiComponentParamSchema>,
        binding: &UiBindingRef,
        expression: &UiBindingExpression,
    ) {
        if let Some(kind) =
            self.infer_expression_kind(path, node, descriptor, params, binding, expression)
        {
            if !kind_matches(UiValueKind::Bool, kind) {
                self.push_error(
                    UiBindingDiagnosticCode::InvalidValueKind,
                    path,
                    node,
                    binding,
                    format!(
                        "binding {} boolean operator received {:?}",
                        binding.id, kind
                    ),
                );
            }
        }
    }

    fn invalid_target(
        &mut self,
        path: &str,
        node: &UiNodeDefinition,
        binding: &UiBindingRef,
        message: impl Into<String>,
    ) {
        self.push_error(
            UiBindingDiagnosticCode::InvalidTarget,
            path,
            node,
            binding,
            message,
        );
    }

    fn push_error(
        &mut self,
        code: UiBindingDiagnosticCode,
        path: impl Into<String>,
        node: &UiNodeDefinition,
        binding: &UiBindingRef,
        message: impl Into<String>,
    ) {
        self.report.diagnostics.push(UiBindingDiagnostic {
            code,
            severity: UiBindingDiagnosticSeverity::Error,
            path: path.into(),
            node_id: node.node_id.clone(),
            binding_id: binding.id.clone(),
            message: message.into(),
        });
    }
}

fn target_name(target: &UiBindingTarget) -> Option<&str> {
    target
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
}

fn kind_matches(expected: UiValueKind, actual: UiValueKind) -> bool {
    expected == UiValueKind::Any
        || actual == UiValueKind::Any
        || expected == actual
        || matches!((expected, actual), (UiValueKind::Float, UiValueKind::Int))
}

fn component_param_kind(value: &str) -> Option<UiValueKind> {
    match value.trim().to_ascii_lowercase().as_str() {
        "any" => Some(UiValueKind::Any),
        "bool" | "boolean" => Some(UiValueKind::Bool),
        "int" | "integer" => Some(UiValueKind::Int),
        "float" | "number" => Some(UiValueKind::Float),
        "string" | "text" => Some(UiValueKind::String),
        "color" => Some(UiValueKind::Color),
        "vec2" => Some(UiValueKind::Vec2),
        "vec3" => Some(UiValueKind::Vec3),
        "vec4" => Some(UiValueKind::Vec4),
        "asset_ref" | "assetref" | "asset" => Some(UiValueKind::AssetRef),
        "instance_ref" | "instanceref" | "instance" => Some(UiValueKind::InstanceRef),
        "array" | "collection" => Some(UiValueKind::Array),
        "map" | "object" => Some(UiValueKind::Map),
        "enum" => Some(UiValueKind::Enum),
        "flags" => Some(UiValueKind::Flags),
        "null" => Some(UiValueKind::Null),
        _ => None,
    }
}

fn payload_value_kind(payload_key: &str, value: &Value) -> UiValueKind {
    match payload_key {
        "checked" | "committed" | "confirm" | "enabled" | "visible" => UiValueKind::Bool,
        "delta" | "index" | "count" => UiValueKind::Int,
        _ => UiValue::from_toml(value).kind(),
    }
}

fn is_m18_runtime_expression(text: &str) -> bool {
    let expression = text
        .trim_start()
        .strip_prefix('=')
        .unwrap_or(text)
        .trim_start();
    expression.starts_with("prop.")
        || expression.starts_with("param.")
        || expression.starts_with("!")
        || expression.starts_with("(")
        || matches!(expression, "true" | "false" | "null")
        || expression.starts_with('"')
        || expression.starts_with('\'')
        || expression
            .chars()
            .next()
            .is_some_and(|ch| ch == '-' || ch.is_ascii_digit())
}
