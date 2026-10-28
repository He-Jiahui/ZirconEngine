use std::collections::BTreeSet;

use zircon_runtime::core::math::Real;
use zircon_runtime_interface::resource::ResourceKind;

use super::{
    AssetReference, MaterialGraphAsset, MaterialGraphDiagnostic, MaterialGraphNodeAsset,
    MaterialGraphNodeKindAsset,
};

/// An object to inspect in this exact authoring source, not a compiled material.
///
/// UE inspects the material for an empty selection or the Output root, and the
/// original expression for a regular node. `Graph` retains the actual authored
/// graph and its shader/override presence. The real Toolkit must resolve that
/// source through its current document; this does not invent MaterialAsset or
/// Material Function objects, property schemas or a flattened effective value.
#[derive(Debug)]
pub enum MaterialGraphDetailsTarget<'a> {
    Graph(&'a MaterialGraphAsset),
    Expression(&'a MaterialGraphNodeAsset),
}

/// A common authored value, or a field whose selected values differ.
///
/// Mixed is not zero, an absent override or the first selected value. The UI
/// supplies its localized multiple-values caption without writing that caption
/// into the source. Numeric comparison below preserves authored bit patterns.
#[derive(Clone, Copy, Debug)]
pub enum MaterialGraphDetailsValue<T> {
    Common(T),
    Mixed,
}

impl<T> MaterialGraphDetailsValue<T> {
    fn retain_when(self, equal: impl FnOnce(&T) -> bool) -> Self {
        match self {
            Self::Common(value) if equal(&value) => Self::Common(value),
            _ => Self::Mixed,
        }
    }
}

/// One expression field shared by every selected node's actual kind.
///
/// Vectors preserve common and mixed channels independently. Texture Common
/// borrows the first original reference after comparing both UUID and locator.
#[derive(Clone, Copy, Debug)]
pub enum MaterialGraphExpressionDetailsValues<'a> {
    ScalarDefault(MaterialGraphDetailsValue<Real>),
    VectorDefault([MaterialGraphDetailsValue<Real>; 4]),
    Texture(MaterialGraphDetailsValue<&'a AssetReference>),
}

/// Source-borrowed Details objects and the original selected nodes.
///
/// Build on selection change, not for every paint. Source order is stable even
/// when the input selection has a different order. A graph Details target is
/// included once, while every selected root keeps its own original identity.
///
/// This is neither a selection store nor mutation/focus authority. Before
/// publishing to its existing Details view, the host must revalidate the same
/// view incarnation, document/subdocument, source revision and selection request.
/// SetObjects, focus, diagnostics navigation and editing/undo stay with that
/// host's existing retained input and prepare/permit/CAS/history lifecycle.
#[derive(Debug)]
pub struct MaterialGraphDetailsSelection<'a> {
    source: &'a MaterialGraphAsset,
    selected_nodes: Vec<&'a MaterialGraphNodeAsset>,
    targets: Vec<MaterialGraphDetailsTarget<'a>>,
}

