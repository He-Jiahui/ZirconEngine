use std::collections::BTreeMap;

use super::{MaterialGraphAsset, MaterialGraphLinkAsset, MaterialGraphNodeAsset};

pub(super) struct MaterialGraphIndex<'a> {
    pub(super) nodes: BTreeMap<&'a str, &'a MaterialGraphNodeAsset>,
    pub(super) incoming_links: BTreeMap<&'a str, BTreeMap<&'a str, IncomingLinkIndexEntry<'a>>>,
}

impl<'a> MaterialGraphIndex<'a> {
    pub(super) fn new(graph: &'a MaterialGraphAsset) -> Self {
        let nodes = graph
            .nodes
            .iter()
            .map(|node| (node.id.as_str(), node))
            .collect();
        let mut incoming_links = BTreeMap::new();
        for link in &graph.links {
            incoming_links
                .entry(link.to_node.as_str())
                .or_insert_with(BTreeMap::new)
                .entry(link.to_pin.as_str())
                .and_modify(|entry: &mut IncomingLinkIndexEntry<'a>| entry.count += 1)
                .or_insert(IncomingLinkIndexEntry {
                    first: link,
                    count: 1,
                });
        }
        Self {
            nodes,
            incoming_links,
        }
    }

    pub(super) fn node(&self, node_id: &str) -> Option<&'a MaterialGraphNodeAsset> {
        self.nodes.get(node_id).copied()
    }

    pub(super) fn incoming_link(
        &self,
        node_id: &str,
        pin: &str,
    ) -> Option<&'a MaterialGraphLinkAsset> {
        self.incoming_links
            .get(node_id)?
            .get(pin)
            .map(|entry| entry.first)
    }
}

pub(super) struct IncomingLinkIndexEntry<'a> {
    first: &'a MaterialGraphLinkAsset,
    pub(super) count: usize,
}
