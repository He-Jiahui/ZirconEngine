use std::{collections::BTreeMap, time::Instant};

use zircon_runtime_interface::ui::{
    binding::{
        UiBindingDirtyDomain, UiBindingExecutionReceipt, UiBindingSource, UiBindingSourceKind,
        UiBindingTarget as UiRuntimeBindingTarget, UiBindingUpdate, UiBindingUpdateReport,
        UiBindingUpdateStatus,
    },
    component::UiValue,
    dispatch::{UiPointerComponentEvent, UiTemplateActionInvocation},
    event_ui::UiNodeId,
    template::{
        UI_BINDING_EXPRESSION_INLINE_STACK_CAPACITY, UI_BINDING_EXPRESSION_MAX_DEPTH,
        UI_BINDING_EXPRESSION_MAX_NODES, UiBindingMissingValueResolution,
        UiCompiledActionPayloadValue, UiCompiledBinding, UiCompiledBindingExpression,
        UiCompiledBindingHandle, UiCompiledBindingTarget, UiCompiledBindingTargetKind,
        UiCompiledNodeId, UiPropertyId,
    },
    tree::{UiDirtyFlags, UiTreeError},
};

use super::{
    UiBindingMutationTransaction, UiPropertyMutationReport, UiPropertyMutationRequest,
    UiPropertyMutationStatus, UiSurface,
};

struct UiPreparedBindingTargets {
    targets: Vec<UiPreparedBindingTarget>,
    action_payload: BTreeMap<UiPropertyId, UiValue>,
}

enum UiPreparedBindingTarget {
    Property {
        name: String,
        value: UiValue,
    },
    Class {
        name: String,
        enabled: bool,
    },
    Visibility {
        visible: bool,
    },
    Enabled {
        enabled: bool,
    },
    ActionPayload {
        name: String,
        previous: Option<UiValue>,
        value: UiValue,
    },
}

struct UiBindingTargetExecution {
    report: UiBindingUpdateReport,
    template_action: Option<UiTemplateActionInvocation>,
    published: bool,
}

#[derive(Default)]
struct UiBindingTargetApplySummary {
    applied_target_count: usize,
    unchanged_target_count: usize,
    impact: Vec<UiBindingDirtyDomain>,
    advances_surface_revision: bool,
}

impl UiBindingTargetApplySummary {
    fn record(
        &mut self,
        status: UiBindingUpdateStatus,
        impact: impl IntoIterator<Item = UiBindingDirtyDomain>,
        advances_surface_revision: bool,
    ) {
        match status {
            UiBindingUpdateStatus::Applied => self.applied_target_count += 1,
            UiBindingUpdateStatus::Unchanged => self.unchanged_target_count += 1,
            UiBindingUpdateStatus::Rejected => return,
        }
        if status == UiBindingUpdateStatus::Applied {
            for domain in impact {
                if !self.impact.contains(&domain) {
                    self.impact.push(domain);
                }
            }
            self.advances_surface_revision |= advances_surface_revision;
        }
    }
}

struct UiBindingTargetCommit {
    report: UiBindingUpdateReport,
    summary: UiBindingTargetApplySummary,
}

#[derive(Clone, Copy)]
enum UiCompiledBinaryOperator {
    Equals,
    NotEquals,
}

enum UiCompiledExpressionFrame<'a> {
    Enter(&'a UiCompiledBindingExpression, usize),
    BinaryAfterLeft {
        operator: UiCompiledBinaryOperator,
        right: &'a UiCompiledBindingExpression,
        depth: usize,
    },
    BinaryAfterRight {
        operator: UiCompiledBinaryOperator,
        left: UiValue,
    },
    AndAfterLeft {
        right: &'a UiCompiledBindingExpression,
        depth: usize,
    },
    AndAfterRight,
    OrAfterLeft {
        right: &'a UiCompiledBindingExpression,
        depth: usize,
    },
    OrAfterRight,
    NotAfterValue,
}

struct UiCompiledExpressionStack<T> {
    inline: [Option<T>; UI_BINDING_EXPRESSION_INLINE_STACK_CAPACITY],
    inline_len: usize,
    spill: Vec<T>,
}