impl<'a> MaterialGraphDetailsSelection<'a> {
    pub fn source(&self) -> &'a MaterialGraphAsset {
        self.source
    }

    pub fn selected_nodes(&self) -> &[&'a MaterialGraphNodeAsset] {
        &self.selected_nodes
    }

    pub fn targets(&self) -> &[MaterialGraphDetailsTarget<'a>] {
        &self.targets
    }

    pub fn has_expression(&self) -> bool {
        self.targets
            .iter()
            .any(|target| matches!(target, MaterialGraphDetailsTarget::Expression(_)))
    }

    /// Projects defaults/references from these exact selected source nodes.
    ///
    /// None means no common expression-field schema: empty selection, a root,
    /// a math node or different expression kinds. Mixed retains that field's
    /// schema while withholding a uniform value. Unselected nodes and graph
    /// parameter overrides do not participate. Non-finite source values remain
    /// visible for repair; this is not the validation of a proposed edit.
    ///
    /// This walks the borrowed selection without allocating, resolving assets,
    /// changing source order or creating transaction/focus authority. The real
    /// host must still revalidate its document/revision/selection on publication
    /// and use its existing single transaction for any subsequent multi-edit.
    pub fn expression_values(&self) -> Option<MaterialGraphExpressionDetailsValues<'a>> {
        use MaterialGraphDetailsValue::Common;
        use MaterialGraphExpressionDetailsValues as Values;
        let first = *self.selected_nodes.first()?;
        match &first.kind {
            MaterialGraphNodeKindAsset::ScalarParameter { default, .. } => {
                let mut value = Common(*default);
                for node in self.selected_nodes.iter().skip(1) {
                    let MaterialGraphNodeKindAsset::ScalarParameter { default, .. } = &node.kind
                    else {
                        return None;
                    };
                    value = value.retain_when(|previous| previous.to_bits() == default.to_bits());
                }
                Some(Values::ScalarDefault(value))
            }
            MaterialGraphNodeKindAsset::VectorParameter { default, .. } => {
                let mut values = (*default).map(Common);
                for node in self.selected_nodes.iter().skip(1) {
                    let MaterialGraphNodeKindAsset::VectorParameter { default, .. } = &node.kind
                    else {
                        return None;
                    };
                    for (value, channel) in values.iter_mut().zip(default) {
                        *value = (*value)
                            .retain_when(|previous| previous.to_bits() == channel.to_bits());
                    }
                }
                Some(Values::VectorDefault(values))
            }
            MaterialGraphNodeKindAsset::TextureSample { texture } => {
                let mut value = Common(texture);
                for node in self.selected_nodes.iter().skip(1) {
                    let MaterialGraphNodeKindAsset::TextureSample { texture } = &node.kind else {
                        return None;
                    };
                    value = value.retain_when(|previous| *previous == texture);
                }
                Some(Values::Texture(value))
            }
            _ => None,
        }
    }
}

/// Resolves exact current node keys without compiling or changing the source.
///
/// Stale, repeated or ambiguous identities reject the whole projection: do not
/// publish a partial selection or fall back to a different document. Empty
/// selection can still inspect the graph when its node identities are invalid,
/// and incomplete/cyclic/ill-typed expressions remain inspectable for repair.
pub fn project_material_graph_details_selection<'a>(
    graph: &'a MaterialGraphAsset,
    selected_node_ids: &[&str],
) -> Result<MaterialGraphDetailsSelection<'a>, Vec<MaterialGraphDiagnostic>> {
    if selected_node_ids.is_empty() {
        return Ok(MaterialGraphDetailsSelection {
            source: graph,
            selected_nodes: Vec::new(),
            targets: vec![MaterialGraphDetailsTarget::Graph(graph)],
        });
    }

    let mut selected = BTreeSet::new();
    for id in selected_node_ids {
        if id.trim().is_empty() || !selected.insert(*id) {
            return Err(vec![MaterialGraphDiagnostic::graph(
                "material graph Details requires distinct non-empty selected node IDs",
            )]);
        }
    }

    let mut known = BTreeSet::new();
    let mut selection = MaterialGraphDetailsSelection {
        source: graph,
        selected_nodes: Vec::new(),
        targets: Vec::new(),
    };
    let mut includes_graph = false;
    for node in &graph.nodes {
        if node.id.trim().is_empty() || !known.insert(node.id.as_str()) {
            return Err(vec![MaterialGraphDiagnostic::graph(
                "material graph Details requires unambiguous non-empty source node IDs",
            )]);
        }
        if !selected.contains(node.id.as_str()) {
            continue;
        }
        selection.selected_nodes.push(node);
        match &node.kind {
            MaterialGraphNodeKindAsset::Output => {
                if !includes_graph {
                    selection
                        .targets
                        .push(MaterialGraphDetailsTarget::Graph(graph));
                    includes_graph = true;
                }
            }
            MaterialGraphNodeKindAsset::ScalarParameter { .. }
            | MaterialGraphNodeKindAsset::VectorParameter { .. }
            | MaterialGraphNodeKindAsset::TextureSample { .. }
            | MaterialGraphNodeKindAsset::Add
            | MaterialGraphNodeKindAsset::Multiply => selection
                .targets
                .push(MaterialGraphDetailsTarget::Expression(node)),
        }
    }
    if !selected.is_subset(&known) {
        return Err(vec![MaterialGraphDiagnostic::graph(
            "material graph Details selection contains a node that is no longer present",
        )]);
    }
    Ok(selection)
}

