use super::{
    project_material_graph_details_selection, MaterialGraphAsset, MaterialGraphDiagnostic,
    MaterialGraphNodeAsset, MaterialGraphNodeKindAsset, MaterialGraphParameterAsset,
};

/// One declaration from the exact presented authoring graph.
///
/// Node/name/override borrow the original source; only the fixed-size numeric
/// default is copied into the existing asset value type. This is neither a
/// second parameter store nor a mutation/selection permit. Rebuild after source
/// changes and retain the host's document/revision before using the node/key.
#[derive(Debug)]
pub struct MaterialGraphParameterView<'a> {
    node: &'a MaterialGraphNodeAsset,
    name: &'a str,
    default: MaterialGraphParameterAsset,
    override_value: Option<&'a MaterialGraphParameterAsset>,
}

impl<'a> MaterialGraphParameterView<'a> {
    pub fn node(&self) -> &'a MaterialGraphNodeAsset {
        self.node
    }

    /// Exact persistent key, including case and whitespace; not a display label.
    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn default_value(&self) -> &MaterialGraphParameterAsset {
        &self.default
    }

    /// Presence is distinct from equality to the default, including zero values.
    pub fn override_value(&self) -> Option<&'a MaterialGraphParameterAsset> {
        self.override_value
    }

    /// Raw typed source value; this is not compile/readiness validation.
    ///
    /// A wrong-type override returns None instead of displaying a successful
    /// default fallback. The original override remains available for repair.
    /// Non-finite defaults/overrides remain intact; the existing graph validator
    /// must report them even when another, finite override supplies this value.
    pub fn effective_value(&self) -> Option<&MaterialGraphParameterAsset> {
        use MaterialGraphParameterAsset as Value;
        match (&self.default, self.override_value) {
            (_, None) => Some(&self.default),
            (Value::Scalar(_), Some(value @ Value::Scalar(_)))
            | (Value::Vector(_), Some(value @ Value::Vector(_))) => Some(value),
            _ => None,
        }
    }
}

pub(super) fn project_parameter_declarations(
    graph: &MaterialGraphAsset,
) -> impl Iterator<Item = MaterialGraphParameterView<'_>> {
    graph.nodes.iter().filter_map(move |node| {
        let (name, default) = match &node.kind {
            MaterialGraphNodeKindAsset::ScalarParameter { name, default } => {
                (name, MaterialGraphParameterAsset::Scalar(*default))
            }
            MaterialGraphNodeKindAsset::VectorParameter { name, default } => {
                (name, MaterialGraphParameterAsset::Vector(*default))
            }
            _ => return None,
        };
        Some(MaterialGraphParameterView {
            node,
            name,
            default,
            override_value: graph.parameters.get(name),
        })
    })
}

/// One edit of an exact key in the authored graph's parameter override map.
///
/// Set preserves explicit presence even when equal to a node's default. Reset
/// removes that key; it never copies a default into the map or rewrites nodes.
/// These are graph parameters, not the separate MaterialAsset instance ABI.
#[derive(Clone, Debug, PartialEq)]
pub enum MaterialGraphParameterEdit {
    Set(MaterialGraphParameterAsset),
    Reset,
}

/// A borrowed, source-bound candidate for one parameter map entry.
///
/// The original host must recheck document/subdocument, revision, view and
/// request; charge/copy this key and both presence-sensitive values; then use
/// one domain delta/inverse and its existing prepare/permit/CAS/history. This
/// candidate creates no parameter store, revision, mutation or undo authority.
#[derive(Debug)]
pub struct MaterialGraphParameterEditPlan<'a> {
    source: &'a MaterialGraphAsset,
    node: &'a MaterialGraphNodeAsset,
    key: &'a str,
    previous: Option<&'a MaterialGraphParameterAsset>,
    edit: MaterialGraphParameterEdit,
}