impl<T> UiCompiledExpressionStack<T> {
    fn new() -> Self {
        Self {
            inline: std::array::from_fn(|_| None),
            inline_len: 0,
            spill: Vec::new(),
        }
    }

    fn push(&mut self, value: T) -> Option<()> {
        if self.len() >= UI_BINDING_EXPRESSION_MAX_DEPTH + 1 {
            return None;
        }
        if self.spill.is_empty() && self.inline_len < self.inline.len() {
            self.inline[self.inline_len] = Some(value);
            self.inline_len += 1;
        } else {
            self.spill.push(value);
        }
        Some(())
    }

    fn pop(&mut self) -> Option<T> {
        if let Some(value) = self.spill.pop() {
            return Some(value);
        }
        self.inline_len = self.inline_len.checked_sub(1)?;
        self.inline[self.inline_len].take()
    }

    fn len(&self) -> usize {
        self.inline_len + self.spill.len()
    }
}

impl UiSurface {
    pub(crate) fn apply_pointer_binding_targets(
        &mut self,
        events: &mut Vec<UiPointerComponentEvent>,
    ) -> Result<Vec<UiBindingUpdateReport>, UiTreeError> {
        if self.compiled_bindings.binding_count() == 0
            && !events.iter().any(|event| self.raw_event_has_targets(event))
        {
            return Ok(Vec::new());
        }

        let mut reports = Vec::new();
        let mut failure = None;
        events.retain_mut(|event| {
            if failure.is_some() {
                return false;
            }
            match self.apply_pointer_binding_target_event(event, &mut reports) {
                Ok(retain) => retain,
                Err(error) => {
                    failure = Some(error);
                    false
                }
            }
        });
        if let Some(error) = failure {
            events.clear();
            return Err(error);
        }
        Ok(reports)
    }

    #[cfg(test)]
    pub(crate) fn apply_pointer_binding_targets_legacy_for_benchmark(
        &mut self,
        events: &mut Vec<UiPointerComponentEvent>,
    ) -> Result<Vec<UiBindingUpdateReport>, UiTreeError> {
        if self.compiled_bindings.binding_count() == 0
            && !events.iter().any(|event| self.raw_event_has_targets(event))
        {
            return Ok(Vec::new());
        }

        let mut retained = Vec::with_capacity(events.len());
        let mut reports = Vec::new();
        for mut event in std::mem::take(events) {
            if self.apply_pointer_binding_target_event(&mut event, &mut reports)? {
                retained.push(event);
            }
        }
        *events = retained;
        Ok(reports)
    }

    fn apply_pointer_binding_target_event(
        &mut self,
        event: &mut UiPointerComponentEvent,
        reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let Some((generation, executable)) = self.binding_target_event_profile(event) else {
            crate::profile_counter!("runtime", "ui.binding.target_passthrough_count", 1);
            return Ok(true);
        };
        let started_at = Instant::now();
        let report_index = reports.len();
        let result = self.apply_pointer_binding_target_event_inner(event, reports);
        if let Some(report) = reports.get_mut(report_index) {
            let cost_nanos = started_at.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
            let asset_id = self
                .compiled_bindings
                .asset_id()
                .unwrap_or(event.envelope.document_id.as_str());
            let receipt = if executable {
                UiBindingExecutionReceipt::executed(
                    asset_id,
                    event.binding_id.as_str(),
                    generation,
                    report.rejected_count > 0,
                    cost_nanos,
                )
            } else {
                UiBindingExecutionReceipt::missed(
                    asset_id,
                    event.binding_id.as_str(),
                    generation,
                    cost_nanos,
                )
            };
            crate::profile_counter!(
                "runtime",
                "ui.binding.execution_count",
                receipt.execution_count
            );
            crate::profile_counter!("runtime", "ui.binding.miss_count", receipt.miss_count);
            crate::profile_counter!("runtime", "ui.binding.error_count", receipt.error_count);
            crate::profile_counter!("runtime", "ui.binding.cost_nanos", receipt.cost_nanos);
            report.execution_receipt = Some(receipt);
        }
        result
    }