/// One actual graph-level authoring field; values borrow the source or request.
///
/// Name is a String in the asset schema, so an empty value remains an explicit
/// empty name. Shader None clears that optional authored reference; it does not
/// install the compiler's fallback into the source. No UE property or editor
/// metadata is inferred from these two Zircon fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaterialGraphDetailsEdit<'a> {
    Name(&'a str),
    Shader(Option<&'a AssetReference>),
}

/// A source-borrowed candidate for the host's existing single-field transaction.
///
/// This owns no document, revision, delta, history or mutation permit. The host
/// must retain the same document/subdocument/revision and resource catalog,
/// revalidate the request, charge/copy its before and after values, then prepare
/// one domain delta and inverse through the existing retention/permit/CAS path.
/// UI focus, dirty state, compile/preview invalidation and save stay with that
/// real host; merely constructing this candidate changes none of them.
#[derive(Debug)]
pub struct MaterialGraphDetailsEditPlan<'a> {
    source: &'a MaterialGraphAsset,
    edit: MaterialGraphDetailsEdit<'a>,
}

impl<'a> MaterialGraphDetailsEditPlan<'a> {
    pub fn source(&self) -> &'a MaterialGraphAsset {
        self.source
    }

    pub fn edit(&self) -> MaterialGraphDetailsEdit<'a> {
        self.edit
    }

    pub fn previous_value(&self) -> MaterialGraphDetailsEdit<'a> {
        match self.edit {
            MaterialGraphDetailsEdit::Name(_) => MaterialGraphDetailsEdit::Name(&self.source.name),
            MaterialGraphDetailsEdit::Shader(_) => {
                MaterialGraphDetailsEdit::Shader(self.source.shader.as_ref())
            }
        }
    }

    pub fn changes_source(&self) -> bool {
        self.edit != self.previous_value()
    }
}

/// Plans a Name or Shader edit without changing the graph or compiling it.
///
/// Compare the complete reference pair, including UUID and locator. A changed
/// non-empty Shader is resolved once through the caller's current catalog;
/// neither its extension nor the previous reference proves its resource kind.
/// Unchanged values and clearing a reference need no lookup. Invalid expressions
/// remain repairable: unrelated node/link/parameter errors do not block a graph
/// field edit, and this candidate never resets those authoring values.
pub fn plan_material_graph_details_edit<'a>(
    graph: &'a MaterialGraphAsset,
    edit: MaterialGraphDetailsEdit<'a>,
    mut resolve_kind: impl FnMut(&AssetReference) -> Result<ResourceKind, String>,
) -> Result<MaterialGraphDetailsEditPlan<'a>, Vec<MaterialGraphDiagnostic>> {
    let plan = MaterialGraphDetailsEditPlan {
        source: graph,
        edit,
    };
    if plan.changes_source() {
        if let MaterialGraphDetailsEdit::Shader(Some(reference)) = edit {
            match resolve_kind(reference) {
                Ok(ResourceKind::Shader) => {}
                Ok(kind) => {
                    return Err(vec![MaterialGraphDiagnostic::graph(format!(
                        "material graph Shader references {kind:?}, expected Shader"
                    ))]);
                }
                Err(error) => {
                    return Err(vec![MaterialGraphDiagnostic::graph(format!(
                        "material graph Shader reference could not be resolved: {error}"
                    ))]);
                }
            }
        }
    }
    Ok(plan)
}

