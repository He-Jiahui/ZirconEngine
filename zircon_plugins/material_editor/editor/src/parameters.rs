use super::{
    MaterialGraphAsset, MaterialGraphNodeAsset, MaterialGraphNodeKindAsset,
    MaterialGraphParameterAsset,
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