    fn binding_target_event_profile(&self, event: &UiPointerComponentEvent) -> Option<(u64, bool)> {
        match event.compiled_binding {
            Some(handle) => {
                let generation = handle.generation.get();
                let Some(binding) = self.compiled_bindings.binding(handle) else {
                    return Some((generation, false));
                };
                if binding.targets.is_empty() {
                    return None;
                }
                Some((
                    generation,
                    self.compiled_binding_matches_event(binding, event),
                ))
            }
            None => self
                .raw_event_has_targets(event)
                .then(|| (self.compiled_bindings.generation().get(), false)),
        }
    }

    fn apply_pointer_binding_target_event_inner(
        &mut self,
        event: &mut UiPointerComponentEvent,
        reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let Some(handle) = event.compiled_binding else {
            if self.raw_event_has_targets(event) {
                reports.push(rejected_compiled_endpoint_report(
                    event.node_id,
                    &event.binding_id,
                    "binding target event is missing its compiled endpoint",
                ));
                return Ok(false);
            }
            return Ok(true);
        };
        let Some(binding) = self.compiled_bindings.binding(handle) else {
            reports.push(rejected_compiled_endpoint_report(
                event.node_id,
                &event.binding_id,
                "binding target endpoint generation is stale or unknown",
            ));
            return Ok(false);
        };
        if binding.targets.is_empty() {
            return Ok(true);
        }
        if !self.compiled_binding_matches_event(binding, event) {
            reports.push(rejected_compiled_endpoint_report(
                event.node_id,
                &event.binding_id,
                "binding target endpoint does not match the dispatched node, event, or identity",
            ));
            return Ok(false);
        }

        let execution = self.execute_binding_targets(event.node_id, handle)?;
        reports.push(execution.report);
        if !execution.published {
            return Ok(false);
        }
        event.template_action = execution.template_action;
        Ok(true)
    }

    fn raw_event_has_targets(&self, event: &UiPointerComponentEvent) -> bool {
        self.tree
            .node(event.node_id)
            .and_then(|node| node.template_metadata.as_ref())
            .is_some_and(|metadata| {
                metadata.bindings.iter().any(|binding| {
                    binding.id == event.binding_id
                        && binding.event == event.event_kind
                        && !binding.targets.is_empty()
                })
            })
    }

    fn compiled_binding_matches_event(
        &self,
        binding: &UiCompiledBinding,
        event: &UiPointerComponentEvent,
    ) -> bool {
        let Some(node_index) = event
            .node_id
            .0
            .checked_sub(1)
            .and_then(|value| u32::try_from(value).ok())
        else {
            return false;
        };
        binding.node_id == UiCompiledNodeId::new(node_index)
            && binding.event == event.event_kind
            && self.compiled_bindings.binding_name(binding.handle)
                == Some(event.binding_id.as_str())
    }