impl<'a> MaterialGraphParameterEditPlan<'a> {
    pub fn source(&self) -> &'a MaterialGraphAsset {
        self.source
    }

    pub fn node(&self) -> &'a MaterialGraphNodeAsset {
        self.node
    }

    /// Exact persistent key; never trimmed, case folded or inferred from a label.
    pub fn key(&self) -> &'a str {
        self.key
    }

    pub fn edit(&self) -> &MaterialGraphParameterEdit {
        &self.edit
    }

    /// None is absent; a wrong-type/non-finite original remains intact for undo.
    pub fn previous_value(&self) -> Option<&'a MaterialGraphParameterAsset> {
        self.previous
    }

    pub fn proposed_value(&self) -> Option<&MaterialGraphParameterAsset> {
        match &self.edit {
            MaterialGraphParameterEdit::Set(value) => Some(value),
            MaterialGraphParameterEdit::Reset => None,
        }
    }

    /// Each declaration affected by this one map entry, in original source order.
    pub fn affected_nodes(&self) -> impl Iterator<Item = &'a MaterialGraphNodeAsset> + 'a {
        let source = self.source;
        let key = self.key;
        source
            .nodes
            .iter()
            .filter(move |node| parameter_name(node) == Some(key))
    }

    /// Presence, type and authored numeric bits all participate in the no-op test.
    pub fn changes_source(&self) -> bool {
        match (self.previous, self.proposed_value()) {
            (None, None) => false,
            (Some(before), Some(after)) => !same_parameter_bits(before, after),
            _ => true,
        }
    }
}

/// Plans Set/Reset through an exact current parameter declaration's node ID.
///
/// Same-key declarations share one override. Set must match every declaration,
/// and only finite new numeric values are accepted; finite HDR/negative values
/// are not clamped. Reset can repair a wrong-type/non-finite override or an empty
/// authored key. Invalid defaults/links remain unchanged and still diagnostic.
/// Unknown/orphan keys cannot fabricate a parameter row or redirect this edit.
/// No graph clone, asset resolution, compilation, save or renderer update occurs.
pub fn plan_material_graph_parameter_edit<'a>(
    graph: &'a MaterialGraphAsset,
    node_id: &str,
    edit: MaterialGraphParameterEdit,
) -> Result<MaterialGraphParameterEditPlan<'a>, Vec<MaterialGraphDiagnostic>> {
    let selection = project_material_graph_details_selection(graph, &[node_id])?;
    let node = selection.selected_nodes().first().copied().ok_or_else(|| {
        vec![MaterialGraphDiagnostic::graph(
            "material graph parameter edit did not resolve its declaration",
        )]
    })?;
    let key = parameter_name(node).ok_or_else(|| {
        vec![MaterialGraphDiagnostic::node(
            &node.id,
            "material graph parameter edit requires a parameter declaration",
        )]
    })?;
    if let MaterialGraphParameterEdit::Set(value) = &edit {
        if key.trim().is_empty() {
            return Err(vec![MaterialGraphDiagnostic::node(
                &node.id,
                "material graph cannot set an override for an empty parameter name",
            )]);
        }
        for declaration in graph
            .nodes
            .iter()
            .filter(|candidate| parameter_name(candidate) == Some(key))
        {
            if !matches!(
                (&declaration.kind, value),
                (
                    MaterialGraphNodeKindAsset::ScalarParameter { .. },
                    MaterialGraphParameterAsset::Scalar(_)
                ) | (
                    MaterialGraphNodeKindAsset::VectorParameter { .. },
                    MaterialGraphParameterAsset::Vector(_)
                )
            ) {
                return Err(vec![MaterialGraphDiagnostic::node(
                    &declaration.id,
                    "material graph parameter override must match every declaration of its exact key",
                )]);
            }
        }
        let finite = match value {
            MaterialGraphParameterAsset::Scalar(value) => value.is_finite(),
            MaterialGraphParameterAsset::Vector(value) => value.iter().all(|v| v.is_finite()),
        };
        if !finite {
            return Err(vec![MaterialGraphDiagnostic::parameter(
                key,
                "material graph new parameter override must be finite",
            )]);
        }
    }
    Ok(MaterialGraphParameterEditPlan {
        source: graph,
        node,
        key,
        previous: graph.parameters.get(key),
        edit,
    })
}

fn parameter_name(node: &MaterialGraphNodeAsset) -> Option<&str> {
    match &node.kind {
        MaterialGraphNodeKindAsset::ScalarParameter { name, .. }
        | MaterialGraphNodeKindAsset::VectorParameter { name, .. } => Some(name),
        _ => None,
    }
}

fn same_parameter_bits(
    before: &MaterialGraphParameterAsset,
    after: &MaterialGraphParameterAsset,
) -> bool {
    match (before, after) {
        (
            MaterialGraphParameterAsset::Scalar(before),
            MaterialGraphParameterAsset::Scalar(after),
        ) => before.to_bits() == after.to_bits(),
        (
            MaterialGraphParameterAsset::Vector(before),
            MaterialGraphParameterAsset::Vector(after),
        ) => before
            .iter()
            .zip(after)
            .all(|(a, b)| a.to_bits() == b.to_bits()),
        _ => false,
    }
}
