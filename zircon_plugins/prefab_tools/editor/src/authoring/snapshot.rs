use std::collections::{btree_map::Entry, BTreeMap};

use zircon_runtime::asset::{
    PrefabAsset, PrefabInstanceAsset, PrefabPropertyOverrideAsset, SceneEntityAsset,
};

use super::PrefabSourceComponentDescriptor;

/// The caller's source lookup result; availability does not imply preview readiness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrefabSourceState {
    Unchecked,
    Available,
    Missing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrefabAuthoringDiagnosticKind {
    MissingSource,
    EmptyEntityPath,
    EmptyPropertyPath,
    DuplicateOverridePath,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrefabAuthoringDiagnostic {
    pub kind: PrefabAuthoringDiagnosticKind,
    /// Indices into this snapshot's original override vector, never persistent IDs.
    /// Empty for diagnostics about the instance as a whole.
    pub source_indices: Vec<usize>,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrefabOverrideRow<'a> {
    /// Valid only for the borrowed instance. Mutation needs a document revision too.
    pub source_index: usize,
    pub override_value: &'a PrefabPropertyOverrideAsset,
}

/// A read-only projection of a supplied instance, without another authoring store.
#[derive(Debug)]
pub struct PrefabInstanceAuthoringSnapshot<'a> {
    pub instance: &'a PrefabInstanceAsset,
    pub source_state: PrefabSourceState,
    /// Serialized order, including all conflicting values for duplicate paths.
    pub override_rows: Vec<PrefabOverrideRow<'a>>,
    pub diagnostics: Vec<PrefabAuthoringDiagnostic>,
}

impl<'a> PrefabInstanceAuthoringSnapshot<'a> {
    pub fn new(instance: &'a PrefabInstanceAsset, source_state: PrefabSourceState) -> Self {
        Self {
            instance,
            source_state,
            override_rows: instance
                .overrides
                .iter()
                .enumerate()
                .map(|(source_index, override_value)| PrefabOverrideRow {
                    source_index,
                    override_value,
                })
                .collect(),
            diagnostics: prefab_instance_diagnostics(instance, source_state),
        }
    }
}

/// Entity hierarchy only; this is not component attachment or a mutation target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrefabSourceEntityParent {
    Root,
    /// An index into the same borrowed source's entity rows, never a durable ID.
    Resolved {
        source_index: usize,
    },
    Missing,
    Ambiguous,
    /// No traversable parent edge is published for any member of a cycle.
    Cyclic,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrefabSourceEntityRow<'a> {
    pub source_index: usize,
    /// Keeps native fields and opaque component rows/references without decoding.
    pub entity: &'a SceneEntityAsset,
    pub parent: PrefabSourceEntityParent,
}

/// Display order for the source entity forest, independent of view expansion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrefabSourceHierarchyRow {
    /// Index into this snapshot's entity rows, never a persistent tree item key.
    pub source_index: usize,
    pub depth: usize,
    /// Only uniquely resolved, non-cyclic parent edges count as children.
    pub has_children: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrefabSourceDiagnosticKind {
    DuplicateEntityId,
    MissingParent,
    AmbiguousParent,
    CyclicHierarchy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrefabSourceDiagnostic {
    pub kind: PrefabSourceDiagnosticKind,
    /// Original source entity positions, scoped to this borrow, not override rows.
    pub source_indices: Vec<usize>,
    pub message: String,
}

/// Records that cannot be selected using only the native component descriptor.
///
/// Positions address this borrow's original entity/component vectors, never
/// persistent IDs. This does not declare the records invalid or compare their
/// payloads: different opaque values/references cannot disambiguate identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrefabSourceComponentConflict {
    pub entity_source_index: usize,
    /// All matching records in serialized order, including the first occurrence.
    pub component_source_indices: Vec<usize>,
}

/// Lossless source-scene rows for an already obtained prefab payload.
///
/// No loading, availability or write authority is inferred. A caller using a
/// resident snapshot must retain/revalidate its native version key and owning
/// project/document/view. Entity IDs are meaningful within that source only;
/// duplicate IDs remain visible and cannot identify an unambiguous target.
#[derive(Debug)]
pub struct PrefabSourceAuthoringSnapshot<'a> {
    pub source: &'a PrefabAsset,
    /// Serialized order, including duplicate IDs/names and unresolved parents.
    pub entity_rows: Vec<PrefabSourceEntityRow<'a>>,
    /// Preorder with roots and siblings in serialized order. Unresolved parents
    /// and cut cycle members are roots; their uniquely resolved children remain.
    pub hierarchy_rows: Vec<PrefabSourceHierarchyRow>,
    pub diagnostics: Vec<PrefabSourceDiagnostic>,
    /// Descriptor ambiguities within each serialized entity, ordered by owner
    /// position and first occurrence. Duplicate entity IDs are not merged.
    pub component_conflicts: Vec<PrefabSourceComponentConflict>,
}

