use std::collections::{BTreeMap, BTreeSet};

use zircon_editor::{
    core::editor_authoring_extension::GraphNodePaletteDescriptor,
    ui::graph::{
        GraphCanvasState, GraphEdgeView, GraphNodeBounds, GraphNodeView, GraphPinDirection,
        GraphPinView, GraphPoint, GraphPortRef,
    },
};

use super::{
    connections, graph_contract, node_creation, MaterialGraphAsset, MaterialGraphConnectionPlan,
    MaterialGraphDiagnostic, MaterialGraphIndex, MaterialGraphLinkAsset, MaterialGraphNodeAsset,
    MaterialGraphNodeKindAsset, MaterialGraphValueType,
};

/// An edge in this exact source borrow. The index is not a persistent edge ID.
///
/// MaterialGraphLinkAsset currently has endpoint keys, not a stable edge UUID.
/// The host must keep its document/revision with this original occurrence; a
/// copied view or the same index in a later revision does not identify the edge.
#[derive(Debug, PartialEq)]
pub struct MaterialGraphEdgeProjection<'a> {
    pub source_index: usize,
    pub source: &'a MaterialGraphLinkAsset,
    pub view: GraphEdgeView<String>,
}

/// Material's UE default spline policy for one actual source edge.
///
/// Coordinates are pane-local logical view coordinates after the existing
/// canvas transform. `travel_tangent` is the Hermite derivative at both ends,
/// not a Bezier control offset: controls are source + tangent/3 and target -
/// tangent/3. No arrow, color, DPI, clipping or hit-test authority is supplied.
/// The material drawing policy disables arrowheads; the host owns its brush
/// and must retain the exact source/revision and measured geometry generation.
#[derive(Debug, PartialEq)]
pub struct MaterialGraphEdgeSplineProjection<'a> {
    pub edge: MaterialGraphEdgeProjection<'a>,
    pub source: GraphPoint,
    pub target: GraphPoint,
    pub travel_tangent: GraphPoint,
}

/// Read-only material source adapter for the existing shared graph presentation.
///
/// Build once for the source being presented, not once per node/paint. The
/// indices borrow nodes/links and the type pass is the existing connection
/// analyzer. Incomplete or ill-typed graphs remain visible for repair; their
/// actual type diagnostic is retained separately from displayable geometry.
///
/// The host supplies measured graph-local logical sizes from the same retained
/// geometry generation, then uses its one GraphCanvasState transform for paint
/// and hit testing. No DPI, camera, IDs, model, history or mutation authority is
/// created here. A returned view needs the host's source/revision guard before
/// it may drive selection, Details, a command or a diagnostic jump.
pub struct MaterialGraphPresentation<'a> {
    graph: &'a MaterialGraphAsset,
    index: MaterialGraphIndex<'a>,
    palette: GraphNodePaletteDescriptor,
    types: Result<BTreeMap<&'a str, MaterialGraphValueType>, MaterialGraphDiagnostic>,
}

impl<'a> MaterialGraphPresentation<'a> {
    pub fn new(graph: &'a MaterialGraphAsset) -> Result<Self, Vec<MaterialGraphDiagnostic>> {
        let mut ids = BTreeSet::new();
        for node in &graph.nodes {
            if node.id.trim().is_empty() || !ids.insert(node.id.as_str()) {
                return Err(vec![MaterialGraphDiagnostic::graph(
                    "material graph presentation requires unambiguous non-empty node IDs",
                )]);
            }
        }
        Ok(Self {
            graph,
            index: MaterialGraphIndex::new(graph),
            palette: super::plugin::material_node_palette(),
            types: connections::resolved_node_types(graph),
        })
    }