    fn execute_binding_targets(
        &mut self,
        source_node_id: UiNodeId,
        handle: UiCompiledBindingHandle,
    ) -> Result<UiBindingTargetExecution, UiTreeError> {
        let Some(binding) = self.compiled_bindings.binding(handle) else {
            return Ok(UiBindingTargetExecution {
                report: rejected_compiled_endpoint_report(
                    source_node_id,
                    "unknown",
                    "binding target endpoint generation is stale or unknown",
                ),
                template_action: None,
                published: false,
            });
        };
        let Some(binding_name) = self
            .compiled_bindings
            .binding_name(handle)
            .map(str::to_string)
        else {
            return Ok(UiBindingTargetExecution {
                report: rejected_compiled_endpoint_report(
                    source_node_id,
                    "unknown",
                    "binding target endpoint has no interned binding identity",
                ),
                template_action: None,
                published: false,
            });
        };
        let prepared = match self.prepare_binding_targets(source_node_id, &binding_name, binding) {
            Ok(prepared) => prepared,
            Err(report) => {
                return Ok(UiBindingTargetExecution {
                    report,
                    template_action: None,
                    published: false,
                });
            }
        };
        let UiPreparedBindingTargets {
            targets,
            action_payload,
        } = prepared;
        let template_action = self.template_action_for_compiled_binding_with_overrides(
            source_node_id,
            handle,
            action_payload,
        );
        let target_count = targets.len();
        let transaction = UiBindingMutationTransaction::prepare(self, target_count);
        let report =
            match self.commit_prepared_binding_targets(source_node_id, &binding_name, targets) {
                Ok(Ok(mut commit)) => {
                    commit.report.transaction = Some(transaction.commit(
                        commit.summary.applied_target_count,
                        commit.summary.unchanged_target_count,
                        commit.summary.impact,
                        commit.summary.advances_surface_revision,
                    ));
                    commit.report
                }
                Ok(Err(mut report)) => {
                    report.transaction = Some(transaction.rollback(self));
                    return Ok(UiBindingTargetExecution {
                        report,
                        template_action: None,
                        published: false,
                    });
                }
                Err(error) => {
                    let _ = transaction.rollback(self);
                    return Err(error);
                }
            };
        Ok(UiBindingTargetExecution {
            report,
            template_action,
            published: true,
        })
    }

    fn prepare_binding_targets(
        &self,
        source_node_id: UiNodeId,
        binding_name: &str,
        binding: &UiCompiledBinding,
    ) -> Result<UiPreparedBindingTargets, UiBindingUpdateReport> {
        let mut targets = Vec::with_capacity(binding.targets.len());
        let mut action_payload = BTreeMap::new();
        for (target_index, target) in binding.targets.iter().enumerate() {
            if !compiled_target_matches_binding(binding, target, target_index) {
                return Err(self.rejected_target_report(
                    source_node_id,
                    binding_name,
                    target,
                    "binding target endpoint generation or identity is invalid",
                ));
            }
            let value = match target.missing_policy.resolve(
                self.resolve_compiled_binding_expression(source_node_id, &target.expression),
            ) {
                UiBindingMissingValueResolution::Value(value) => value,
                UiBindingMissingValueResolution::Omitted => continue,
                UiBindingMissingValueResolution::RequiredMissing => {
                    return Err(self.rejected_target_report(
                        source_node_id,
                        binding_name,
                        target,
                        "binding target required value is missing",
                    ));
                }
                UiBindingMissingValueResolution::ExplicitError => {
                    return Err(self.rejected_target_report(
                        source_node_id,
                        binding_name,
                        target,
                        "binding target missing value reached explicit error policy",
                    ));
                }
            };
            let prepared =
                self.prepare_binding_target(source_node_id, binding_name, binding, target, value)?;
            if let UiPreparedBindingTarget::ActionPayload { value, .. } = &prepared {
                if let Some(property) = target.property {
                    action_payload.insert(property, value.clone());
                }
            }
            targets.push(prepared);
        }
        Ok(UiPreparedBindingTargets {
            targets,
            action_payload,
        })
    }

