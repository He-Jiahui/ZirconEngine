use thiserror::Error;
use zircon_runtime::ui::surface::{
    UiSurface, UiVirtualListItemKey, UiVirtualListMaterializationChange,
    UiVirtualListMaterializationError, UiVirtualListPrototypeNodeContext,
    UiVirtualListPrototypePoolError,
};
use zircon_runtime_interface::ui::{event_ui::UiNodeId, tree::UiTemplateNodeMetadata};

use zircon_runtime_interface::ui::v2::{
    UI_V2_REPEAT_ATTRIBUTE, UI_V2_REPEAT_FIELD_AUTHORED_COUNT, UI_V2_REPEAT_FIELD_KIND,
    UI_V2_REPEAT_FIELD_NODE_PATH_NAMESPACE, UI_V2_REPEAT_FIELD_PROTOTYPE,
    UI_V2_REPEAT_FIELD_VIRTUAL_CONTROL_PREFIX, UI_V2_REPEAT_KIND_VIRTUAL_ROWS,
};

#[derive(Debug, Error)]
pub(crate) enum TemplateBridgeVirtualRowsError {
    #[error("template virtual row source is missing required control {control_id}")]
    MissingControl { control_id: String },
    #[error("template virtual row control {control_id} is missing a repeat declaration")]
    MissingRepeatDeclaration { control_id: String },
    #[error("template virtual row control {control_id} has invalid repeat declaration: {detail}")]
    InvalidRepeatDeclaration { control_id: String, detail: String },
    #[error(
        "template virtual row control {control_id} uses unsupported repeat kind {kind}; expected virtual_rows"
    )]
    UnsupportedRepeatKind { control_id: String, kind: String },
    #[error(transparent)]
    Materialization(#[from] UiVirtualListMaterializationError),
    #[error(transparent)]
    PrototypePool(#[from] UiVirtualListPrototypePoolError),
}

/// Runtime-backed virtual row sequence with one authored prototype and a bounded physical pool.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TemplateBridgeVirtualRowSequence {
    parent_control_id: String,
    parent_id: UiNodeId,
    prototype_control_id: String,
    prototype_id: UiNodeId,
    virtual_control_prefix: String,
    node_path_namespace: String,
}

