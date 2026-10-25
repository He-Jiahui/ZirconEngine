use std::collections::{BTreeMap, BTreeSet};

use super::{
    AssetReference, MaterialGraphAsset, MaterialGraphDiagnostic, MaterialGraphIndex,
    MaterialGraphNodeAsset, MaterialGraphNodeKindAsset as Node,
    MaterialGraphParameterAsset as Parameter,
};

#[derive(Clone, Debug, PartialEq)]
pub(super) enum MaterialColorInput {
    Constant([f32; 4]),
    Texture(AssetReference),
}

/// Transient values for one compile, never retained authoring or undo state.
pub(super) struct MaterialGraphEvaluator<'graph, 'index> {
    graph: &'graph MaterialGraphAsset,
    index: &'index MaterialGraphIndex<'graph>,
    colors: BTreeMap<&'graph str, MaterialColorInput>,
    scalars: BTreeMap<&'graph str, f32>,
}

impl<'graph, 'index> MaterialGraphEvaluator<'graph, 'index> {
    pub(super) fn new(
        graph: &'graph MaterialGraphAsset,
        index: &'index MaterialGraphIndex<'graph>,
    ) -> Self {
        Self {
            graph,
            index,
            colors: BTreeMap::new(),
            scalars: BTreeMap::new(),
        }
    }

    pub(super) fn color(
        &mut self,
        node_id: &'graph str,
    ) -> Result<MaterialColorInput, MaterialGraphDiagnostic> {
        let parameters = &self.graph.parameters;
        evaluate(
            self.index,
            node_id,
            &mut self.colors,
            |node| match &node.kind {
                Node::TextureSample { texture } => Ok(MaterialColorInput::Texture(texture.clone())),
                Node::ScalarParameter { name, default } => {
                    let value = match parameters.get(name) {
                        Some(Parameter::Scalar(value)) => *value,
                        _ => *default,
                    };
                    Ok(MaterialColorInput::Constant([value, value, value, 1.0]))
                }
                Node::VectorParameter { name, default } => {
                    let value = match parameters.get(name) {
                        Some(Parameter::Vector(value)) => *value,
                        _ => *default,
                    };
                    Ok(MaterialColorInput::Constant(value))
                }
                _ => Err(MaterialGraphDiagnostic::node(
                    &node.id,
                    format!(
                        "material graph output node `{}` cannot feed another node",
                        node.id
                    ),
                )),
            },
            combine_colors,
        )
    }

    pub(super) fn scalar(&mut self, node_id: &'graph str) -> Result<f32, MaterialGraphDiagnostic> {
        let parameters = &self.graph.parameters;
        evaluate(
            self.index,
            node_id,
            &mut self.scalars,
            |node| match &node.kind {
                Node::ScalarParameter { name, default } => checked_scalar(
                    &node.id,
                    match parameters.get(name) {
                        Some(Parameter::Scalar(value)) => *value,
                        _ => *default,
                    },
                ),
                _ => Err(MaterialGraphDiagnostic::node(
                    &node.id,
                    format!("node `{}` does not produce a scalar", node.id),
                )),
            },
            |node, a, b| {
                checked_scalar(
                    &node.id,
                    if matches!(&node.kind, Node::Add) {
                        a + b
                    } else {
                        a * b
                    },
                )
            },
        )
    }
}

fn evaluate<'a, Value: Clone>(
    index: &MaterialGraphIndex<'a>,
    root: &'a str,
    values: &mut BTreeMap<&'a str, Value>,
    leaf: impl Fn(&MaterialGraphNodeAsset) -> Result<Value, MaterialGraphDiagnostic>,
    combine: impl Fn(&MaterialGraphNodeAsset, &Value, &Value) -> Result<Value, MaterialGraphDiagnostic>,
) -> Result<Value, MaterialGraphDiagnostic> {
    // Explicit postorder avoids call-stack growth and evaluates shared inputs once.
    // Only demanded ancestors are visited: disconnected arithmetic may overflow
    // without affecting a material until an output actually depends on it.
    let mut pending = vec![(root, false)];
    let mut active = BTreeSet::new();
    while let Some((id, expanded)) = pending.pop() {
        if values.contains_key(id) {
            continue;
        }
        let node = index.node(id).ok_or_else(|| {
            MaterialGraphDiagnostic::graph(format!("material graph references missing node `{id}`"))
        })?;
        if !matches!(&node.kind, Node::Add | Node::Multiply) {
            values.insert(id, leaf(node)?);
            continue;
        }
        let input = |pin| {
            index
                .incoming_link(id, pin)
                .map(|link| link.from_node.as_str())
                .ok_or_else(|| {
                    MaterialGraphDiagnostic::input(
                        id,
                        pin,
                        format!("material graph node `{id}` missing input `{pin}`"),
                    )
                })
        };
        let a = input("a")?;
        let b = input("b")?;
        if expanded {
            let value = combine(
                node,
                values.get(a).expect("postorder evaluated input a"),
                values.get(b).expect("postorder evaluated input b"),
            )?;
            values.insert(id, value);
            active.remove(id);
        } else {
            if !active.insert(id) {
                return Err(MaterialGraphDiagnostic::node(
                    id,
                    format!("material graph contains a cycle at node `{id}`"),
                ));
            }
            pending.push((id, true));
            pending.push((b, false));
            pending.push((a, false));
        }
    }
    Ok(values.get(root).expect("evaluated requested root").clone())
}

fn combine_colors(
    node: &MaterialGraphNodeAsset,
    a: &MaterialColorInput,
    b: &MaterialColorInput,
) -> Result<MaterialColorInput, MaterialGraphDiagnostic> {
    let op = if matches!(&node.kind, Node::Add) {
        "add"
    } else {
        "multiply"
    };
    match (a, b) {
        (MaterialColorInput::Constant(a), MaterialColorInput::Constant(b)) => {
            let value = std::array::from_fn(|channel| {
                if matches!(&node.kind, Node::Add) {
                    a[channel] + b[channel]
                } else {
                    a[channel] * b[channel]
                }
            });
            if value.iter().any(|channel| !channel.is_finite()) {
                return Err(MaterialGraphDiagnostic::node(
                    &node.id,
                    format!("material graph `{op}` produced a non-finite value"),
                ));
            }
            Ok(MaterialColorInput::Constant(value))
        }
        _ => Err(MaterialGraphDiagnostic::node(
            &node.id,
            format!("material graph `{op}` node cannot combine texture-backed inputs in v1"),
        )),
    }
}

fn checked_scalar(node_id: &str, value: f32) -> Result<f32, MaterialGraphDiagnostic> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(MaterialGraphDiagnostic::node(
            node_id,
            format!("node `{node_id}` produced a non-finite scalar"),
        ))
    }
}