    fn prepare_binding_target(
        &self,
        source_node_id: UiNodeId,
        binding_name: &str,
        binding: &UiCompiledBinding,
        target: &UiCompiledBindingTarget,
        value: UiValue,
    ) -> Result<UiPreparedBindingTarget, UiBindingUpdateReport> {
        let reject = |message: &str| {
            self.rejected_target_report(source_node_id, binding_name, target, message)
        };
        match target.kind {
            UiCompiledBindingTargetKind::Property => {
                let name = self.compiled_target_property(target).ok_or_else(|| {
                    reject("property binding target requires a valid interned property")
                })?;
                Ok(UiPreparedBindingTarget::Property {
                    name: name.to_string(),
                    value,
                })
            }
            UiCompiledBindingTargetKind::Class => {
                let name = self.compiled_target_property(target).ok_or_else(|| {
                    reject("class binding target requires a valid interned class")
                })?;
                let UiValue::Bool(enabled) = value else {
                    return Err(reject("class binding target requires a boolean value"));
                };
                Ok(UiPreparedBindingTarget::Class {
                    name: name.to_string(),
                    enabled,
                })
            }
            UiCompiledBindingTargetKind::Visibility => {
                let UiValue::Bool(visible) = value else {
                    return Err(reject("visibility binding target requires a boolean value"));
                };
                Ok(UiPreparedBindingTarget::Visibility { visible })
            }
            UiCompiledBindingTargetKind::Enabled => {
                let UiValue::Bool(enabled) = value else {
                    return Err(reject("enabled binding target requires a boolean value"));
                };
                Ok(UiPreparedBindingTarget::Enabled { enabled })
            }
            UiCompiledBindingTargetKind::ActionPayload => {
                let name = self.compiled_target_property(target).ok_or_else(|| {
                    reject("action payload binding target requires a valid interned field")
                })?;
                let previous = binding
                    .payload_fields
                    .iter()
                    .find(|field| Some(field.property) == target.property)
                    .and_then(|field| {
                        self.resolve_compiled_action_payload_value(source_node_id, &field.value)
                    });
                if previous.is_none() {
                    return Err(reject(
                        "action payload binding target must name an existing payload field",
                    ));
                }
                Ok(UiPreparedBindingTarget::ActionPayload {
                    name: name.to_string(),
                    previous,
                    value,
                })
            }
        }
    }

    fn resolve_compiled_binding_expression(
        &self,
        source_node_id: UiNodeId,
        root: &UiCompiledBindingExpression,
    ) -> Option<UiValue> {
        let mut frames = UiCompiledExpressionStack::new();
        frames.push(UiCompiledExpressionFrame::Enter(root, 1))?;
        let mut values = UiCompiledExpressionStack::new();
        let mut visited_nodes = 0usize;
        while let Some(frame) = frames.pop() {
            match frame {
                UiCompiledExpressionFrame::Enter(expression, depth) => {
                    visited_nodes += 1;
                    if visited_nodes > UI_BINDING_EXPRESSION_MAX_NODES
                        || depth > UI_BINDING_EXPRESSION_MAX_DEPTH
                    {
                        return None;
                    }
                    match expression {
                        UiCompiledBindingExpression::Literal(value) => {
                            values.push(value.clone())?
                        }
                        UiCompiledBindingExpression::Property(property_id) => values.push(
                            self.compiled_bindings
                                .property_name(*property_id)
                                .and_then(|property| {
                                    self.template_action_property_value(source_node_id, property)
                                })?,
                        )?,
                        UiCompiledBindingExpression::ControlProperty {
                            control_id,
                            property_id,
                        } => values.push(
                            self.control_index
                                .unique_node_id_for_compiled_control(
                                    &self.tree,
                                    &self.compiled_bindings,
                                    *control_id,
                                )
                                .zip(self.compiled_bindings.property_name(*property_id))
                                .and_then(|(node_id, property)| {
                                    self.template_action_property_value(node_id, property)
                                })?,
                        )?,
                        UiCompiledBindingExpression::Equals(left, right) => {
                            frames.push(UiCompiledExpressionFrame::BinaryAfterLeft {
                                operator: UiCompiledBinaryOperator::Equals,
                                right,
                                depth: depth + 1,
                            })?;
                            frames.push(UiCompiledExpressionFrame::Enter(left, depth + 1))?;
                        }
                        UiCompiledBindingExpression::NotEquals(left, right) => {
                            frames.push(UiCompiledExpressionFrame::BinaryAfterLeft {
                                operator: UiCompiledBinaryOperator::NotEquals,
                                right,
                                depth: depth + 1,
                            })?;
                            frames.push(UiCompiledExpressionFrame::Enter(left, depth + 1))?;
                        }
                        UiCompiledBindingExpression::And(left, right) => {
                            frames.push(UiCompiledExpressionFrame::AndAfterLeft {
                                right,
                                depth: depth + 1,
                            })?;
                            frames.push(UiCompiledExpressionFrame::Enter(left, depth + 1))?;
                        }
                        UiCompiledBindingExpression::Or(left, right) => {
                            frames.push(UiCompiledExpressionFrame::OrAfterLeft {
                                right,
                                depth: depth + 1,
                            })?;
                            frames.push(UiCompiledExpressionFrame::Enter(left, depth + 1))?;
                        }
                        UiCompiledBindingExpression::Not(value) => {
                            frames.push(UiCompiledExpressionFrame::NotAfterValue)?;
                            frames.push(UiCompiledExpressionFrame::Enter(value, depth + 1))?;
                        }
                    }
                }
                UiCompiledExpressionFrame::BinaryAfterLeft {
                    operator,
                    right,
                    depth,
                } => {
                    let left = values.pop()?;
                    frames.push(UiCompiledExpressionFrame::BinaryAfterRight { operator, left })?;
                    frames.push(UiCompiledExpressionFrame::Enter(right, depth))?;
                }
                UiCompiledExpressionFrame::BinaryAfterRight { operator, left } => {
                    let right = values.pop()?;
                    values.push(UiValue::Bool(match operator {
                        UiCompiledBinaryOperator::Equals => left == right,
                        UiCompiledBinaryOperator::NotEquals => left != right,
                    }))?;
                }
                UiCompiledExpressionFrame::AndAfterLeft { right, depth } => {
                    if !compiled_binding_bool(&values.pop()?)? {
                        values.push(UiValue::Bool(false))?;
                    } else {
                        frames.push(UiCompiledExpressionFrame::AndAfterRight)?;
                        frames.push(UiCompiledExpressionFrame::Enter(right, depth))?;
                    }
                }
                UiCompiledExpressionFrame::AndAfterRight => {
                    let value = compiled_binding_bool(&values.pop()?)?;
                    values.push(UiValue::Bool(value))?;
                }
                UiCompiledExpressionFrame::OrAfterLeft { right, depth } => {
                    if compiled_binding_bool(&values.pop()?)? {
                        values.push(UiValue::Bool(true))?;
                    } else {
                        frames.push(UiCompiledExpressionFrame::OrAfterRight)?;
                        frames.push(UiCompiledExpressionFrame::Enter(right, depth))?;
                    }
                }
                UiCompiledExpressionFrame::OrAfterRight => {
                    let value = compiled_binding_bool(&values.pop()?)?;
                    values.push(UiValue::Bool(value))?;
                }
                UiCompiledExpressionFrame::NotAfterValue => {
                    let value = !compiled_binding_bool(&values.pop()?)?;
                    values.push(UiValue::Bool(value))?;
                }
            }
        }
        if values.len() == 1 {
            values.pop()
        } else {
            None
        }
    }

