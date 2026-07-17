use std::collections::HashSet;

use thiserror::Error;
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath},
    layout::{UiSlot, UiSlotKind},
    tree::{UiDirtyFlags, UiTemplateNodeMetadata, UiTreeError, UiTreeNode},
    v2::{
        UI_V2_REPEAT_ATTRIBUTE, UI_V2_REPEAT_FIELD_AUTHORED_COUNT, UI_V2_REPEAT_FIELD_KIND,
        UI_V2_REPEAT_FIELD_NODE_PATH_NAMESPACE, UI_V2_REPEAT_FIELD_PROTOTYPE,
        UI_V2_REPEAT_FIELD_VIRTUAL_CONTROL_PREFIX, UI_V2_REPEAT_KIND_VIRTUAL_ROWS,
    },
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
    Tree(#[from] UiTreeError),
}

/// Describes a retained template row sequence whose overflow rows are cloned from an authored row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TemplateBridgeVirtualRowSequence {
    parent_control_id: String,
    prototype_control_id: String,
    virtual_control_prefix: String,
    authored_row_count: usize,
    node_path_namespace: String,
}

pub(crate) struct TemplateBridgeVirtualRowContext {
    pub(crate) row_number: usize,
    pub(crate) control_id: String,
}

impl TemplateBridgeVirtualRowSequence {
    pub(crate) fn new(
        parent_control_id: impl Into<String>,
        prototype_control_id: impl Into<String>,
        virtual_control_prefix: impl Into<String>,
        authored_row_count: usize,
        node_path_namespace: impl Into<String>,
    ) -> Self {
        Self {
            parent_control_id: parent_control_id.into(),
            prototype_control_id: prototype_control_id.into(),
            virtual_control_prefix: virtual_control_prefix.into(),
            authored_row_count,
            node_path_namespace: node_path_namespace.into(),
        }
    }

    pub(crate) fn from_surface_repeat(
        surface: &UiSurface,
        parent_control_id: &str,
    ) -> Result<Self, TemplateBridgeVirtualRowsError> {
        let parent_id = required_control_node_id(surface, parent_control_id)?;
        let metadata = surface
            .tree
            .nodes
            .get(&parent_id)
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

        Ok(Self::new(
            parent_control_id,
            required_string_field(parent_control_id, table, UI_V2_REPEAT_FIELD_PROTOTYPE)?,
            required_string_field(
                parent_control_id,
                table,
                UI_V2_REPEAT_FIELD_VIRTUAL_CONTROL_PREFIX,
            )?,
            required_usize_field(parent_control_id, table, UI_V2_REPEAT_FIELD_AUTHORED_COUNT)?,
            optional_string_field(table, UI_V2_REPEAT_FIELD_NODE_PATH_NAMESPACE)
                .unwrap_or_default(),
        ))
    }

    pub(crate) fn reconcile<F>(
        &self,
        surface: &mut UiSurface,
        total_row_count: usize,
        configure_metadata: F,
    ) -> Result<(), TemplateBridgeVirtualRowsError>
    where
        F: FnMut(
            UiTemplateNodeMetadata,
            &TemplateBridgeVirtualRowContext,
        ) -> UiTemplateNodeMetadata,
    {
        let required_virtual_count = total_row_count.saturating_sub(self.authored_row_count);
        self.prune(surface, required_virtual_count)?;
        self.ensure(surface, required_virtual_count, configure_metadata)
    }