    pub fn nodes(&self) -> &'a [MaterialGraphNodeAsset] {
        &self.graph.nodes
    }

    pub fn links(&self) -> &'a [MaterialGraphLinkAsset] {
        &self.graph.links
    }

    /// Actual scalar/vector declarations in source order, including disconnected
    /// expressions and invalid declarations that must remain available to repair.
    ///
    /// These are not compiled-visible UE parameter groups. The source has no
    /// group/sort/association metadata; the host must obtain real compiled
    /// visibility before populating that panel. Orphan override keys create no
    /// declarations. Names stay exact schema keys, and duplicate names keep
    /// their distinct original nodes/defaults rather than choosing one silently.
    /// Host source/revision guards and the existing diagnostics remain required.
    pub fn parameter_declarations(
        &self,
    ) -> impl Iterator<Item = super::MaterialGraphParameterView<'a>> + 'a {
        super::parameters::project_parameter_declarations(self.graph)
    }

    /// Texture usage for the exact source, including disconnected expressions.
    ///
    /// The host supplies the current registry display name once per distinct
    /// UUID/locator pair. Labels do not prove resolution or ResourceKind, and
    /// unproven aliases remain separate. The host supplies consistent name
    /// collation (UE FString uses Stricmp); rows use ascending counts first.
    /// Details resolves the last matching expression in source order.
    /// Material Function traversal is unavailable in the current source schema.
    pub fn texture_overview(
        &self,
        display_name: impl FnMut(&super::AssetReference) -> String,
        compare_names: impl FnMut(&str, &str) -> std::cmp::Ordering,
    ) -> Vec<super::MaterialGraphTextureUsage<'a>> {
        super::overview::project_texture_usage(&self.graph.nodes, display_name, compare_names)
    }

    /// Unions every real source node using caller-measured graph-local sizes.
    ///
    /// The callback must read complete retained node sizes from the same source
    /// and geometry generation as this borrow. Missing measurements fail rather
    /// than omit a node or substitute a palette/default size. Empty sources yield
    /// `None`; incomplete/ill-typed sources remain frameable for repair.
    ///
    /// This is O(nodes) with constant extra storage and no graph/view clones.
    /// Pass the result to the existing canvas's `fit_graph_bounds` using its
    /// pane-local logical viewport/padding; it reports zoom-floor containment.
    /// No transform, selection, history, source position or DPI is changed here.
    pub fn graph_bounds(
        &self,
        mut measure_size: impl FnMut(
            &'a MaterialGraphNodeAsset,
        ) -> Result<GraphPoint, MaterialGraphDiagnostic>,
    ) -> Result<Option<GraphNodeBounds>, MaterialGraphDiagnostic> {
        let mut extrema: Option<[f32; 4]> = None;
        for node in self.nodes() {
            let bounds = measured_node_bounds(node, measure_size(node)?)?;
            extrema = Some(match extrema {
                None => [
                    bounds.origin.x,
                    bounds.origin.y,
                    bounds.right(),
                    bounds.bottom(),
                ],
                Some([left, top, right, bottom]) => [
                    left.min(bounds.origin.x),
                    top.min(bounds.origin.y),
                    right.max(bounds.right()),
                    bottom.max(bounds.bottom()),
                ],
            });
        }
        let Some([left, top, right, bottom]) = extrema else {
            return Ok(None);
        };
        let bounds = GraphNodeBounds::new(
            GraphPoint::new(left, top),
            GraphPoint::new(outward_span(left, right), outward_span(top, bottom)),
        );
        if !valid_bounds(bounds) || bounds.right() < right || bounds.bottom() < bottom {
            return Err(MaterialGraphDiagnostic::graph(
                "material graph measured union is not representable as finite logical bounds",
            ));
        }
        Ok(Some(bounds))
    }

    /// Does not replace complete compile/readiness diagnostics.
    pub fn type_diagnostic(&self) -> Option<&MaterialGraphDiagnostic> {
        self.types.as_ref().err()
    }

    /// Uses the actual shared port references and material conversion rules.
    /// A replace plan borrows the exact old source edge; it is not permission
    /// to commit or to reuse this source after its document revision changes.
    pub fn connection_plan(
        &self,
        from: &GraphPortRef<String>,
        to: &GraphPortRef<String>,
    ) -> Result<MaterialGraphConnectionPlan<'a>, String> {
        if from.direction != GraphPinDirection::Output || to.direction != GraphPinDirection::Input {
            return Err("material graph connection requires output to input ports".into());
        }
        super::can_connect_material_graph(
            self.graph,
            &MaterialGraphLinkAsset {
                from_node: from.node_id.clone(),
                from_pin: from.pin_name.clone(),
                to_node: to.node_id.clone(),
                to_pin: to.pin_name.clone(),
            },
        )
    }

    pub fn node(
        &self,
        node_id: &str,
    ) -> Result<&'a MaterialGraphNodeAsset, MaterialGraphDiagnostic> {
        self.index.node(node_id).ok_or_else(|| {
            MaterialGraphDiagnostic::graph(format!(
                "material graph presentation node `{node_id}` is no longer present"
            ))
        })
    }

    pub fn output_type(
        &self,
        node_id: &str,
    ) -> Result<Option<MaterialGraphValueType>, MaterialGraphDiagnostic> {
        let kind = &self.node(node_id)?.kind;
        Ok(match kind {
            MaterialGraphNodeKindAsset::Output => None,
            MaterialGraphNodeKindAsset::ScalarParameter { .. } => {
                Some(MaterialGraphValueType::Scalar)
            }
            MaterialGraphNodeKindAsset::VectorParameter { .. } => {
                Some(MaterialGraphValueType::Vector)
            }
            MaterialGraphNodeKindAsset::TextureSample { .. } => {
                Some(MaterialGraphValueType::Texture)
            }
            MaterialGraphNodeKindAsset::Add | MaterialGraphNodeKindAsset::Multiply => Some(
                self.types
                    .as_ref()
                    .ok()
                    .and_then(|types| types.get(node_id))
                    .copied()
                    .unwrap_or(MaterialGraphValueType::Undetermined),
            ),
        })
    }

    /// Pin value types describe this source's resolved values. Connection
    /// legality still uses can_connect_material_graph (including material
    /// conversions/texture limits), never the shared default string comparison.
    pub fn node_view(
        &self,
        node_id: &str,
        measured_size: GraphPoint,
    ) -> Result<GraphNodeView<String>, MaterialGraphDiagnostic> {
        let node = self.node(node_id)?;
        let bounds = measured_node_bounds(node, measured_size)?;

        let action = self
            .palette
            .nodes()
            .iter()
            .find(|action| action.id() == node_creation::action_key(&node.kind))
            .ok_or_else(|| {
                MaterialGraphDiagnostic::node(node_id, "material graph node kind is not registered")
            })?;
        let display_name = match &node.kind {
            MaterialGraphNodeKindAsset::Output => self.graph.name.clone(),
            MaterialGraphNodeKindAsset::ScalarParameter { name, .. }
            | MaterialGraphNodeKindAsset::VectorParameter { name, .. } => name.clone(),
            _ => action.display_name().to_owned(),
        };
        let math = matches!(
            node.kind,
            MaterialGraphNodeKindAsset::Add | MaterialGraphNodeKindAsset::Multiply
        );
        let inputs = action
            .inputs()
            .iter()
            .map(|pin| {
                let value_type = if math {
                    let input_type = self
                        .index
                        .incoming_links
                        .get(node_id)
                        .and_then(|inputs| inputs.get(pin.name()))
                        // Multiple occurrences are repairable source data, not
                        // a reason to choose the first wire's concrete type.
                        .filter(|entry| entry.count == 1)
                        .and_then(|_| self.index.incoming_link(node_id, pin.name()))
                        .and_then(|link| self.output_type(&link.from_node).ok().flatten())
                        .unwrap_or(MaterialGraphValueType::Undetermined);
                    type_name(input_type)
                } else {
                    pin.value_type()
                };
                GraphPinView::input(pin.name(), value_type, pin.is_required())
            })
            .collect();
        let output_type = self.output_type(node_id)?;
        let outputs = action
            .outputs()
            .iter()
            .map(|pin| {
                GraphPinView::output(pin.name(), output_type.map_or(pin.value_type(), type_name))
            })
            .collect();
        Ok(GraphNodeView {
            id: node.id.clone(),
            display_name,
            bounds,
            inputs,
            outputs,
            attachments: Vec::new(),
        })
    }

    /// Projects one original occurrence, including an ill-typed wire that can
    /// still be selected for repair. Invalid endpoints are diagnosed, not drawn
    /// at a fabricated node/pin or redirected to a different occurrence.
    pub fn edge_view(
        &self,
        source_index: usize,
    ) -> Result<MaterialGraphEdgeProjection<'a>, MaterialGraphDiagnostic> {
        let edge = self.graph.links.get(source_index).ok_or_else(|| {
            MaterialGraphDiagnostic::graph("material graph edge occurrence is no longer present")
        })?;
        let from = self.node(&edge.from_node)?;
        let to = self.node(&edge.to_node)?;
        if graph_contract::output_pin(&from.kind) != Some(edge.from_pin.as_str()) {
            return Err(MaterialGraphDiagnostic::node(
                &from.id,
                format!(
                    "material graph has no output pin `{}.{}`",
                    from.id, edge.from_pin
                ),
            ));
        }
        if !graph_contract::has_input_pin(&to.kind, &edge.to_pin) {
            return Err(MaterialGraphDiagnostic::node(
                &to.id,
                format!(
                    "material graph has no input pin `{}.{}`",
                    to.id, edge.to_pin
                ),
            ));
        }
        Ok(MaterialGraphEdgeProjection {
            source_index,
            source: edge,
            view: GraphEdgeView::new(
                GraphPortRef::output(edge.from_node.clone(), edge.from_pin.clone()),
                GraphPortRef::input(edge.to_node.clone(), edge.to_pin.clone()),
            ),
        })
    }

    /// Uses caller-measured graph-local pin anchors, never node centers or
    /// positions invented from a palette row. The caller must check that the
    /// anchors, viewport and canvas belong to the same retained generation.
    ///
    /// UE GraphEditorSettings computes its tension from the drawn endpoints,
    /// so the one canvas transform precedes the forward/backward clamps. The
    /// result must not be transformed again as graph-local data. Only the
    /// eventual view-to-native conversion may apply DPI. This read-only method
    /// changes neither the source nor the existing canvas/selection/history.
    /// A coincident endpoint pair retains UE's zero tangent without fabricating
    /// a 48-unit handle or authorizing an arrow.
    pub fn edge_spline_view(
        &self,
        source_index: usize,
        canvas: &GraphCanvasState<String>,
        source_graph_anchor: GraphPoint,
        target_graph_anchor: GraphPoint,
    ) -> Result<MaterialGraphEdgeSplineProjection<'a>, MaterialGraphDiagnostic> {
        let edge = self.edge_view(source_index)?;
        let source = canvas.graph_to_screen(source_graph_anchor);
        let target = canvas.graph_to_screen(target_graph_anchor);
        let dx = target.x - source.x;
        let dy = target.y - source.y;
        if [
            source_graph_anchor.x,
            source_graph_anchor.y,
            target_graph_anchor.x,
            target_graph_anchor.y,
            source.x,
            source.y,
            target.x,
            target.y,
            dx,
            dy,
        ]
        .iter()
        .any(|value| !value.is_finite())
        {
            return Err(MaterialGraphDiagnostic::graph(
                "material graph spline requires finite, representable logical geometry",
            ));
        }
        // UE GraphEditorSettings.cpp defaults. This is a material policy, not
        // a change to the shared route_connection default or another camera.
        let tension = if dx >= 0.0 {
            dx.abs().min(1000.0) + dy.abs().min(1000.0)
        } else {
            2.0 * dx.abs().min(200.0) + 1.5 * dy.abs().min(200.0)
        };
        Ok(MaterialGraphEdgeSplineProjection {
            edge,
            source,
            target,
            travel_tangent: GraphPoint::new(tension, 0.0),
        })
    }
}