impl<'a> PrefabSourceAuthoringSnapshot<'a> {
    pub fn new(source: &'a PrefabAsset) -> Self {
        let mut indices_by_id: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
        for (source_index, entity) in source.scene.entities.iter().enumerate() {
            indices_by_id
                .entry(entity.entity)
                .or_default()
                .push(source_index);
        }
        let mut diagnostics = Vec::new();
        for (entity_id, indices) in &indices_by_id {
            if indices.len() > 1 {
                diagnostics.push(PrefabSourceDiagnostic {
                    kind: PrefabSourceDiagnosticKind::DuplicateEntityId,
                    source_indices: indices.clone(),
                    message: format!("prefab source entity ID {entity_id} is duplicated"),
                });
            }
        }
        let mut entity_rows: Vec<_> = source
            .scene
            .entities
            .iter()
            .enumerate()
            .map(|(source_index, entity)| {
                let parent = match entity.parent {
                    None => PrefabSourceEntityParent::Root,
                    Some(parent_id) => match indices_by_id.get(&parent_id).map(Vec::as_slice) {
                        Some([parent_index]) => PrefabSourceEntityParent::Resolved {
                            source_index: *parent_index,
                        },
                        candidates => {
                            let (parent, kind, detail) = if candidates.is_some() {
                                (
                                    PrefabSourceEntityParent::Ambiguous,
                                    PrefabSourceDiagnosticKind::AmbiguousParent,
                                    "is ambiguous",
                                )
                            } else {
                                (
                                    PrefabSourceEntityParent::Missing,
                                    PrefabSourceDiagnosticKind::MissingParent,
                                    "does not exist",
                                )
                            };
                            diagnostics.push(PrefabSourceDiagnostic {
                                kind,
                                source_indices: vec![source_index],
                                message: format!(
                                    "prefab source entity {} parent {parent_id} {detail}",
                                    entity.entity
                                ),
                            });
                            parent
                        }
                    },
                };
                PrefabSourceEntityRow {
                    source_index,
                    entity,
                    parent,
                }
            })
            .collect();
        mark_source_parent_cycles(&mut entity_rows, &mut diagnostics);
        let hierarchy_rows = source_hierarchy_rows(&entity_rows);
        Self {
            source,
            entity_rows,
            hierarchy_rows,
            diagnostics,
            component_conflicts: source_component_conflicts(source),
        }
    }
}

fn source_component_conflicts(source: &PrefabAsset) -> Vec<PrefabSourceComponentConflict> {
    let mut result = Vec::new();
    for (entity_source_index, entity) in source.scene.entities.iter().enumerate() {
        let mut first_indices = BTreeMap::new();
        let mut conflicts = BTreeMap::new();
        for (component_source_index, record) in entity.components.iter().enumerate() {
            let first = match first_indices.entry(PrefabSourceComponentDescriptor::from(record)) {
                Entry::Vacant(entry) => {
                    entry.insert(component_source_index);
                    continue;
                }
                Entry::Occupied(entry) => *entry.get(),
            };
            conflicts
                .entry(first)
                .or_insert_with(|| vec![first])
                .push(component_source_index);
        }
        result.extend(conflicts.into_values().map(|component_source_indices| {
            PrefabSourceComponentConflict {
                entity_source_index,
                component_source_indices,
            }
        }));
    }
    result
}