    pub(crate) fn virtual_control_ids(&self, surface: &UiSurface) -> Vec<String> {
        let mut controls = surface
            .tree
            .nodes
            .values()
            .filter_map(|node| {
                let control_id = node
                    .template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.control_id.as_deref())?;
                self.virtual_row_number(control_id)
                    .map(|row_number| (row_number, control_id.to_string()))
            })
            .collect::<Vec<_>>();
        controls.sort_by_key(|(row_number, _)| *row_number);
        controls
            .into_iter()
            .map(|(_, control_id)| control_id)
            .collect()
    }

    pub(crate) fn contains_control(&self, surface: &UiSurface, control_id: &str) -> bool {
        self.virtual_row_number(control_id)
            .is_some_and(|row_number| {
                row_number > self.authored_row_count
                    && surface.tree.nodes.values().any(|node| {
                        node.template_metadata
                            .as_ref()
                            .and_then(|metadata| metadata.control_id.as_deref())
                            == Some(control_id)
                    })
            })
    }

    fn ensure<F>(
        &self,
        surface: &mut UiSurface,
        required_virtual_count: usize,
        mut configure_metadata: F,
    ) -> Result<(), TemplateBridgeVirtualRowsError>
    where
        F: FnMut(
            UiTemplateNodeMetadata,
            &TemplateBridgeVirtualRowContext,
        ) -> UiTemplateNodeMetadata,
    {
        if required_virtual_count == 0 {
            return Ok(());
        }

        let parent_id = required_control_node_id(surface, &self.parent_control_id)?;
        let prototype_id = required_control_node_id(surface, &self.prototype_control_id)?;
        let prototype_node = surface
            .tree
            .nodes
            .get(&prototype_id)
            .cloned()
            .ok_or_else(|| TemplateBridgeVirtualRowsError::MissingControl {
                control_id: self.prototype_control_id.clone(),
            })?;
        let prototype_slot = prototype_slot(surface, parent_id, prototype_id);
        let existing_virtual_rows = surface
            .tree
            .nodes
            .values()
            .filter_map(|node| {
                node.template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.control_id.as_deref())
                    .and_then(|control_id| self.virtual_row_number(control_id))
            })
            .collect::<HashSet<_>>();
        let mut next_node_id = next_node_id(surface).0;

        for virtual_index in 0..required_virtual_count {
            let row_number = self.authored_row_count + virtual_index + 1;
            let control_id = self.virtual_control_id(row_number);
            if existing_virtual_rows.contains(&row_number) {
                continue;
            }
            let context = TemplateBridgeVirtualRowContext {
                row_number,
                control_id,
            };
            let node_id = UiNodeId::new(next_node_id);
            next_node_id = next_node_id.saturating_add(1);
            let row = self.virtual_row_from_prototype(
                prototype_node.clone(),
                node_id,
                &context,
                &mut configure_metadata,
            );
            surface.insert_or_reuse_pooled_child(parent_id, row)?;
            surface.tree.slots.push(slot_for_virtual_row(
                prototype_slot.clone(),
                parent_id,
                node_id,
            ));
        }

        Ok(())
    }

    fn prune(
        &self,
        surface: &mut UiSurface,
        required_virtual_count: usize,
    ) -> Result<(), TemplateBridgeVirtualRowsError> {
        let retained_max_row_number = self.authored_row_count + required_virtual_count;
        let mut stale_rows = surface
            .tree
            .nodes
            .values()
            .filter_map(|node| {
                let control_id = node
                    .template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.control_id.as_deref())?;
                let row_number = self.virtual_row_number(control_id)?;
                (row_number > retained_max_row_number).then_some((row_number, node.node_id))
            })
            .collect::<Vec<_>>();
        stale_rows.sort_by(|left, right| right.0.cmp(&left.0));

        for (_, node_id) in stale_rows {
            let _ = surface.detach_subtree_to_pool(node_id)?;
        }
        Ok(())
    }

    fn virtual_row_from_prototype<F>(
        &self,
        mut node: UiTreeNode,
        node_id: UiNodeId,
        context: &TemplateBridgeVirtualRowContext,
        configure_metadata: &mut F,
    ) -> UiTreeNode
    where
        F: FnMut(
            UiTemplateNodeMetadata,
            &TemplateBridgeVirtualRowContext,
        ) -> UiTemplateNodeMetadata,
    {
        node.node_id = node_id;
        node.node_path = self.virtual_node_path(&context.control_id);
        node.parent = None;
        node.children.clear();
        node.dirty = structure_dirty_flags();
        node.layout_cache = Default::default();

        let mut metadata = node.template_metadata.unwrap_or_default();
        metadata.control_id = Some(context.control_id.clone());
        let mut metadata = configure_metadata(metadata, context);
        metadata.control_id = Some(context.control_id.clone());
        node.template_metadata = Some(metadata);
        node
    }

    fn virtual_control_id(&self, row_number: usize) -> String {
        format!("{}{row_number:02}", self.virtual_control_prefix)
    }

    fn virtual_row_number(&self, control_id: &str) -> Option<usize> {
        control_id
            .strip_prefix(&self.virtual_control_prefix)?
            .parse::<usize>()
            .ok()
    }

    fn virtual_node_path(&self, control_id: &str) -> UiNodePath {
        let namespace = self.node_path_namespace.trim_end_matches('/');
        if namespace.is_empty() {
            UiNodePath::new(control_id.to_string())
        } else {
            UiNodePath::new(format!("{namespace}/{control_id}"))
        }
    }
}

fn prototype_slot(surface: &UiSurface, parent_id: UiNodeId, prototype_id: UiNodeId) -> UiSlot {
    surface
        .tree
        .slots
        .iter()
        .find(|slot| slot.parent_id == parent_id && slot.child_id == prototype_id)
        .cloned()
        .unwrap_or_else(|| UiSlot::new(parent_id, prototype_id, UiSlotKind::Linear))
}

fn slot_for_virtual_row(mut slot: UiSlot, parent_id: UiNodeId, child_id: UiNodeId) -> UiSlot {
    slot.parent_id = parent_id;
    slot.child_id = child_id;
    slot
}

fn required_control_node_id(
    surface: &UiSurface,
    control_id: &str,
) -> Result<UiNodeId, TemplateBridgeVirtualRowsError> {
    control_node_id(surface, control_id).ok_or_else(|| {
        TemplateBridgeVirtualRowsError::MissingControl {
            control_id: control_id.to_string(),
        }
    })
}

fn control_node_id(surface: &UiSurface, control_id: &str) -> Option<UiNodeId> {
    surface.tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            .filter(|candidate| *candidate == control_id)
            .map(|_| node.node_id)
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

fn next_node_id(surface: &UiSurface) -> UiNodeId {
    let next = surface
        .tree
        .nodes
        .keys()
        .map(|node_id| node_id.0)
        .max()
        .unwrap_or_default()
        .saturating_add(1);
    UiNodeId::new(next)
}

fn structure_dirty_flags() -> UiDirtyFlags {
    UiDirtyFlags {
        layout: true,
        hit_test: true,
        render: true,
        input: true,
        text: true,
        ..UiDirtyFlags::default()
    }
}

#[cfg(test)]
mod performance_tests {
    #[test]
    fn virtual_row_growth_indexes_existing_rows_and_node_ids_once() {
        let source = include_str!("virtual_rows.rs");
        let implementation = source.split("#[cfg(test)]").next().expect("implementation");
        let ensure = implementation
            .split("    fn ensure<F>(")
            .nth(1)
            .and_then(|body| body.split("    fn prune(").next())
            .expect("ensure implementation");

        assert!(ensure.contains("existing_virtual_rows"));
        assert!(ensure.contains("let mut next_node_id = next_node_id(surface).0"));
        assert!(!ensure.contains("control_node_id(surface, &control_id)"));
        assert_eq!(ensure.matches("next_node_id(surface)").count(), 1);
    }
}
