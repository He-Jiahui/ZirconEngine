use zircon_runtime_interface::ui::{
    binding::{UiBindingUpdateReport, UiEventKind},
    component::{UiComponentEvent, UiComponentKeyboardAction, UiDragMetrics, UiValue},
    dispatch::{UiPointerComponentEvent, UiPointerComponentEventReason},
    event_ui::UiNodeId,
    surface::{UiPointerActivationPhase, UiPointerEventKind, UiPointerRoute},
    tree::UiTreeError,
};

use crate::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface};

const TREE_REORDER_DRAG_PREFIX: &str = "tree_reorder";

use super::tree_view_reparent::reparent_tree_node_values;
pub(super) use super::tree_view_support::is_default_tree_view_behavior;
use super::tree_view_support::{
    anchor_index, anchor_property, editing_index_property, editing_node_id_property,
    editing_text_property, expanded_ids, expanded_property, is_tree_view_owner, range_selected_ids,
    rename_committed_property, renamed_node_id_property, renamed_text_property, selected_ids,
    selected_property, string_array_value, toggled_selected_ids, tree_editable, tree_item_id,
    tree_multi_select, tree_node_ids, tree_node_ids_for_property, tree_node_label,
    tree_node_values_for_property, tree_nodes_property, tree_option_is_disabled, tree_reorderable,
};

struct UiDefaultTreeViewSelection {
    owner_id: UiNodeId,
    event_property: String,
    selected_property: String,
    option_id: String,
    selected: bool,
    selected_ids: Vec<String>,
    target_index: usize,
    anchor_property: String,
    update_anchor: bool,
    write_value: bool,
}

struct UiDefaultTreeViewRenameEntry {
    owner_id: UiNodeId,
    option_id: String,
    editing_text: String,
    target_index: usize,
    editing_node_id_property: String,
    editing_text_property: String,
    editing_index_property: String,
    rename_committed_property: String,
    renamed_node_id_property: String,
    renamed_text_property: String,
    reason: UiPointerComponentEventReason,
}

struct UiDefaultTreeViewReorderStart {
    owner_id: UiNodeId,
    property: String,
    option_id: String,
    source_index: usize,
}

struct UiDefaultTreeViewReorderDrop {
    owner_id: UiNodeId,
    property: String,
    option_id: String,
    from: usize,
    to: usize,
    values: Vec<UiValue>,
    selected_property: String,
    anchor_property: String,
    expanded_property: Option<String>,
    expanded_ids: Vec<String>,
    update_virtual_window: bool,
}

struct UiTreeReorderDragToken {
    property: String,
    option_id: String,
}