    pub(crate) fn resolve_compiled_action_payload_value(
        &self,
        source_node_id: UiNodeId,
        value: &UiCompiledActionPayloadValue,
    ) -> Option<UiValue> {
        match value {
            UiCompiledActionPayloadValue::Literal(value) => Some(value.clone()),
            UiCompiledActionPayloadValue::Expression(expression) => {
                self.resolve_compiled_binding_expression(source_node_id, expression)
            }
            UiCompiledActionPayloadValue::Unavailable => None,
        }
    }

    fn compiled_target_property<'a>(&'a self, target: &UiCompiledBindingTarget) -> Option<&'a str> {
        target
            .property
            .and_then(|property| self.compiled_bindings.property_name(property))
    }

    fn rejected_target_report(
        &self,
        source_node_id: UiNodeId,
        binding_name: &str,
        target: &UiCompiledBindingTarget,
        message: impl Into<String>,
    ) -> UiBindingUpdateReport {
        let property = self.compiled_target_property(target).unwrap_or_default();
        let runtime_target = runtime_target(source_node_id, target.kind, property);
        UiBindingUpdateReport::from_updates(vec![UiBindingUpdate::rejected(
            binding_source(source_node_id, binding_name),
            runtime_target,
            UiValue::Null,
            message,
        )])
    }

    fn commit_prepared_binding_targets(
        &mut self,
        source_node_id: UiNodeId,
        binding_name: &str,
        targets: Vec<UiPreparedBindingTarget>,
    ) -> Result<Result<UiBindingTargetCommit, UiBindingUpdateReport>, UiTreeError> {
        let mut updates = Vec::new();
        let mut summary = UiBindingTargetApplySummary::default();
        for target in targets {
            match target {
                UiPreparedBindingTarget::Property { name, value } => {
                    let report = self.mutate_property(
                        UiPropertyMutationRequest::new(source_node_id, name, value)
                            .with_binding_source_kind(UiBindingSourceKind::ComponentEvent),
                    )?;
                    if report.status == UiPropertyMutationStatus::Rejected {
                        return Ok(Err(report.binding));
                    }
                    summary.record(
                        binding_status(report.status),
                        property_report_impact(&report),
                        report.status == UiPropertyMutationStatus::Accepted,
                    );
                    updates.extend(report.binding.updates);
                }
                UiPreparedBindingTarget::Class { name, enabled } => {
                    let update =
                        self.commit_class_target(source_node_id, binding_name, name, enabled)?;
                    summary.record(
                        update.status,
                        update.dirty.iter().copied(),
                        update.status == UiBindingUpdateStatus::Applied,
                    );
                    updates.push(update);
                }
                UiPreparedBindingTarget::Visibility { visible } => {
                    let report = self.mutate_property(
                        UiPropertyMutationRequest::new(
                            source_node_id,
                            "visible",
                            UiValue::Bool(visible),
                        )
                        .with_binding_source_kind(UiBindingSourceKind::ComponentEvent),
                    )?;
                    if report.status == UiPropertyMutationStatus::Rejected {
                        return Ok(Err(report.binding));
                    }
                    summary.record(
                        binding_status(report.status),
                        property_report_impact(&report),
                        report.status == UiPropertyMutationStatus::Accepted,
                    );
                    updates.extend(report.binding.updates);
                }
                UiPreparedBindingTarget::Enabled { enabled } => {
                    let report = self.mutate_property(
                        UiPropertyMutationRequest::new(
                            source_node_id,
                            "enabled",
                            UiValue::Bool(enabled),
                        )
                        .with_binding_source_kind(UiBindingSourceKind::ComponentEvent),
                    )?;
                    if report.status == UiPropertyMutationStatus::Rejected {
                        return Ok(Err(report.binding));
                    }
                    summary.record(
                        binding_status(report.status),
                        property_report_impact(&report),
                        report.status == UiPropertyMutationStatus::Accepted,
                    );
                    updates.extend(report.binding.updates);
                }
                UiPreparedBindingTarget::ActionPayload {
                    name,
                    previous,
                    value,
                } => {
                    let source = binding_source(source_node_id, binding_name);
                    let target = UiRuntimeBindingTarget::runtime_state(
                        source_node_id,
                        format!("action_payload.{name}"),
                    );
                    let update = if previous.as_ref() == Some(&value) {
                        UiBindingUpdate::unchanged(source, target, value)
                    } else {
                        UiBindingUpdate::applied(source, target, value)
                    }
                    .with_previous(previous);
                    summary.record(update.status, [UiBindingDirtyDomain::Interaction], false);
                    updates.push(update);
                }
            }
        }
        for update in &mut updates {
            update.source.path = Some(binding_name.to_string());
        }
        Ok(Ok(UiBindingTargetCommit {
            report: UiBindingUpdateReport::from_updates(updates),
            summary,
        }))
    }

    fn commit_class_target(
        &mut self,
        source_node_id: UiNodeId,
        binding_name: &str,
        class: String,
        enabled: bool,
    ) -> Result<UiBindingUpdate, UiTreeError> {
        let metadata = self
            .tree
            .node_mut(source_node_id)
            .ok_or(UiTreeError::MissingNode(source_node_id))?
            .template_metadata
            .as_mut()
            .ok_or(UiTreeError::MissingNode(source_node_id))?;
        let previous = metadata.classes.iter().any(|candidate| candidate == &class);
        if enabled && !previous {
            metadata.classes.push(class.clone());
        } else if !enabled && previous {
            metadata.classes.retain(|candidate| candidate != &class);
        }
        let source = binding_source(source_node_id, binding_name);
        let target =
            UiRuntimeBindingTarget::retained_attribute(source_node_id, format!("class.{class}"));
        if previous == enabled {
            return Ok(
                UiBindingUpdate::unchanged(source, target, UiValue::Bool(enabled))
                    .with_previous(Some(UiValue::Bool(previous))),
            );
        }
        let dirty = UiDirtyFlags {
            style: true,
            layout: true,
            hit_test: true,
            render: true,
            text: true,
            input: true,
            ..UiDirtyFlags::default()
        };
        self.mark_node_dirty(source_node_id, dirty)?;
        let _ = self.apply_runtime_state_style_node(source_node_id, true)?;
        Ok(
            UiBindingUpdate::applied(source, target, UiValue::Bool(enabled))
                .with_previous(Some(UiValue::Bool(previous)))
                .with_dirty_flags(dirty),
        )
    }
}

