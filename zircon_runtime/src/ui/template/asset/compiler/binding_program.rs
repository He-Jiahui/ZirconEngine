use std::collections::BTreeMap;

use zircon_runtime_interface::ui::template::{
    UiAssetError, UiAssetFingerprint, UiBindingExpression, UiBindingExpressionParseError,
    UiBindingId, UiBindingSchemaNameKind, UiBindingTargetKind, UiCompiledActionId,
    UiCompiledActionPayloadField, UiCompiledActionPayloadValue, UiCompiledAssetId,
    UiCompiledBinding, UiCompiledBindingExpression, UiCompiledBindingGeneration,
    UiCompiledBindingHandle, UiCompiledBindingProgram, UiCompiledBindingTarget,
    UiCompiledBindingTargetEndpoint, UiCompiledBindingTargetId, UiCompiledBindingTargetKind,
    UiCompiledControlId, UiCompiledNodeBindings, UiCompiledNodeId, UiCompiledRouteId, UiPropertyId,
    UiTemplateNode,
};

pub(crate) fn compile_binding_program(
    root: &UiTemplateNode,
    asset_id: &str,
) -> Result<UiCompiledBindingProgram, UiAssetError> {
    let serialized = toml::to_string(root).map_err(|error| {
        binding_program_error(
            asset_id,
            format!("failed to fingerprint compiled template: {error}"),
        )
    })?;
    let fingerprint = UiAssetFingerprint::from_bytes(serialized.as_bytes()).value;
    let generation =
        UiCompiledBindingGeneration::new(if fingerprint == 0 { 1 } else { fingerprint });

    let mut compiler = BindingProgramCompiler::new(generation, asset_id);
    let mut pending = vec![root];
    while let Some(node) = pending.pop() {
        compiler.compile_node(node)?;
        pending.extend(node.children.iter().rev());
    }
    let program = compiler.finish();
    if !program.is_well_formed() {
        return Err(binding_program_error(
            asset_id,
            "compiled binding program is malformed".to_string(),
        ));
    }
    Ok(program)
}

struct BindingProgramCompiler<'a> {
    generation: UiCompiledBindingGeneration,
    asset_id: &'a str,
    binding_names: Vec<String>,
    assets: StringInterner,
    properties: StringInterner,
    routes: StringInterner,
    actions: StringInterner,
    controls: StringInterner,
    nodes: Vec<UiCompiledNodeBindings>,
    bindings: Vec<UiCompiledBinding>,
}

impl<'a> BindingProgramCompiler<'a> {
    fn new(generation: UiCompiledBindingGeneration, asset_id: &'a str) -> Self {
        Self {
            generation,
            asset_id,
            binding_names: Vec::new(),
            assets: StringInterner::default(),
            properties: StringInterner::default(),
            routes: StringInterner::default(),
            actions: StringInterner::default(),
            controls: StringInterner::default(),
            nodes: Vec::new(),
            bindings: Vec::new(),
        }
    }

