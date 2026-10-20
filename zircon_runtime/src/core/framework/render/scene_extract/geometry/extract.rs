use super::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryDebugState,
    RenderVirtualGeometryHierarchyNode, RenderVirtualGeometryInstance, RenderVirtualGeometryPage,
    RenderVirtualGeometryPageDependency,
};

/// provider 的一帧虚拟几何输入；可见性规划按预算选择簇和页，并通过实例键关联常规网格。
/// 簇、层级、页和实例索引必须在同一帧快照内相互对应。
#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryExtract {
    pub cluster_budget: u32,
    pub page_budget: u32,
    pub clusters: Vec<RenderVirtualGeometryCluster>,
    pub hierarchy_nodes: Vec<RenderVirtualGeometryHierarchyNode>,
    pub hierarchy_child_ids: Vec<u32>,
    pub pages: Vec<RenderVirtualGeometryPage>,
    pub page_dependencies: Vec<RenderVirtualGeometryPageDependency>,
    pub instances: Vec<RenderVirtualGeometryInstance>,
    pub debug: RenderVirtualGeometryDebugState,
}

impl Default for RenderVirtualGeometryExtract {
    fn default() -> Self {
        Self {
            cluster_budget: 0,
            page_budget: 0,
            clusters: Vec::new(),
            hierarchy_nodes: Vec::new(),
            hierarchy_child_ids: Vec::new(),
            pages: Vec::new(),
            page_dependencies: Vec::new(),
            instances: Vec::new(),
            debug: RenderVirtualGeometryDebugState::default(),
        }
    }
}