/// One field actually stored on an expression in the current asset schema.
///
/// Defaults belong to the selected node, not to the graph parameter override
/// map. A required TextureSample reference has no invented None/reset value.
/// Parameter-name remapping, grouping and UE-only properties are not inferred.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MaterialGraphExpressionDetailsEdit<'a> {
    ScalarDefault(Real),
    VectorDefault([Real; 4]),
    Texture(&'a AssetReference),
}

#[derive(Debug)]
enum ExpressionFieldValues<'a> {
    Scalar {
        before: Real,
        after: Real,
    },
    Vector {
        before: [Real; 4],
        after: [Real; 4],
    },
    Texture {
        before: &'a AssetReference,
        after: &'a AssetReference,
    },
}

/// A borrowed candidate for the original host's expression-field transaction.
///
/// The node retains its exact ID, position, kind and parameter name. No graph,
/// parameter store, revision, mutation permit or inverse history is created.
/// Before applying, the host must recheck the original view/document/source,
/// selection request and resource catalog, then charge and prepare one domain
/// delta/inverse through its existing retention/permit/CAS/history lifecycle.
#[derive(Debug)]
pub struct MaterialGraphExpressionDetailsEditPlan<'a> {
    source: &'a MaterialGraphAsset,
    node: &'a MaterialGraphNodeAsset,
    values: ExpressionFieldValues<'a>,
}

impl<'a> MaterialGraphExpressionDetailsEditPlan<'a> {
    pub fn source(&self) -> &'a MaterialGraphAsset {
        self.source
    }

    pub fn node(&self) -> &'a MaterialGraphNodeAsset {
        self.node
    }

    pub fn edit(&self) -> MaterialGraphExpressionDetailsEdit<'a> {
        match &self.values {
            ExpressionFieldValues::Scalar { after, .. } => {
                MaterialGraphExpressionDetailsEdit::ScalarDefault(*after)
            }
            ExpressionFieldValues::Vector { after, .. } => {
                MaterialGraphExpressionDetailsEdit::VectorDefault(*after)
            }
            ExpressionFieldValues::Texture { after, .. } => {
                MaterialGraphExpressionDetailsEdit::Texture(after)
            }
        }
    }

    /// Numeric before values copy only one scalar or four source channels.
    pub fn previous_value(&self) -> MaterialGraphExpressionDetailsEdit<'a> {
        match &self.values {
            ExpressionFieldValues::Scalar { before, .. } => {
                MaterialGraphExpressionDetailsEdit::ScalarDefault(*before)
            }
            ExpressionFieldValues::Vector { before, .. } => {
                MaterialGraphExpressionDetailsEdit::VectorDefault(*before)
            }
            ExpressionFieldValues::Texture { before, .. } => {
                MaterialGraphExpressionDetailsEdit::Texture(before)
            }
        }
    }

    /// Preserve authored signed zero as well as the complete reference pair.
    pub fn changes_source(&self) -> bool {
        match &self.values {
            ExpressionFieldValues::Scalar { before, after } => after.to_bits() != before.to_bits(),
            ExpressionFieldValues::Vector { before, after } => after
                .iter()
                .zip(before)
                .any(|(after, before)| after.to_bits() != before.to_bits()),
            ExpressionFieldValues::Texture { before, after } => after != before,
        }
    }
}

/// One field edit across the original expression selection.
///
/// None vector channels retain each node's own authored bits, including mixed
/// or invalid values still awaiting repair. Only supplied channels must be
/// finite. An all-None vector request remains a typed, source-validated no-op.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MaterialGraphExpressionDetailsBatchEdit<'a> {
    ScalarDefault(Real),
    VectorChannels([Option<Real>; 4]),
    Texture(&'a AssetReference),
}