fn binding_source(source_node_id: UiNodeId, binding_name: &str) -> UiBindingSource {
    UiBindingSource {
        kind: UiBindingSourceKind::ComponentEvent,
        node_id: Some(source_node_id),
        property: None,
        path: Some(binding_name.to_string()),
    }
}

fn binding_status(status: UiPropertyMutationStatus) -> UiBindingUpdateStatus {
    match status {
        UiPropertyMutationStatus::Accepted => UiBindingUpdateStatus::Applied,
        UiPropertyMutationStatus::Unchanged => UiBindingUpdateStatus::Unchanged,
        UiPropertyMutationStatus::Rejected => UiBindingUpdateStatus::Rejected,
    }
}

fn property_report_impact(report: &UiPropertyMutationReport) -> Vec<UiBindingDirtyDomain> {
    let mut impact = report.binding.dirty.clone();
    if report.focus_change.is_some() {
        for domain in [
            UiBindingDirtyDomain::Accessibility,
            UiBindingDirtyDomain::Interaction,
        ] {
            if !impact.contains(&domain) {
                impact.push(domain);
            }
        }
    }
    impact
}

fn rejected_compiled_endpoint_report(
    source_node_id: UiNodeId,
    binding_name: &str,
    message: impl Into<String>,
) -> UiBindingUpdateReport {
    UiBindingUpdateReport::from_updates(vec![UiBindingUpdate::rejected(
        binding_source(source_node_id, binding_name),
        UiRuntimeBindingTarget::runtime_state(source_node_id, "compiled_binding.endpoint"),
        UiValue::Null,
        message,
    )])
}