impl RenderVirtualGeometryExtract {
    /// Resolve original raw rows without mixing explicit instances and another
    /// entity's legacy domain. Residency never changes this domain or its ordinals.
    pub fn checked_cluster_row_indices(
        instances: &[RenderVirtualGeometryInstance],
        clusters: &[RenderVirtualGeometryCluster],
        entity: u64,
        stable_key: u64,
        instance_index: Option<u32>,
    ) -> Result<Vec<usize>, &'static str> {
        if let Some(index) = instance_index {
            let instance = instances
                .get(usize::try_from(index).map_err(|_| "instance index overflow")?)
                .ok_or("instance index exceeds original table")?;
            if instance.entity != entity || instance.stable_instance_key_or_legacy() != stable_key {
                return Err("instance identity differs from its original domain");
            }
            let start =
                usize::try_from(instance.cluster_offset).map_err(|_| "cluster offset overflow")?;
            let end = usize::try_from(
                instance
                    .cluster_offset
                    .checked_add(instance.cluster_count)
                    .ok_or("cluster domain overflow")?,
            )
            .map_err(|_| "cluster end overflow")?;
            let rows = clusters
                .get(start..end)
                .ok_or("cluster domain exceeds original rows")?;
            if rows.iter().any(|row| row.entity != entity) {
                return Err("cluster domain contains another entity");
            }
            Ok((start..end).collect())
        } else {
            if instances.iter().any(|instance| instance.entity == entity)
                || stable_key != super::super::mesh::render_mesh_stable_instance_key(entity, 0)
            {
                return Err("legacy entity has an explicit or different domain");
            }
            Ok(clusters
                .iter()
                .enumerate()
                .filter_map(|(index, row)| (row.entity == entity).then_some(index))
                .collect())
        }
    }

    /// Original rows requiring authored source admission. Reachable hierarchy
    /// rows do not extend the normal selection ordinal domain. Legacy rows
    /// require an entity visible in the actual main view.
    pub fn checked_active_cluster_rows(
        &self,
        visible_legacy_entities: &std::collections::BTreeSet<u64>,
    ) -> Result<Vec<(Option<u32>, u64, usize)>, &'static str> {
        use std::collections::{BTreeMap, BTreeSet};
        let mut active = BTreeSet::new();
        let explicit_entities = self
            .instances
            .iter()
            .map(|row| row.entity)
            .collect::<BTreeSet<_>>();
        for (index, instance) in self.instances.iter().enumerate() {
            let instance_index = u32::try_from(index).map_err(|_| "instance index overflow")?;
            let key = instance.stable_instance_key_or_legacy();
            let normal = self.cluster_row_indices(instance.entity, key, Some(instance_index))?;
            let page_start =
                usize::try_from(instance.page_offset).map_err(|_| "page offset overflow")?;
            let page_end = usize::try_from(
                instance
                    .page_offset
                    .checked_add(instance.page_count)
                    .ok_or("page domain overflow")?,
            )
            .map_err(|_| "page end overflow")?;
            self.pages
                .get(page_start..page_end)
                .ok_or("page domain exceeds original table")?;
            let mut roots = BTreeSet::new();
            for row in normal {
                active.insert((Some(instance_index), key, row));
                if let Some(id) = self.clusters[row].hierarchy_node_id {
                    // A normal row may have no authored node descriptor. Do not
                    // invent a node or enlarge its domain in that case.
                    if self.hierarchy_nodes.iter().any(|node| node.node_id == id) {
                        roots.insert(id);
                    }
                }
            }
            let mut states = BTreeMap::new();
            for root in roots {
                let mut pending = vec![(root, false)];
                while let Some((id, finished)) = pending.pop() {
                    if finished {
                        states.insert(id, 2_u8);
                        continue;
                    }
                    match states.get(&id) {
                        Some(2) => continue,
                        // The CPU traversal owns cycle diagnostics. Both a back
                        // edge and a completed DAG edge add no new source rows.
                        Some(1) => continue,
                        _ => {}
                    }
                    let mut matching = self
                        .hierarchy_nodes
                        .iter()
                        .filter(|node| node.instance_index == instance_index && node.node_id == id);
                    let node = matching
                        .next()
                        .ok_or("reachable hierarchy node has no matching instance")?;
                    if matching.next().is_some() {
                        return Err("reachable hierarchy node identity is ambiguous");
                    }
                    states.insert(id, 1_u8);
                    if states.len() > self.hierarchy_nodes.len() {
                        return Err("authored hierarchy traversal exceeds original nodes");
                    }
                    let start = usize::try_from(node.cluster_start)
                        .map_err(|_| "hierarchy cluster start overflow")?;
                    let end_u32 = node
                        .cluster_start
                        .checked_add(node.cluster_count)
                        .ok_or("hierarchy cluster domain overflow")?;
                    let end =
                        usize::try_from(end_u32).map_err(|_| "hierarchy cluster end overflow")?;
                    let rows = self
                        .clusters
                        .get(start..end)
                        .ok_or("hierarchy cluster domain exceeds original rows")?;
                    if rows.iter().any(|row| row.entity != instance.entity) {
                        return Err("hierarchy cluster domain contains another entity");
                    }
                    for (other_index, other) in self.instances.iter().enumerate() {
                        if other_index == index {
                            continue;
                        }
                        let other_end = other
                            .cluster_offset
                            .checked_add(other.cluster_count)
                            .ok_or("cluster domain overflow")?;
                        if !rows.is_empty()
                            && node.cluster_start < other_end
                            && other.cluster_offset < end_u32
                        {
                            return Err("hierarchy cluster domain overlaps another instance");
                        }
                    }
                    for row in start..end {
                        active.insert((Some(instance_index), key, row));
                    }
                    let child_start = usize::try_from(node.child_base)
                        .map_err(|_| "hierarchy child start overflow")?;
                    let child_end = usize::try_from(
                        node.child_base
                            .checked_add(node.child_count)
                            .ok_or("hierarchy child domain overflow")?,
                    )
                    .map_err(|_| "hierarchy child end overflow")?;
                    let children = self
                        .hierarchy_child_ids
                        .get(child_start..child_end)
                        .ok_or("hierarchy child domain exceeds original IDs")?;
                    pending.push((id, true));
                    for &child in children.iter().rev() {
                        pending.push((child, false));
                    }
                }
            }
        }
        for (index, cluster) in self.clusters.iter().enumerate() {
            if !explicit_entities.contains(&cluster.entity)
                && visible_legacy_entities.contains(&cluster.entity)
            {
                active.insert((
                    None,
                    super::super::mesh::render_mesh_stable_instance_key(cluster.entity, 0),
                    index,
                ));
            }
        }
        Ok(active.into_iter().collect())
    }

    pub fn cluster_row_indices(
        &self,
        entity: u64,
        stable_key: u64,
        instance_index: Option<u32>,
    ) -> Result<Vec<usize>, &'static str> {
        Self::checked_cluster_row_indices(
            &self.instances,
            &self.clusters,
            entity,
            stable_key,
            instance_index,
        )
    }
}

#[cfg(test)]
mod row_domain_tests {
    use super::*;

    #[test]
    fn mixed_explicit_and_legacy_rows_keep_separate_original_domains() {
        let extract = RenderVirtualGeometryExtract {
            clusters: vec![
                RenderVirtualGeometryCluster {
                    entity: 2,
                    cluster_id: 20,
                    ..Default::default()
                },
                RenderVirtualGeometryCluster {
                    entity: 3,
                    cluster_id: 30,
                    ..Default::default()
                },
                RenderVirtualGeometryCluster {
                    entity: 2,
                    cluster_id: 21,
                    ..Default::default()
                },
            ],
            instances: vec![RenderVirtualGeometryInstance {
                entity: 2,
                cluster_count: 1,
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(
            extract.cluster_row_indices(2, 2 << 16, Some(0)).unwrap(),
            vec![0]
        );
        assert_eq!(
            extract.cluster_row_indices(3, 3 << 16, None).unwrap(),
            vec![1]
        );
        assert!(extract.cluster_row_indices(2, 2 << 16, None).is_err());
        assert!(extract.cluster_row_indices(3, 3 << 16, Some(0)).is_err());
        assert!(extract
            .cluster_row_indices(2, (2 << 16) | 1, Some(0))
            .is_err());
        let mut malformed = extract;
        malformed.instances[0].cluster_count = 2;
        assert!(malformed.cluster_row_indices(2, 2 << 16, Some(0)).is_err());
    }
}