fn measured_node_bounds(
    node: &MaterialGraphNodeAsset,
    measured_size: GraphPoint,
) -> Result<GraphNodeBounds, MaterialGraphDiagnostic> {
    let bounds = GraphNodeBounds::new(
        GraphPoint::new(node.position[0], node.position[1]),
        measured_size,
    );
    if !valid_bounds(bounds) {
        return Err(MaterialGraphDiagnostic::node(
            &node.id,
            "material graph node requires finite, positive measured logical bounds",
        ));
    }
    Ok(bounds)
}

fn valid_bounds(bounds: GraphNodeBounds) -> bool {
    [
        bounds.origin.x,
        bounds.origin.y,
        bounds.size.x,
        bounds.size.y,
        bounds.right(),
        bounds.bottom(),
    ]
    .iter()
    .all(|value| value.is_finite())
        && bounds.size.x > 0.0
        && bounds.size.y > 0.0
        && bounds.right() > bounds.origin.x
        && bounds.bottom() > bounds.origin.y
}

fn outward_span(start: f32, end: f32) -> f32 {
    let span = f64::from(end) - f64::from(start);
    let rounded = span as f32;
    // An inward-rounded width would silently omit part of a measured node.
    if rounded.is_finite() && f64::from(rounded) < span {
        f32::from_bits(rounded.to_bits() + 1)
    } else {
        rounded
    }
}

fn type_name(value_type: MaterialGraphValueType) -> &'static str {
    match value_type {
        MaterialGraphValueType::Scalar => "float",
        MaterialGraphValueType::Vector => "vec4",
        MaterialGraphValueType::Texture => "texture",
        MaterialGraphValueType::Undetermined => "undetermined",
    }
}
