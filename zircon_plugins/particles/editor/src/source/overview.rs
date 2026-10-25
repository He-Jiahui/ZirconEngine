use std::collections::{BTreeMap, BTreeSet};

use zircon_editor::ui::graph::{GraphEdgeView, GraphNodeBounds, GraphNodeView};

use super::{ParticleSourceError, ParticleSystemSource};

/// Stable only within the original document. Captions and source order are not identities.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParticleOverviewNodeId {
    System,
    Emitter(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleOverviewConnectionEligibility {
    UnavailableNoSourcePortsOrEdges,
}

impl ParticleOverviewConnectionEligibility {
    /// All connection entry points must consume the same domain reason.
    pub const fn reason(self) -> &'static str {
        "此资源未定义端口或连线"
    }
}

/// A read-only projection, not a mutable GraphModel or an authoring/preview success receipt.
/// The host still binds this to the actual document/revision and retained layout generation.
#[derive(Debug, PartialEq)]
pub struct ParticleOverviewProjection {
    nodes: Vec<GraphNodeView<ParticleOverviewNodeId>>,
    edges: Vec<GraphEdgeView<ParticleOverviewNodeId>>,
}

impl ParticleOverviewProjection {
    pub fn nodes(&self) -> &[GraphNodeView<ParticleOverviewNodeId>] {
        &self.nodes
    }

    pub fn edges(&self) -> &[GraphEdgeView<ParticleOverviewNodeId>] {
        &self.edges
    }

    pub fn connection_eligibility(&self) -> ParticleOverviewConnectionEligibility {
        ParticleOverviewConnectionEligibility::UnavailableNoSourcePortsOrEdges
    }
}

impl ParticleSystemSource {
    /// Use actual measured graph-local bounds. Missing/stale layout is an error, not an empty
    /// ready graph; this projection neither guesses positions nor persists a second layout.
    pub fn project_overview(
        &self,
        layout: &BTreeMap<ParticleOverviewNodeId, GraphNodeBounds>,
    ) -> Result<ParticleOverviewProjection, ParticleSourceError> {
        let mut nodes = vec![node(
            ParticleOverviewNodeId::System,
            &self.system.id,
            layout,
        )?];
        let mut keys = BTreeSet::new();
        for emitter in &self.emitters {
            let key = emitter
                .key
                .as_deref()
                .filter(|key| !key.trim().is_empty())
                .ok_or_else(|| {
                    ParticleSourceError::invalid(
                        "overview.emitters.key",
                        "source emitter needs its assigned stable key before projection",
                    )
                })?;
            if !keys.insert(key) {
                return Err(ParticleSourceError::invalid(
                    "overview.emitters.key",
                    "duplicate source emitter key",
                ));
            }
            nodes.push(node(
                ParticleOverviewNodeId::Emitter(key.to_owned()),
                &emitter.id,
                layout,
            )?);
        }
        Ok(ParticleOverviewProjection {
            nodes,
            edges: Vec::new(),
        })
    }
}

fn node(
    id: ParticleOverviewNodeId,
    caption: &str,
    layout: &BTreeMap<ParticleOverviewNodeId, GraphNodeBounds>,
) -> Result<GraphNodeView<ParticleOverviewNodeId>, ParticleSourceError> {
    let bounds = *layout.get(&id).ok_or_else(|| {
        ParticleSourceError::invalid(
            "overview.layout",
            format!("missing measured bounds for {id:?}"),
        )
    })?;
    if !bounds.origin.x.is_finite()
        || !bounds.origin.y.is_finite()
        || !bounds.size.x.is_finite()
        || !bounds.size.y.is_finite()
        || bounds.size.x <= 0.0
        || bounds.size.y <= 0.0
        || !bounds.right().is_finite()
        || !bounds.bottom().is_finite()
        || bounds.right() <= bounds.origin.x
        || bounds.bottom() <= bounds.origin.y
    {
        return Err(ParticleSourceError::invalid(
            "overview.layout",
            "node bounds must be finite and have positive size",
        ));
    }
    Ok(GraphNodeView {
        id,
        display_name: caption.to_owned(),
        bounds,
        inputs: Vec::new(),
        outputs: Vec::new(),
        attachments: Vec::new(),
    })
}
