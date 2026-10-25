use std::collections::HashMap;

use super::{AssetReference, MaterialGraphNodeAsset, MaterialGraphNodeKindAsset};

/// One texture usage row borrowed from the presented source graph.
///
/// Full UUID/locator pairs remain distinct without registry alias evidence.
/// Display names are supplied by the host and never identify a resource. The
/// host must bind row activation to the same document/revision as its source
/// borrow before resolving Details, selection or focus. This is not a permit.
#[derive(Debug)]
pub struct MaterialGraphTextureUsage<'a> {
    texture: &'a AssetReference,
    display_name: String,
    nodes: Vec<&'a MaterialGraphNodeAsset>,
}

impl<'a> MaterialGraphTextureUsage<'a> {
    pub fn texture(&self) -> &'a AssetReference {
        self.texture
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Real source occurrences, including disconnected texture expressions.
    pub fn nodes(&self) -> &[&'a MaterialGraphNodeAsset] {
        &self.nodes
    }

    pub fn reference_count(&self) -> usize {
        self.nodes.len()
    }

    /// UE's row click resolves the last matching expression in source order.
    pub fn details_node(&self) -> &'a MaterialGraphNodeAsset {
        self.nodes
            .last()
            .copied()
            .expect("texture usage has at least one source expression")
    }
}

pub(super) fn project_texture_usage<'a>(
    nodes: &'a [MaterialGraphNodeAsset],
    mut display_name: impl FnMut(&AssetReference) -> String,
    mut compare_names: impl FnMut(&str, &str) -> std::cmp::Ordering,
) -> Vec<MaterialGraphTextureUsage<'a>> {
    let mut rows: Vec<MaterialGraphTextureUsage<'a>> = Vec::new();
    let mut indices = HashMap::<&AssetReference, usize>::new();
    for node in nodes {
        let MaterialGraphNodeKindAsset::TextureSample { texture } = &node.kind else {
            continue;
        };
        if let Some(&index) = indices.get(texture) {
            rows[index].nodes.push(node);
        } else {
            indices.insert(texture, rows.len());
            rows.push(MaterialGraphTextureUsage {
                texture,
                display_name: display_name(texture),
                nodes: vec![node],
            });
        }
    }
    // UE's comparator uses ascending counts despite its descending comment.
    // FString uses Stricmp. The host supplies its name collation; Rust's default
    // string order is not a replacement. Stable ties preserve first occurrence.
    rows.sort_by(|left, right| {
        left.reference_count()
            .cmp(&right.reference_count())
            .then_with(|| compare_names(&left.display_name, &right.display_name))
    });
    rows
}