fn source_hierarchy_rows(rows: &[PrefabSourceEntityRow<'_>]) -> Vec<PrefabSourceHierarchyRow> {
    // The constructor has resolved unique parent indices and cut every cycle.
    // Traverse that forest iteratively without sorting or changing source rows.
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); rows.len()];
    let mut roots = Vec::new();
    for row in rows {
        match row.parent {
            PrefabSourceEntityParent::Resolved { source_index } => {
                children[source_index].push(row.source_index);
            }
            _ => roots.push(row.source_index),
        }
    }
    let mut pending: Vec<_> = roots.into_iter().rev().map(|index| (index, 0)).collect();
    let mut hierarchy = Vec::with_capacity(rows.len());
    while let Some((source_index, depth)) = pending.pop() {
        hierarchy.push(PrefabSourceHierarchyRow {
            source_index,
            depth,
            has_children: !children[source_index].is_empty(),
        });
        pending.extend(
            children[source_index]
                .iter()
                .rev()
                .map(|&child| (child, depth + 1)),
        );
    }
    hierarchy
}

fn mark_source_parent_cycles(
    rows: &mut [PrefabSourceEntityRow<'_>],
    diagnostics: &mut Vec<PrefabSourceDiagnostic>,
) {
    // Iterative functional-graph walk: every row is visited once, so deep source
    // hierarchies cannot exhaust the call stack. Only unique parent IDs are edges.
    #[derive(Clone, Copy)]
    enum Visit {
        Unseen,
        OnPath(usize),
        Done,
    }
    let mut visits = vec![Visit::Unseen; rows.len()];
    for start in 0..rows.len() {
        if !matches!(visits[start], Visit::Unseen) {
            continue;
        }
        let mut path = Vec::new();
        let mut current = Some(start);
        while let Some(index) = current {
            match visits[index] {
                Visit::Done => break,
                Visit::OnPath(cycle_start) => {
                    let mut source_indices = path[cycle_start..].to_vec();
                    source_indices.sort_unstable();
                    for &cycle_index in &source_indices {
                        rows[cycle_index].parent = PrefabSourceEntityParent::Cyclic;
                    }
                    diagnostics.push(PrefabSourceDiagnostic {
                        kind: PrefabSourceDiagnosticKind::CyclicHierarchy,
                        source_indices,
                        message: "prefab source entity parent hierarchy contains a cycle".into(),
                    });
                    break;
                }
                Visit::Unseen => visits[index] = Visit::OnPath(path.len()),
            }
            path.push(index);
            current = match rows[index].parent {
                PrefabSourceEntityParent::Resolved { source_index } => Some(source_index),
                _ => None,
            };
        }
        for index in path {
            visits[index] = Visit::Done;
        }
    }
}

pub(super) fn prefab_instance_diagnostics(
    instance: &PrefabInstanceAsset,
    source_state: PrefabSourceState,
) -> Vec<PrefabAuthoringDiagnostic> {
    let mut diagnostics = Vec::new();
    if source_state == PrefabSourceState::Missing {
        diagnostics.push(PrefabAuthoringDiagnostic {
            kind: PrefabAuthoringDiagnosticKind::MissingSource,
            source_indices: Vec::new(),
            message: format!(
                "prefab instance source `{}` is not available",
                instance.prefab
            ),
        });
    }

    let mut first_indices = BTreeMap::new();
    let mut conflicts: BTreeMap<(&str, &str), Vec<usize>> = BTreeMap::new();
    for (source_index, override_value) in instance.overrides.iter().enumerate() {
        if override_value.entity_path.trim().is_empty() {
            diagnostics.push(PrefabAuthoringDiagnostic {
                kind: PrefabAuthoringDiagnosticKind::EmptyEntityPath,
                source_indices: vec![source_index],
                message: "prefab override entity path must not be empty".to_string(),
            });
        }
        if override_value.property_path.trim().is_empty() {
            diagnostics.push(PrefabAuthoringDiagnostic {
                kind: PrefabAuthoringDiagnosticKind::EmptyPropertyPath,
                source_indices: vec![source_index],
                message: "prefab override property path must not be empty".to_string(),
            });
        }
        let key = (
            override_value.entity_path.as_str(),
            override_value.property_path.as_str(),
        );
        match first_indices.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(source_index);
            }
            Entry::Occupied(entry) => {
                conflicts
                    .entry(key)
                    .or_insert_with(|| vec![*entry.get()])
                    .push(source_index);
            }
        }
    }
    for ((entity_path, property_path), source_indices) in conflicts {
        diagnostics.push(PrefabAuthoringDiagnostic {
            kind: PrefabAuthoringDiagnosticKind::DuplicateOverridePath,
            source_indices,
            message: format!(
                "duplicate prefab override `{entity_path}` / `{property_path}` is ambiguous"
            ),
        });
    }
    diagnostics
}