pub(crate) struct TemplateBridgeVirtualRowContext {
    pub(crate) slot_number: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TemplateBridgeVirtualRowBinding {
    pub(crate) logical_index: usize,
    pub(crate) control_id: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TemplateBridgeVirtualRowReconcile {
    pub(crate) topology_changed: bool,
    pub(crate) changes: Vec<UiVirtualListMaterializationChange>,
}

impl TemplateBridgeVirtualRowSequence {
    pub(crate) fn from_surface_repeat(
        surface: &UiSurface,
        parent_control_id: &str,
    ) -> Result<Self, TemplateBridgeVirtualRowsError> {
        let parent_id = required_control_node_id(surface, parent_control_id)?;
        let metadata = surface
            .tree
            .node(parent_id)
            .and_then(|node| node.template_metadata.as_ref())
            .ok_or_else(|| TemplateBridgeVirtualRowsError::MissingControl {
                control_id: parent_control_id.to_string(),
            })?;
        let repeat = metadata
            .attributes
            .get(UI_V2_REPEAT_ATTRIBUTE)
            .ok_or_else(
                || TemplateBridgeVirtualRowsError::MissingRepeatDeclaration {
                    control_id: parent_control_id.to_string(),
                },
            )?;
        let table = repeat.as_table().ok_or_else(|| {
            TemplateBridgeVirtualRowsError::InvalidRepeatDeclaration {
                control_id: parent_control_id.to_string(),
                detail: "repeat must be a table".to_string(),
            }
        })?;
        let kind = required_string_field(parent_control_id, table, UI_V2_REPEAT_FIELD_KIND)?;
        if kind != UI_V2_REPEAT_KIND_VIRTUAL_ROWS {
            return Err(TemplateBridgeVirtualRowsError::UnsupportedRepeatKind {
                control_id: parent_control_id.to_string(),
                kind,
            });
        }
        let authored_count =
            required_usize_field(parent_control_id, table, UI_V2_REPEAT_FIELD_AUTHORED_COUNT)?;
        if authored_count != 1 {
            return Err(TemplateBridgeVirtualRowsError::InvalidRepeatDeclaration {
                control_id: parent_control_id.to_string(),
                detail: format!(
                    "authored_count must be 1 for bounded prototype materialization, got {authored_count}"
                ),
            });
        }
        let prototype_control_id =
            required_string_field(parent_control_id, table, UI_V2_REPEAT_FIELD_PROTOTYPE)?;
        let prototype_id = surface
            .unique_control_node_id(&prototype_control_id)
            .or_else(|| surface.virtual_list_prototype_root_id(parent_id))
            .ok_or_else(|| TemplateBridgeVirtualRowsError::MissingControl {
                control_id: prototype_control_id.clone(),
            })?;
        Ok(Self {
            parent_control_id: parent_control_id.to_string(),
            parent_id,
            prototype_control_id,
            prototype_id,
            virtual_control_prefix: required_string_field(
                parent_control_id,
                table,
                UI_V2_REPEAT_FIELD_VIRTUAL_CONTROL_PREFIX,
            )?,
            node_path_namespace: optional_string_field(
                table,
                UI_V2_REPEAT_FIELD_NODE_PATH_NAMESPACE,
            )
            .unwrap_or_default(),
        })
    }

    pub(crate) fn reconcile_with_keys<K, F>(
        &self,
        surface: &mut UiSurface,
        total_row_count: usize,
        mut item_key_for_logical_index: K,
        mut configure_metadata: F,
    ) -> Result<TemplateBridgeVirtualRowReconcile, TemplateBridgeVirtualRowsError>
    where
        K: FnMut(usize) -> UiVirtualListItemKey,
        F: FnMut(
            UiTemplateNodeMetadata,
            &TemplateBridgeVirtualRowContext,
        ) -> UiTemplateNodeMetadata,
    {
        let mut changes: Vec<UiVirtualListMaterializationChange> = Vec::new();
        surface.reconcile_virtual_list_materialization_with_keys(
            self.parent_id,
            total_row_count,
            &mut changes,
            &mut item_key_for_logical_index,
        )?;
        let report = surface.ensure_virtual_list_prototype_slots(
            self.parent_id,
            self.prototype_id,
            |node, context: UiVirtualListPrototypeNodeContext| {
                if !context.is_slot_root {
                    return;
                }
                let slot_number = context.slot_index + 1;
                let control_id = self.virtual_control_id(slot_number);
                let row_context = TemplateBridgeVirtualRowContext { slot_number };
                let mut metadata = configure_metadata(
                    node.template_metadata.take().unwrap_or_default(),
                    &row_context,
                );
                metadata.control_id = Some(control_id.clone());
                node.template_metadata = Some(metadata);
                node.node_path = self.virtual_node_path(&control_id);
            },
        )?;
        Ok(TemplateBridgeVirtualRowReconcile {
            topology_changed: report.created_slot_count > 0 || report.removed_slot_count > 0,
            changes,
        })
    }

    pub(crate) fn bindings(
        &self,
        surface: &UiSurface,
    ) -> Result<Vec<TemplateBridgeVirtualRowBinding>, TemplateBridgeVirtualRowsError> {
        let roots = surface
            .virtual_list_prototype_slot_roots(self.parent_id)
            .unwrap_or_default();
        let mut bindings = Vec::with_capacity(roots.len());
        for root_id in roots {
            bindings.push(self.binding_for_root(surface, *root_id)?);
        }
        Ok(bindings)
    }

    pub(crate) fn bindings_for_changes(
        &self,
        surface: &UiSurface,
        changes: &[UiVirtualListMaterializationChange],
    ) -> Result<Vec<TemplateBridgeVirtualRowBinding>, TemplateBridgeVirtualRowsError> {
        let roots = surface
            .virtual_list_prototype_slot_roots(self.parent_id)
            .unwrap_or_default();
        let mut bindings = Vec::with_capacity(changes.len().min(roots.len()));
        for change in changes {
            if change.logical_index.is_none() {
                continue;
            }
            let root_id = *roots.get(change.slot_index).ok_or_else(|| {
                TemplateBridgeVirtualRowsError::MissingControl {
                    control_id: self.parent_control_id.clone(),
                }
            })?;
            bindings.push(self.binding_for_root(surface, root_id)?);
        }
        Ok(bindings)
    }

    fn binding_for_root(
        &self,
        surface: &UiSurface,
        root_id: UiNodeId,
    ) -> Result<TemplateBridgeVirtualRowBinding, TemplateBridgeVirtualRowsError> {
        let binding = surface
            .virtual_list_binding_for_node(self.parent_id, root_id)
            .ok_or_else(|| TemplateBridgeVirtualRowsError::MissingControl {
                control_id: self.parent_control_id.clone(),
            })?;
        let control_id = surface
            .tree
            .node(root_id)
            .and_then(|node| node.template_metadata.as_ref())
            .and_then(|metadata| metadata.control_id.clone())
            .ok_or_else(|| TemplateBridgeVirtualRowsError::MissingControl {
                control_id: self.prototype_control_id.clone(),
            })?;
        Ok(TemplateBridgeVirtualRowBinding {
            logical_index: binding.logical_index,
            control_id,
        })
    }

    fn virtual_control_id(&self, slot_number: usize) -> String {
        format!("{}{slot_number:02}", self.virtual_control_prefix)
    }

    fn virtual_node_path(
        &self,
        control_id: &str,
    ) -> zircon_runtime_interface::ui::event_ui::UiNodePath {
        let namespace = self.node_path_namespace.trim_end_matches('/');
        if namespace.is_empty() {
            zircon_runtime_interface::ui::event_ui::UiNodePath::new(control_id.to_string())
        } else {
            zircon_runtime_interface::ui::event_ui::UiNodePath::new(format!(
                "{namespace}/{control_id}"
            ))
        }
    }
}

fn required_control_node_id(
    surface: &UiSurface,
    control_id: &str,
) -> Result<UiNodeId, TemplateBridgeVirtualRowsError> {
    surface.unique_control_node_id(control_id).ok_or_else(|| {
        TemplateBridgeVirtualRowsError::MissingControl {
            control_id: control_id.to_string(),
        }
    })
}

fn required_string_field(
    control_id: &str,
    table: &toml::map::Map<String, toml::Value>,
    field: &str,
) -> Result<String, TemplateBridgeVirtualRowsError> {
    let value = optional_string_field(table, field).ok_or_else(|| {
        TemplateBridgeVirtualRowsError::InvalidRepeatDeclaration {
            control_id: control_id.to_string(),
            detail: format!("missing string field {field}"),
        }
    })?;
    if value.trim().is_empty() {
        return Err(TemplateBridgeVirtualRowsError::InvalidRepeatDeclaration {
            control_id: control_id.to_string(),
            detail: format!("field {field} must not be empty"),
        });
    }
    Ok(value)
}

fn optional_string_field(
    table: &toml::map::Map<String, toml::Value>,
    field: &str,
) -> Option<String> {
    table
        .get(field)
        .and_then(toml::Value::as_str)
        .map(str::to_string)
}

fn required_usize_field(
    control_id: &str,
    table: &toml::map::Map<String, toml::Value>,
    field: &str,
) -> Result<usize, TemplateBridgeVirtualRowsError> {
    let value = table
        .get(field)
        .and_then(toml::Value::as_integer)
        .ok_or_else(
            || TemplateBridgeVirtualRowsError::InvalidRepeatDeclaration {
                control_id: control_id.to_string(),
                detail: format!("missing integer field {field}"),
            },
        )?;
    usize::try_from(value).map_err(
        |_| TemplateBridgeVirtualRowsError::InvalidRepeatDeclaration {
            control_id: control_id.to_string(),
            detail: format!("field {field} must be a non-negative integer"),
        },
    )
}

#[cfg(test)]
mod performance_tests {
    #[test]
    fn virtual_row_reconcile_uses_runtime_materialization_authority() {
        let source = include_str!("virtual_rows.rs");
        let implementation = source.split("#[cfg(test)]").next().expect("implementation");

        assert!(implementation.contains("reconcile_virtual_list_materialization_with_keys"));
        assert!(implementation.contains("ensure_virtual_list_prototype_slots"));
        assert!(!implementation.contains("surface.tree.nodes.iter()"));
        assert!(!implementation.contains("surface.tree.nodes.values()"));
        assert!(!implementation.contains("fn inventory("));
        assert!(!implementation.contains("fn next_node_id("));
    }
}