    fn compile_node(&mut self, node: &UiTemplateNode) -> Result<(), UiAssetError> {
        if !node.binding_source_asset_ids.is_empty()
            && node.binding_source_asset_ids.len() != node.bindings.len()
        {
            return Err(binding_program_error(
                self.asset_id,
                "compiled node binding ownership count does not match bindings".to_string(),
            ));
        }
        let node_owner = node.source_asset_id.as_deref().unwrap_or(self.asset_id);
        let node_owner_asset_id = UiCompiledAssetId::new(self.assets.intern(
            node_owner,
            self.asset_id,
            "binding owner asset",
        )?);
        let node_id = UiCompiledNodeId::new(checked_index(
            self.nodes.len(),
            self.asset_id,
            "compiled node",
        )?);
        let mut binding_ids = Vec::with_capacity(node.bindings.len());
        for (source_binding_index, binding) in node.bindings.iter().enumerate() {
            let binding_owner = node
                .binding_source_asset_ids
                .get(source_binding_index)
                .map(String::as_str)
                .unwrap_or(node_owner);
            let binding_owner_asset_id = UiCompiledAssetId::new(self.assets.intern(
                binding_owner,
                self.asset_id,
                "binding owner asset",
            )?);
            let binding_id = UiBindingId::new(checked_index(
                self.bindings.len(),
                self.asset_id,
                "compiled binding",
            )?);
            let handle = UiCompiledBindingHandle {
                generation: self.generation,
                binding_id,
            };
            let mut targets = Vec::with_capacity(binding.targets.len());
            for (target_index, assignment) in binding.targets.iter().enumerate() {
                let (kind, property) = match assignment.target.kind {
                    UiBindingTargetKind::Prop => (
                        UiCompiledBindingTargetKind::Property,
                        Some(self.required_property(
                            assignment.target.name.as_deref(),
                            &binding.id,
                            "property",
                        )?),
                    ),
                    UiBindingTargetKind::Class => (
                        UiCompiledBindingTargetKind::Class,
                        Some(self.required_property(
                            assignment.target.name.as_deref(),
                            &binding.id,
                            "class",
                        )?),
                    ),
                    UiBindingTargetKind::Visibility => {
                        (UiCompiledBindingTargetKind::Visibility, None)
                    }
                    UiBindingTargetKind::Enabled => (UiCompiledBindingTargetKind::Enabled, None),
                    UiBindingTargetKind::ActionPayload => (
                        UiCompiledBindingTargetKind::ActionPayload,
                        Some(self.required_property(
                            assignment.target.name.as_deref(),
                            &binding.id,
                            "action payload",
                        )?),
                    ),
                };
                let parsed =
                    UiBindingExpression::parse(&assignment.expression).map_err(|error| {
                        binding_program_error(
                            self.asset_id,
                            format!(
                                "binding {} has invalid target expression: {error}",
                                binding.id
                            ),
                        )
                    })?;
                let expression = self.compile_expression(parsed, &binding.id)?;
                targets.push(UiCompiledBindingTarget {
                    endpoint: UiCompiledBindingTargetEndpoint {
                        generation: self.generation,
                        node_id,
                        binding_id,
                        target_index: UiCompiledBindingTargetId::new(checked_index(
                            target_index,
                            self.asset_id,
                            "compiled binding target",
                        )?),
                    },
                    kind,
                    property,
                    missing_policy: assignment.target.missing_policy.clone(),
                    expression,
                });
            }

            self.validate_optional_schema_name(
                UiBindingSchemaNameKind::Route,
                binding.route.as_deref(),
                &binding.id,
                "route",
            )?;
            let action = binding.action.as_ref();
            if let Some(action) = action {
                self.validate_optional_schema_name(
                    UiBindingSchemaNameKind::Route,
                    action.route.as_deref(),
                    &binding.id,
                    "route",
                )?;
                self.validate_optional_schema_name(
                    UiBindingSchemaNameKind::Action,
                    action.action.as_deref(),
                    &binding.id,
                    "action",
                )?;
            }
            let route = action
                .and_then(|action| action.route.as_deref())
                .or(binding.route.as_deref())
                .filter(|value| !value.is_empty());
            let action_id = action
                .and_then(|action| action.action.as_deref())
                .filter(|value| !value.is_empty());
            let route_id = route
                .map(|value| self.routes.intern(value, self.asset_id, "route"))
                .transpose()?
                .map(UiCompiledRouteId::new);
            let action_id = action_id
                .map(|value| self.actions.intern(value, self.asset_id, "action"))
                .transpose()?
                .map(UiCompiledActionId::new);
            let payload_fields = action
                .map(|action| {
                    action
                        .payload
                        .iter()
                        .map(|(field, value)| {
                            self.validate_schema_name(
                                UiBindingSchemaNameKind::PayloadField,
                                field,
                                &binding.id,
                                "action payload field",
                            )?;
                            let property = self
                                .properties
                                .intern(field, self.asset_id, "action payload field")
                                .map(UiPropertyId::new)?;
                            let value = self.compile_action_payload_value(value, &binding.id)?;
                            Ok(UiCompiledActionPayloadField { property, value })
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?
                .unwrap_or_default();

            self.binding_names.push(binding.id.clone());
            self.bindings.push(UiCompiledBinding {
                handle,
                owner_asset_id: binding_owner_asset_id,
                node_id,
                source_binding_index: checked_index(
                    source_binding_index,
                    self.asset_id,
                    "source binding slot",
                )?,
                event: binding.event,
                mode: binding.mode,
                component_event: binding.component_event,
                route_id,
                action_id,
                payload_missing_policy: action
                    .map(|action| action.payload_missing_policy.clone())
                    .unwrap_or_default(),
                payload_fields,
                targets,
            });
            binding_ids.push(binding_id);
        }
        self.nodes.push(UiCompiledNodeBindings {
            owner_asset_id: node_owner_asset_id,
            binding_ids,
        });
        Ok(())
    }

    fn required_property(
        &mut self,
        value: Option<&str>,
        binding_id: &str,
        label: &str,
    ) -> Result<UiPropertyId, UiAssetError> {
        let value = value
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                binding_program_error(
                    self.asset_id,
                    format!("binding {binding_id} has an empty {label} target"),
                )
            })?;
        self.properties
            .intern(value, self.asset_id, "property")
            .map(UiPropertyId::new)
    }

    fn validate_optional_schema_name(
        &self,
        kind: UiBindingSchemaNameKind,
        value: Option<&str>,
        binding_id: &str,
        label: &str,
    ) -> Result<(), UiAssetError> {
        let Some(value) = value else {
            return Ok(());
        };
        self.validate_schema_name(kind, value, binding_id, label)
    }

    fn validate_schema_name(
        &self,
        kind: UiBindingSchemaNameKind,
        value: &str,
        binding_id: &str,
        label: &str,
    ) -> Result<(), UiAssetError> {
        kind.validate(value).map_err(|error| {
            binding_program_error(
                self.asset_id,
                format!("binding {binding_id} has invalid {label} `{value}`: {error}"),
            )
        })
    }

    fn compile_expression(
        &mut self,
        expression: UiBindingExpression,
        binding_id: &str,
    ) -> Result<UiCompiledBindingExpression, UiAssetError> {
        match expression {
            UiBindingExpression::Literal(value) => {
                Ok(UiCompiledBindingExpression::Literal(value))
            }
            UiBindingExpression::ParamRef(name) => Err(binding_program_error(
                self.asset_id,
                format!(
                    "binding {binding_id} retains unresolved component parameter {name} after expansion"
                ),
            )),
            UiBindingExpression::PropRef(property) => self
                .properties
                .intern(&property, self.asset_id, "property")
                .map(UiPropertyId::new)
                .map(UiCompiledBindingExpression::Property),
            UiBindingExpression::ControlPropRef {
                control_id,
                property,
            } => Ok(UiCompiledBindingExpression::ControlProperty {
                control_id: UiCompiledControlId::new(self.controls.intern(
                    &control_id,
                    self.asset_id,
                    "control",
                )?),
                property_id: UiPropertyId::new(self.properties.intern(
                    &property,
                    self.asset_id,
                    "property",
                )?),
            }),
            UiBindingExpression::Equals(lhs, rhs) => Ok(UiCompiledBindingExpression::Equals(
                Box::new(self.compile_expression(*lhs, binding_id)?),
                Box::new(self.compile_expression(*rhs, binding_id)?),
            )),
            UiBindingExpression::NotEquals(lhs, rhs) => {
                Ok(UiCompiledBindingExpression::NotEquals(
                    Box::new(self.compile_expression(*lhs, binding_id)?),
                    Box::new(self.compile_expression(*rhs, binding_id)?),
                ))
            }
            UiBindingExpression::And(lhs, rhs) => Ok(UiCompiledBindingExpression::And(
                Box::new(self.compile_expression(*lhs, binding_id)?),
                Box::new(self.compile_expression(*rhs, binding_id)?),
            )),
            UiBindingExpression::Or(lhs, rhs) => Ok(UiCompiledBindingExpression::Or(
                Box::new(self.compile_expression(*lhs, binding_id)?),
                Box::new(self.compile_expression(*rhs, binding_id)?),
            )),
            UiBindingExpression::Not(value) => Ok(UiCompiledBindingExpression::Not(Box::new(
                self.compile_expression(*value, binding_id)?,
            ))),
        }
    }

    fn compile_action_payload_value(
        &mut self,
        value: &toml::Value,
        binding_id: &str,
    ) -> Result<UiCompiledActionPayloadValue, UiAssetError> {
        let toml::Value::String(expression_text) = value else {
            return Ok(UiCompiledActionPayloadValue::Literal(
                zircon_runtime_interface::ui::component::UiValue::from_toml(value),
            ));
        };
        if !expression_text.trim_start().starts_with('=') {
            return Ok(UiCompiledActionPayloadValue::Literal(
                zircon_runtime_interface::ui::component::UiValue::String(expression_text.clone()),
            ));
        }

        match UiBindingExpression::parse(expression_text) {
            Ok(expression) => self
                .compile_expression(expression, binding_id)
                .map(UiCompiledActionPayloadValue::Expression),
            Err(error @ UiBindingExpressionParseError::BudgetExceeded { .. }) => {
                Err(binding_program_error(
                    self.asset_id,
                    format!("binding {binding_id} has over-budget action payload: {error}"),
                ))
            }
            Err(_) => Ok(UiCompiledActionPayloadValue::Unavailable),
        }
    }

    fn finish(self) -> UiCompiledBindingProgram {
        let asset_id = self.asset_id.to_string();
        UiCompiledBindingProgram::new(
            self.generation,
            self.binding_names,
            self.properties.values,
            self.routes.values,
            self.actions.values,
            self.controls.values,
            self.nodes,
            self.bindings,
        )
        .with_asset_id(asset_id)
        .with_asset_ownership(self.assets.values)
    }
}

#[derive(Default)]
struct StringInterner {
    by_value: BTreeMap<String, u32>,
    values: Vec<String>,
}

impl StringInterner {
    fn intern(&mut self, value: &str, asset_id: &str, label: &str) -> Result<u32, UiAssetError> {
        if let Some(id) = self.by_value.get(value) {
            return Ok(*id);
        }
        let id = checked_index(self.values.len(), asset_id, label)?;
        self.values.push(value.to_string());
        self.by_value.insert(value.to_string(), id);
        Ok(id)
    }
}

fn checked_index(value: usize, asset_id: &str, label: &str) -> Result<u32, UiAssetError> {
    u32::try_from(value).map_err(|_| {
        binding_program_error(
            asset_id,
            format!("{label} count exceeds the compiled u32 identity space"),
        )
    })
}

fn binding_program_error(asset_id: &str, detail: String) -> UiAssetError {
    UiAssetError::InvalidDocument {
        asset_id: asset_id.to_string(),
        detail,
    }
}