/// A complete candidate, never a sequence of independently authorized edits.
///
/// Entries include no-ops and retain every selected node in source order. A
/// stale identity, incompatible kind or invalid request rejects the whole
/// candidate before resource lookup. No source or history is changed here.
/// The real host must revalidate the original document/revision/selection,
/// account for all changed fields and inverses, then prepare one retained
/// domain delta under one permit/CAS/history operation. Do not execute each
/// entry as a separate command, or recapture expected source after a callback.
#[derive(Debug)]
pub struct MaterialGraphExpressionDetailsBatchPlan<'a> {
    source: &'a MaterialGraphAsset,
    edits: Vec<MaterialGraphExpressionDetailsEditPlan<'a>>,
}

impl<'a> MaterialGraphExpressionDetailsBatchPlan<'a> {
    pub fn source(&self) -> &'a MaterialGraphAsset {
        self.source
    }

    pub fn edits(&self) -> &[MaterialGraphExpressionDetailsEditPlan<'a>] {
        &self.edits
    }

    pub fn changes_source(&self) -> bool {
        self.edits.iter().any(|edit| edit.changes_source())
    }
}

/// Plans a complete multi-object Details edit using the current authored nodes.
///
/// Defaults and parameter overrides remain separate. This validates source
/// identities once and copies only the bounded field values, not a graph or
/// material snapshot. A changed Texture pair is resolved once for the whole
/// selection after all its kinds have passed preflight; an all-no-op request
/// performs no resource lookup. Compile, preview, dirty state and save still
/// belong to the host's actual document transaction, not this candidate.
pub fn plan_material_graph_expression_details_batch_edit<'a>(
    graph: &'a MaterialGraphAsset,
    selected_node_ids: &[&str],
    edit: MaterialGraphExpressionDetailsBatchEdit<'a>,
    mut resolve_kind: impl FnMut(&AssetReference) -> Result<ResourceKind, String>,
) -> Result<MaterialGraphExpressionDetailsBatchPlan<'a>, Vec<MaterialGraphDiagnostic>> {
    use MaterialGraphExpressionDetailsBatchEdit as Edit;
    if selected_node_ids.is_empty() {
        return Err(vec![MaterialGraphDiagnostic::graph(
            "material graph expression Details batch requires a non-empty selection",
        )]);
    }
    let selection = project_material_graph_details_selection(graph, selected_node_ids)?;
    let mut edits = Vec::with_capacity(selection.selected_nodes.len());
    for node in selection.selected_nodes {
        let values = match (&node.kind, edit) {
            (
                MaterialGraphNodeKindAsset::ScalarParameter { default, .. },
                Edit::ScalarDefault(value),
            ) => {
                if !value.is_finite() {
                    return Err(vec![MaterialGraphDiagnostic::node(
                        &node.id,
                        "material graph scalar default must be finite",
                    )]);
                }
                ExpressionFieldValues::Scalar {
                    before: *default,
                    after: value,
                }
            }
            (
                MaterialGraphNodeKindAsset::VectorParameter { default, .. },
                Edit::VectorChannels(channels),
            ) => {
                let mut after = *default;
                for (index, channel) in channels.into_iter().enumerate() {
                    if let Some(value) = channel {
                        if !value.is_finite() {
                            return Err(vec![MaterialGraphDiagnostic::node(
                                &node.id,
                                "material graph edited vector default channels must be finite",
                            )]);
                        }
                        after[index] = value;
                    }
                }
                ExpressionFieldValues::Vector {
                    before: *default,
                    after,
                }
            }
            (MaterialGraphNodeKindAsset::TextureSample { texture }, Edit::Texture(reference)) => {
                ExpressionFieldValues::Texture {
                    before: texture,
                    after: reference,
                }
            }
            _ => {
                return Err(vec![MaterialGraphDiagnostic::node(
                    &node.id,
                    "material graph Details field does not belong to this expression kind",
                )]);
            }
        };
        edits.push(MaterialGraphExpressionDetailsEditPlan {
            source: graph,
            node,
            values,
        });
    }
    if let Edit::Texture(reference) = edit {
        if let Some(changed) = edits.iter().find(|edit| edit.changes_source()) {
            validate_expression_texture_reference(&changed.node.id, reference, &mut resolve_kind)?;
        }
    }
    Ok(MaterialGraphExpressionDetailsBatchPlan {
        source: graph,
        edits,
    })
}