impl UiSurface {
    pub(super) fn apply_default_tree_view_reorder_component_action(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        match route.activation_phase {
            UiPointerActivationPhase::PrimaryPress => {
                let Some(start) = self.default_tree_view_reorder_start(route)? else {
                    return Ok(false);
                };
                self.capture_pointer(start.owner_id)?;
                let drag = self.input.begin_pointer_drag_with_property(
                    start.owner_id,
                    route.point,
                    Some(encode_tree_reorder_drag(
                        &start.property,
                        start.source_index,
                        &start.option_id,
                    )),
                );
                self.push_pointer_component_events_with_drag_metrics(
                    events,
                    start.owner_id,
                    UiEventKind::DragBegin,
                    UiComponentEvent::BeginDrag {
                        property: start.property,
                    },
                    UiPointerComponentEventReason::PressBegin,
                    Some(drag),
                )?;
                Ok(true)
            }
            UiPointerActivationPhase::Hover if matches!(route.kind, UiPointerEventKind::Move) => {
                let Some(owner_id) = route.captured else {
                    return Ok(false);
                };
                if !self
                    .input
                    .pointer_drag_property(owner_id)
                    .and_then(decode_tree_reorder_drag)
                    .is_some()
                {
                    return Ok(false);
                }
                let _drag = self.input.update_pointer_drag(owner_id, route.point);
                Ok(true)
            }
            UiPointerActivationPhase::PrimaryRelease => {
                let Some(owner_id) = route.captured else {
                    return Ok(false);
                };
                let Some(token) = self
                    .input
                    .pointer_drag_property(owner_id)
                    .and_then(decode_tree_reorder_drag)
                else {
                    return Ok(false);
                };
                let end_property = token.property.clone();
                let drag = self.input.end_pointer_drag(owner_id, route.point);
                let Some(drop) = self.default_tree_view_reorder_drop(route, owner_id, token)?
                else {
                    self.push_tree_view_end_drag(events, owner_id, end_property, drag)?;
                    return Ok(false);
                };
                let moved = drop.from != drop.to;
                if moved {
                    let mut changed = false;
                    changed |= self.apply_tree_view_mutation(
                        drop.owner_id,
                        drop.property.clone(),
                        UiValue::Array(drop.values),
                        binding_reports,
                    )?;
                    changed |= self.apply_tree_view_mutation(
                        drop.owner_id,
                        drop.selected_property.clone(),
                        string_array_value(&[drop.option_id.clone()]),
                        binding_reports,
                    )?;
                    changed |= self.apply_tree_view_mutation(
                        drop.owner_id,
                        "focused_index".to_string(),
                        UiValue::Int(drop.to as i64),
                        binding_reports,
                    )?;
                    changed |= self.apply_tree_view_mutation(
                        drop.owner_id,
                        "selected_index".to_string(),
                        UiValue::Int(drop.to as i64),
                        binding_reports,
                    )?;
                    changed |= self.apply_tree_view_mutation(
                        drop.owner_id,
                        drop.anchor_property.clone(),
                        UiValue::Int(drop.to as i64),
                        binding_reports,
                    )?;
                    if let Some(expanded_property) = drop.expanded_property.clone() {
                        changed |= self.apply_tree_view_mutation(
                            drop.owner_id,
                            expanded_property,
                            string_array_value(&drop.expanded_ids),
                            binding_reports,
                        )?;
                    }
                    if drop.selected_property == "selected_items" {
                        changed |= self.apply_tree_view_mutation(
                            drop.owner_id,
                            "value".to_string(),
                            UiValue::String(drop.option_id.clone()),
                            binding_reports,
                        )?;
                    }
                    if changed {
                        self.push_pointer_component_events(
                            events,
                            drop.owner_id,
                            UiEventKind::Change,
                            UiComponentEvent::MoveElement {
                                property: drop.property.clone(),
                                from: drop.from,
                                to: drop.to,
                            },
                            UiPointerComponentEventReason::DefaultClick,
                        )?;
                    }
                    if drop.update_virtual_window {
                        let _ = self.apply_tree_view_virtual_window_for_index(
                            drop.owner_id,
                            drop.to as i64,
                            events,
                            binding_reports,
                        )?;
                    }
                }
                self.push_tree_view_end_drag(events, drop.owner_id, drop.property, drag)?;
                Ok(moved)
            }
            _ => Ok(false),
        }
    }

    pub(super) fn apply_default_tree_view_component_action(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let Some(selection) = self.default_tree_view_selection(route)? else {
            return Ok(false);
        };

        let mut changed = false;
        changed |= self.apply_tree_view_mutation(
            selection.owner_id,
            selection.selected_property.clone(),
            string_array_value(&selection.selected_ids),
            binding_reports,
        )?;
        changed |= self.apply_tree_view_mutation(
            selection.owner_id,
            "focused_index".to_string(),
            UiValue::Int(selection.target_index as i64),
            binding_reports,
        )?;
        changed |= self.apply_tree_view_mutation(
            selection.owner_id,
            "selected_index".to_string(),
            UiValue::Int(selection.target_index as i64),
            binding_reports,
        )?;
        if selection.update_anchor {
            changed |= self.apply_tree_view_mutation(
                selection.owner_id,
                selection.anchor_property.clone(),
                UiValue::Int(selection.target_index as i64),
                binding_reports,
            )?;
        }
        if selection.write_value {
            changed |= self.apply_tree_view_mutation(
                selection.owner_id,
                "value".to_string(),
                UiValue::String(selection.option_id.clone()),
                binding_reports,
            )?;
        }

        if !changed {
            return Ok(false);
        }

        self.push_pointer_component_events(
            events,
            selection.owner_id,
            UiEventKind::Change,
            UiComponentEvent::SelectOption {
                property: selection.event_property,
                option_id: selection.option_id,
                selected: selection.selected,
            },
            UiPointerComponentEventReason::DefaultClick,
        )?;
        Ok(true)
    }