fn runtime_target(
    source_node_id: UiNodeId,
    kind: UiCompiledBindingTargetKind,
    property: &str,
) -> UiRuntimeBindingTarget {
    match kind {
        UiCompiledBindingTargetKind::Property => {
            UiRuntimeBindingTarget::retained_attribute(source_node_id, property)
        }
        UiCompiledBindingTargetKind::Class => {
            UiRuntimeBindingTarget::retained_attribute(source_node_id, format!("class.{property}"))
        }
        UiCompiledBindingTargetKind::Visibility => {
            UiRuntimeBindingTarget::runtime_state(source_node_id, "visible")
        }
        UiCompiledBindingTargetKind::Enabled => {
            UiRuntimeBindingTarget::runtime_state(source_node_id, "enabled")
        }
        UiCompiledBindingTargetKind::ActionPayload => UiRuntimeBindingTarget::runtime_state(
            source_node_id,
            format!("action_payload.{property}"),
        ),
    }
}

fn compiled_binding_bool(value: &UiValue) -> Option<bool> {
    match value {
        UiValue::Bool(value) => Some(*value),
        _ => None,
    }
}

fn compiled_target_matches_binding(
    binding: &UiCompiledBinding,
    target: &UiCompiledBindingTarget,
    target_index: usize,
) -> bool {
    target.endpoint.generation == binding.handle.generation
        && target.endpoint.node_id == binding.node_id
        && target.endpoint.binding_id == binding.handle.binding_id
        && usize::try_from(target.endpoint.target_index.get()).ok() == Some(target_index)
}