/// Plans an exact node default/reference edit without changing authoring state.
///
/// Reuse the same whole-source identity validation as Details selection. Invalid
/// links, values or overrides remain repairable, but stale/ambiguous IDs cannot
/// address a different expression. New numeric values must be finite; finite
/// HDR/negative values are not clamped. Defaults never clear or rewrite an
/// explicit override, including equal/default/zero and wrong-type overrides.
/// A changed Texture is resolved once by UUID plus locator through the current
/// catalog; an unchanged pair needs no lookup. Host focus/compile/save are not
/// performed by planning the candidate.
pub fn plan_material_graph_expression_details_edit<'a>(
    graph: &'a MaterialGraphAsset,
    node_id: &str,
    edit: MaterialGraphExpressionDetailsEdit<'a>,
    mut resolve_kind: impl FnMut(&AssetReference) -> Result<ResourceKind, String>,
) -> Result<MaterialGraphExpressionDetailsEditPlan<'a>, Vec<MaterialGraphDiagnostic>> {
    use MaterialGraphExpressionDetailsEdit as Edit;
    let selection = project_material_graph_details_selection(graph, &[node_id])?;
    let node = selection.selected_nodes().first().copied().ok_or_else(|| {
        vec![MaterialGraphDiagnostic::graph(
            "material graph Details did not resolve the selected expression",
        )]
    })?;
    let values = match (&node.kind, edit) {
        (
            MaterialGraphNodeKindAsset::ScalarParameter { default, .. },
            Edit::ScalarDefault(value),
        ) => {
            if !value.is_finite() {
                return Err(vec![MaterialGraphDiagnostic::node(
                    &node.id,
                    "material graph scalar default must be finite",
                )]);
            }
            ExpressionFieldValues::Scalar {
                before: *default,
                after: value,
            }
        }
        (
            MaterialGraphNodeKindAsset::VectorParameter { default, .. },
            Edit::VectorDefault(value),
        ) => {
            if !value.iter().all(|channel| channel.is_finite()) {
                return Err(vec![MaterialGraphDiagnostic::node(
                    &node.id,
                    "material graph vector default channels must be finite",
                )]);
            }
            ExpressionFieldValues::Vector {
                before: *default,
                after: value,
            }
        }
        (MaterialGraphNodeKindAsset::TextureSample { texture }, Edit::Texture(value)) => {
            ExpressionFieldValues::Texture {
                before: texture,
                after: value,
            }
        }
        _ => {
            return Err(vec![MaterialGraphDiagnostic::node(
                &node.id,
                "material graph Details field does not belong to this expression kind",
            )]);
        }
    };
    let plan = MaterialGraphExpressionDetailsEditPlan {
        source: graph,
        node,
        values,
    };
    if plan.changes_source() {
        if let Edit::Texture(reference) = edit {
            validate_expression_texture_reference(&node.id, reference, &mut resolve_kind)?;
        }
    }
    Ok(plan)
}

fn validate_expression_texture_reference(
    node_id: &str,
    reference: &AssetReference,
    resolve_kind: &mut impl FnMut(&AssetReference) -> Result<ResourceKind, String>,
) -> Result<(), Vec<MaterialGraphDiagnostic>> {
    match resolve_kind(reference) {
        Ok(ResourceKind::Texture) => Ok(()),
        Ok(kind) => Err(vec![MaterialGraphDiagnostic::node(
            node_id,
            format!("material graph texture references {kind:?}, expected Texture"),
        )]),
        Err(error) => Err(vec![MaterialGraphDiagnostic::node(
            node_id,
            format!("material graph texture reference could not be resolved: {error}"),
        )]),
    }
}