    pub(super) fn apply_default_tree_view_rename_component_action(
        &mut self,
        route: &UiPointerRoute,
        click_count: u8,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let Some(entry) = self.default_tree_view_rename_entry(route, click_count)? else {
            return Ok(false);
        };

        let mut changed = false;
        changed |= self.apply_tree_view_mutation(
            entry.owner_id,
            "editing".to_string(),
            UiValue::Bool(true),
            binding_reports,
        )?;
        changed |= self.apply_tree_view_mutation(
            entry.owner_id,
            entry.editing_node_id_property,
            UiValue::String(entry.option_id),
            binding_reports,
        )?;
        changed |= self.apply_tree_view_mutation(
            entry.owner_id,
            entry.editing_text_property,
            UiValue::String(entry.editing_text),
            binding_reports,
        )?;
        changed |= self.apply_tree_view_mutation(
            entry.owner_id,
            entry.editing_index_property,
            UiValue::Int(entry.target_index as i64),
            binding_reports,
        )?;
        changed |= self.apply_tree_view_mutation(
            entry.owner_id,
            "focused_index".to_string(),
            UiValue::Int(entry.target_index as i64),
            binding_reports,
        )?;
        changed |= self.apply_tree_view_mutation(
            entry.owner_id,
            "selected_index".to_string(),
            UiValue::Int(entry.target_index as i64),
            binding_reports,
        )?;
        changed |= self.apply_tree_view_mutation(
            entry.owner_id,
            entry.rename_committed_property,
            UiValue::Bool(false),
            binding_reports,
        )?;
        changed |= self.apply_tree_view_mutation(
            entry.owner_id,
            entry.renamed_node_id_property,
            UiValue::String(String::new()),
            binding_reports,
        )?;
        changed |= self.apply_tree_view_mutation(
            entry.owner_id,
            entry.renamed_text_property,
            UiValue::String(String::new()),
            binding_reports,
        )?;

        self.push_pointer_component_events_for_component_event_kind(
            events,
            entry.owner_id,
            UiEventKind::Change,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::BeginEdit,
            },
            entry.reason,
        )?;
        Ok(changed || !events.is_empty())
    }

    fn default_tree_view_reorder_start(
        &self,
        route: &UiPointerRoute,
    ) -> Result<Option<UiDefaultTreeViewReorderStart>, UiTreeError> {
        let Some(_) = route.target else {
            return Ok(None);
        };
        let Some(hit) = self.tree_view_hit(route)? else {
            return Ok(None);
        };
        let owner = self
            .tree
            .node(hit.owner_id)
            .ok_or(UiTreeError::MissingNode(hit.owner_id))?;
        let Some(owner_metadata) = owner.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !self.widget_interaction_enabled(hit.owner_id, owner, owner_metadata)
            || !tree_reorderable(owner_metadata)
            || tree_option_is_disabled(owner_metadata, &hit.option_id)
        {
            return Ok(None);
        }
        let Some(property) = tree_nodes_property(owner_metadata) else {
            return Ok(None);
        };
        let node_ids = tree_node_ids_for_property(owner_metadata, property);
        let Some(source_index) = node_ids.iter().position(|id| *id == hit.option_id) else {
            return Ok(None);
        };
        Ok(Some(UiDefaultTreeViewReorderStart {
            owner_id: hit.owner_id,
            property: property.to_string(),
            option_id: hit.option_id,
            source_index,
        }))
    }

    fn default_tree_view_reorder_drop(
        &self,
        route: &UiPointerRoute,
        owner_id: UiNodeId,
        token: UiTreeReorderDragToken,
    ) -> Result<Option<UiDefaultTreeViewReorderDrop>, UiTreeError> {
        let Some(hit) = self.tree_view_hit(route)? else {
            return Ok(None);
        };
        if hit.owner_id != owner_id {
            return Ok(None);
        }
        let owner = self
            .tree
            .node(owner_id)
            .ok_or(UiTreeError::MissingNode(owner_id))?;
        let Some(owner_metadata) = owner.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !self.widget_interaction_enabled(owner_id, owner, owner_metadata)
            || !tree_reorderable(owner_metadata)
            || tree_option_is_disabled(owner_metadata, &token.option_id)
            || tree_option_is_disabled(owner_metadata, &hit.option_id)
        {
            return Ok(None);
        }
        let Some(property) = tree_nodes_property(owner_metadata) else {
            return Ok(None);
        };
        if property != token.property {
            return Ok(None);
        }
        let node_ids = tree_node_ids_for_property(owner_metadata, property);
        let Some(current_source_index) = node_ids.iter().position(|id| *id == token.option_id)
        else {
            return Ok(None);
        };
        let Some(target_index) = node_ids.iter().position(|id| *id == hit.option_id) else {
            return Ok(None);
        };
        let from = current_source_index;
        let to = adjusted_reorder_target_index(from, target_index);
        let mut values = tree_node_values_for_property(owner_metadata, property);
        if values.len() != node_ids.len() {
            let Some(reparented) =
                reparent_tree_node_values(values, &token.option_id, &hit.option_id)
            else {
                return Ok(None);
            };
            let expanded_property = expanded_property(owner_metadata).to_string();
            let mut next_expanded_ids = expanded_ids(owner_metadata, &expanded_property);
            if !next_expanded_ids
                .iter()
                .any(|expanded_id| expanded_id == &reparented.parent_id)
            {
                next_expanded_ids.push(reparented.parent_id);
            }
            return Ok(Some(UiDefaultTreeViewReorderDrop {
                owner_id,
                property: property.to_string(),
                option_id: token.option_id,
                from: reparented.from,
                to: reparented.to,
                values: reparented.values,
                selected_property: selected_property(owner_metadata).to_string(),
                anchor_property: anchor_property(owner_metadata).to_string(),
                expanded_property: Some(expanded_property),
                expanded_ids: next_expanded_ids,
                update_virtual_window: true,
            }));
        }
        if from >= values.len() {
            return Ok(None);
        }
        let value = values.remove(from);
        values.insert(to.min(values.len()), value);
        Ok(Some(UiDefaultTreeViewReorderDrop {
            owner_id,
            property: property.to_string(),
            option_id: token.option_id,
            from,
            to,
            values,
            selected_property: selected_property(owner_metadata).to_string(),
            anchor_property: anchor_property(owner_metadata).to_string(),
            expanded_property: None,
            expanded_ids: Vec::new(),
            update_virtual_window: false,
        }))
    }

    fn push_tree_view_end_drag(
        &self,
        events: &mut Vec<UiPointerComponentEvent>,
        owner_id: UiNodeId,
        property: String,
        drag: UiDragMetrics,
    ) -> Result<(), UiTreeError> {
        self.push_pointer_component_events_with_drag_metrics(
            events,
            owner_id,
            UiEventKind::DragEnd,
            UiComponentEvent::EndDrag { property },
            UiPointerComponentEventReason::PressEnd,
            Some(drag),
        )
    }

    fn default_tree_view_selection(
        &self,
        route: &UiPointerRoute,
    ) -> Result<Option<UiDefaultTreeViewSelection>, UiTreeError> {
        let Some(_) = route.click_target else {
            return Ok(None);
        };

        let Some(hit) = self.tree_view_hit(route)? else {
            return Ok(None);
        };
        let owner = self
            .tree
            .node(hit.owner_id)
            .ok_or(UiTreeError::MissingNode(hit.owner_id))?;
        let Some(owner_metadata) = owner.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !self.widget_interaction_enabled(hit.owner_id, owner, owner_metadata) {
            return Ok(None);
        }
        if tree_option_is_disabled(owner_metadata, hit.option_id.as_str()) {
            return Ok(None);
        }

        let node_ids = tree_node_ids(owner_metadata);
        let Some(target_index) = node_ids.iter().position(|id| *id == hit.option_id) else {
            return Ok(None);
        };

        let selected_property = selected_property(owner_metadata).to_string();
        let current_selected_ids = selected_ids(owner_metadata, selected_property.as_str());
        let multi_select = tree_multi_select(owner_metadata);
        let range_selecting = route.modifiers.shift && multi_select;
        let additive_selecting = route.modifiers.control && multi_select && !range_selecting;
        let selected = if additive_selecting {
            !current_selected_ids.iter().any(|id| id == &hit.option_id)
        } else {
            true
        };
        let selected_ids = if range_selecting {
            range_selected_ids(owner_metadata, &node_ids, target_index)
        } else if additive_selecting {
            toggled_selected_ids(current_selected_ids, hit.option_id.as_str(), selected)
        } else {
            vec![hit.option_id.clone()]
        };

        let anchor_property = anchor_property(owner_metadata).to_string();
        Ok(Some(UiDefaultTreeViewSelection {
            owner_id: hit.owner_id,
            event_property: if range_selecting || additive_selecting {
                selected_property.clone()
            } else {
                "value".to_string()
            },
            selected_property,
            option_id: hit.option_id,
            selected,
            selected_ids,
            target_index,
            anchor_property,
            update_anchor: !range_selecting || anchor_index(owner_metadata).is_none(),
            write_value: !range_selecting && !additive_selecting && selected,
        }))
    }

    fn default_tree_view_rename_entry(
        &self,
        route: &UiPointerRoute,
        click_count: u8,
    ) -> Result<Option<UiDefaultTreeViewRenameEntry>, UiTreeError> {
        let reason = match route.activation_phase {
            UiPointerActivationPhase::PrimaryRelease
                if click_count >= 2 && route.click_target.is_some() =>
            {
                UiPointerComponentEventReason::DefaultDoubleClick
            }
            UiPointerActivationPhase::SecondaryRelease
                if route
                    .pressed
                    .is_some_and(|pressed| route.stacked.contains(&pressed)) =>
            {
                UiPointerComponentEventReason::DefaultClick
            }
            _ => return Ok(None),
        };

        let Some(hit) = self.tree_view_hit(route)? else {
            return Ok(None);
        };
        let owner = self
            .tree
            .node(hit.owner_id)
            .ok_or(UiTreeError::MissingNode(hit.owner_id))?;
        let Some(owner_metadata) = owner.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !self.widget_interaction_enabled(hit.owner_id, owner, owner_metadata) {
            return Ok(None);
        }
        if !tree_editable(owner_metadata) || tree_option_is_disabled(owner_metadata, &hit.option_id)
        {
            return Ok(None);
        }

        let node_ids = tree_node_ids(owner_metadata);
        let Some(target_index) = node_ids.iter().position(|id| *id == hit.option_id) else {
            return Ok(None);
        };
        let editing_text = tree_node_label(owner_metadata, &hit.option_id)
            .unwrap_or_else(|| hit.option_id.clone());

        Ok(Some(UiDefaultTreeViewRenameEntry {
            owner_id: hit.owner_id,
            option_id: hit.option_id,
            editing_text,
            target_index,
            editing_node_id_property: editing_node_id_property(owner_metadata).to_string(),
            editing_text_property: editing_text_property(owner_metadata).to_string(),
            editing_index_property: editing_index_property(owner_metadata).to_string(),
            rename_committed_property: rename_committed_property(owner_metadata).to_string(),
            renamed_node_id_property: renamed_node_id_property(owner_metadata).to_string(),
            renamed_text_property: renamed_text_property(owner_metadata).to_string(),
            reason,
        }))
    }

    fn tree_view_hit(&self, route: &UiPointerRoute) -> Result<Option<TreeViewHit>, UiTreeError> {
        let mut option_id = None;
        let mut blocked = false;
        for node_id in route.hit_candidates() {
            let node = self
                .tree
                .node(node_id)
                .ok_or(UiTreeError::MissingNode(node_id))?;
            if !self.node_interaction_enabled(node_id)? {
                blocked = true;
            }
            let Some(metadata) = node.template_metadata.as_ref() else {
                continue;
            };
            if option_id.is_none() {
                option_id = tree_item_id(metadata);
            }
            blocked |= metadata
                .attributes
                .get("disabled")
                .and_then(toml::Value::as_bool)
                .unwrap_or(false);
            if is_tree_view_owner(metadata) {
                if blocked {
                    return Ok(None);
                }
                return Ok(option_id.map(|option_id| TreeViewHit {
                    owner_id: node_id,
                    option_id,
                }));
            }
        }
        Ok(None)
    }

    fn apply_tree_view_mutation(
        &mut self,
        node_id: UiNodeId,
        property: String,
        value: UiValue,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let report = self.mutate_property(UiPropertyMutationRequest::widget_behavior(
            node_id, property, value,
        ))?;
        if matches!(report.status, UiPropertyMutationStatus::Accepted) {
            binding_reports.push(report.binding);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

struct TreeViewHit {
    owner_id: UiNodeId,
    option_id: String,
}

fn encode_tree_reorder_drag(property: &str, source_index: usize, option_id: &str) -> String {
    format!("{TREE_REORDER_DRAG_PREFIX}:{property}:{source_index}:{option_id}")
}

fn decode_tree_reorder_drag(value: &str) -> Option<UiTreeReorderDragToken> {
    let mut parts = value.splitn(4, ':');
    let prefix = parts.next()?;
    if prefix != TREE_REORDER_DRAG_PREFIX {
        return None;
    }
    let property = parts.next()?.to_string();
    let _source_index = parts.next()?.parse::<usize>().ok()?;
    let option_id = parts.next()?.to_string();
    Some(UiTreeReorderDragToken {
        property,
        option_id,
    })
}

fn adjusted_reorder_target_index(_from: usize, target: usize) -> usize {
    target
}
