use crate::core::framework::render::{
    RenderVirtualGeometryBvhVisualizationInstance, RenderVirtualGeometryCpuReferenceInstance,
    RenderVirtualGeometryExtract, RenderVirtualGeometryPagePayload,
};

#[derive(Clone, Debug, Default, PartialEq)]
/// 自动提取的一次性结果；几何、CPU 参考实例、BVH 可视化与驻留页必须来自同一批网格资产。
/// 框架在构建帧上下文时消费全部字段，避免只更新提取内容而丢失随附资源。
pub struct VirtualGeometryRuntimeExtractOutput {
    indexed_sources: Vec<crate::graphics::VirtualGeometryIndexedSourceAdmission>,
    extract: RenderVirtualGeometryExtract,
    cpu_reference_instances: Vec<RenderVirtualGeometryCpuReferenceInstance>,
    bvh_visualization_instances: Vec<RenderVirtualGeometryBvhVisualizationInstance>,
    resident_page_payloads: Vec<RenderVirtualGeometryPagePayload>,
}

impl VirtualGeometryRuntimeExtractOutput {
    pub fn new(
        extract: RenderVirtualGeometryExtract,
        cpu_reference_instances: Vec<RenderVirtualGeometryCpuReferenceInstance>,
        bvh_visualization_instances: Vec<RenderVirtualGeometryBvhVisualizationInstance>,
        resident_page_payloads: Vec<RenderVirtualGeometryPagePayload>,
    ) -> Self {
        Self {
            indexed_sources: Vec::new(),
            extract,
            cpu_reference_instances,
            bvh_visualization_instances,
            resident_page_payloads,
        }
    }

    pub fn extract(&self) -> &RenderVirtualGeometryExtract {
        &self.extract
    }

    pub fn cpu_reference_instances(&self) -> &[RenderVirtualGeometryCpuReferenceInstance] {
        &self.cpu_reference_instances
    }

    pub fn bvh_visualization_instances(&self) -> &[RenderVirtualGeometryBvhVisualizationInstance] {
        &self.bvh_visualization_instances
    }

    pub fn resident_page_payloads(&self) -> &[RenderVirtualGeometryPagePayload] {
        &self.resident_page_payloads
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        RenderVirtualGeometryExtract,
        Vec<RenderVirtualGeometryCpuReferenceInstance>,
        Vec<RenderVirtualGeometryBvhVisualizationInstance>,
        Vec<RenderVirtualGeometryPagePayload>,
    ) {
        (
            self.extract,
            self.cpu_reference_instances,
            self.bvh_visualization_instances,
            self.resident_page_payloads,
        )
    }
}

#[cfg(test)]
#[path = "../tests/extract_output.rs"]
mod tests;

impl VirtualGeometryRuntimeExtractOutput {
    pub fn with_indexed_sources(
        mut self,
        sources: Vec<crate::graphics::VirtualGeometryIndexedSourceAdmission>,
    ) -> Self {
        self.indexed_sources = sources;
        self
    }
    pub fn indexed_sources(&self) -> &[crate::graphics::VirtualGeometryIndexedSourceAdmission] {
        &self.indexed_sources
    }
}

impl VirtualGeometryRuntimeExtractOutput {
    /// Preserve authored IDs and raw row order, but accept geometry only from a
    /// uniquely resolved cooked primitive. Ambiguous authored topology fails.
    pub(crate) fn bind_authored_indexed_topology(
        mut self,
        authored: &RenderVirtualGeometryExtract,
        visible_legacy_entities: &std::collections::BTreeSet<u64>,
    ) -> Result<Self, String> {
        use super::source_admission::AuthoredIndexedRowBinding;
        use crate::core::framework::render::render_mesh_stable_instance_key;
        use std::collections::{BTreeMap, BTreeSet};
        let active_rows = authored
            .checked_active_cluster_rows(visible_legacy_entities)
            .map_err(str::to_owned)?;
        let mut payloads = std::mem::take(&mut self.resident_page_payloads)
            .into_iter()
            .map(Some)
            .collect::<Vec<_>>();
        let mut admitted_payloads = Vec::new();
        let mut remapped = Vec::new();
        let mut covered = BTreeSet::new();
        let mut domain_sources = BTreeSet::new();
        let mut domain_primitives = Vec::new();
        for source in &self.indexed_sources {
            let canonical = self
                .extract
                .instances
                .iter()
                .filter(|instance| {
                    instance.stable_instance_key_or_legacy() == source.stable_instance_key()
                })
                .collect::<Vec<_>>();
            if canonical.len() != 1 {
                return Err("canonical admission instance is not unique".into());
            }
            let instance = canonical[0];
            let mut domains = Vec::new();
            for (index, row) in authored.instances.iter().enumerate().filter(|(_, row)| {
                row.entity == instance.entity
                    && row.stable_instance_key_or_legacy() == source.stable_instance_key()
            }) {
                domains.push((
                    Some(u32::try_from(index).map_err(|_| "authored instance index overflow")?),
                    row.stable_instance_key_or_legacy(),
                ));
            }
            if !authored
                .instances
                .iter()
                .any(|row| row.entity == instance.entity)
                && visible_legacy_entities.contains(&instance.entity)
            {
                domains.push((None, render_mesh_stable_instance_key(instance.entity, 0)));
            }
            if domains.is_empty() {
                remapped.push(
                    source
                        .clone()
                        .with_authored_rows(source.stable_instance_key(), Vec::new()),
                );
            }
            for (instance_index, execution_key) in domains {
                if !domain_sources.insert((
                    source.stable_instance_key(),
                    instance_index,
                    execution_key,
                )) {
                    return Err("canonical primitive is repeated in one authored domain".into());
                }
                if domain_primitives.iter().any(
                    |(previous_index, previous_key, primitive): &(
                        Option<u32>,
                        u64,
                        crate::graphics::VirtualGeometryPrimitiveAdmission,
                    )| {
                        *previous_index == instance_index
                            && *previous_key == execution_key
                            && primitive.same_primitive(source.primitive())
                    },
                ) {
                    return Err(
                        "same immutable primitive is bound twice in one authored domain".into(),
                    );
                }
                domain_primitives.push((instance_index, execution_key, source.primitive().clone()));
                let domain = active_rows
                    .iter()
                    .filter_map(|&(index, key, row)| {
                        (index == instance_index
                            && key == execution_key
                            && authored.clusters[row].entity == instance.entity)
                            .then_some(row)
                    })
                    .collect::<Vec<_>>();
                if domain.is_empty() {
                    remapped.push(source.clone().with_authored_rows(execution_key, Vec::new()));
                    continue;
                }
                // A partial explicit span never changes the primitive's full partition.
                let topology_rows = if self
                    .extract
                    .instances
                    .iter()
                    .filter(|row| row.entity == instance.entity)
                    .count()
                    == 1
                {
                    authored
                        .clusters
                        .iter()
                        .enumerate()
                        .filter_map(|(index, row)| (row.entity == instance.entity).then_some(index))
                        .collect::<Vec<_>>()
                } else {
                    domain.clone()
                };
                let mut identities = BTreeMap::new();
                for &index in &topology_rows {
                    let row = &authored.clusters[index];
                    if identities
                        .insert(row.cluster_id, row.page_id)
                        .is_some_and(|page| page != row.page_id)
                    {
                        return Err("one authored cluster has conflicting page identities".into());
                    }
                }
                if identities.is_empty() {
                    remapped.push(source.clone().with_authored_rows(execution_key, Vec::new()));
                    continue;
                }
                let asset = source
                    .primitive()
                    .primitive()
                    .virtual_geometry
                    .as_ref()
                    .ok_or("authored source has no actual cooked asset")?;
                let topology = source
                    .primitive()
                    .indexed_topology()
                    .map_err(str::to_owned)?;
                let mut leaves = asset
                    .cluster_headers
                    .iter()
                    .filter(|header| {
                        !asset
                            .cluster_headers
                            .iter()
                            .any(|child| child.parent_cluster_id == Some(header.cluster_id))
                    })
                    .map(|header| {
                        let runs = topology
                            .iter()
                            .filter(|run| {
                                run.page_id == header.page_id && run.cluster_id == header.cluster_id
                            })
                            .collect::<Vec<_>>();
                        if runs.len() != 1 {
                            return Err(
                                "cooked leaf has no unique actual indexed range".to_string()
                            );
                        }
                        Ok(runs[0])
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                leaves.sort_by_key(|run| run.indices.start);
                if identities.len() != leaves.len() {
                    return Err("authored partition differs from real cooked leaf partition".into());
                }
                let mut next_index = 0;
                for run in &leaves {
                    if run.indices.start != next_index || run.indices.start >= run.indices.end {
                        return Err(
                            "cooked leaf partition overlaps or omits original triangles".into()
                        );
                    }
                    next_index = run.indices.end;
                }
                if next_index as usize != source.primitive().primitive().indices.len() {
                    return Err(
                        "cooked leaf partition does not cover the original primitive".into(),
                    );
                }
                let mut bindings = Vec::new();
                let mut pages = Vec::new();
                let mut clusters = Vec::new();
                for ((cluster_id, page_id), run) in identities.into_iter().zip(leaves) {
                    let original_page = source
                        .page_ids()
                        .iter()
                        .filter(|(local, _)| *local == run.page_id)
                        .collect::<Vec<_>>();
                    let original_cluster = source
                        .cluster_ids()
                        .iter()
                        .filter(|(local, _)| *local == run.cluster_id)
                        .collect::<Vec<_>>();
                    if original_page.len() != 1 || original_cluster.len() != 1 {
                        return Err("cooked leaf remap is not unique".into());
                    }
                    let physical = payloads
                        .iter()
                        .enumerate()
                        .filter(|(_, payload)| {
                            payload
                                .as_ref()
                                .is_some_and(|payload| payload.page_id == original_page[0].1)
                        })
                        .map(|(index, _)| index)
                        .collect::<Vec<_>>();
                    if physical.len() != 1 {
                        return Err("cooked leaf has no unique physical payload".into());
                    }
                    let mut payload = payloads[physical[0]]
                        .take()
                        .ok_or("cooked payload already claimed")?;
                    let range_index = usize::try_from(run.range_index)
                        .map_err(|_| "cooked range index overflow")?;
                    if payload.cluster_ranges.len() != 1 {
                        return Err("leaf payload has ambiguous extra ranges".into());
                    }
                    let range = payload
                        .cluster_ranges
                        .get_mut(range_index)
                        .ok_or("cooked payload range is absent")?;
                    if range.cluster_id != original_cluster[0].1
                        || range.vertex_count != run.indices.end - run.indices.start
                    {
                        return Err(
                            "cooked physical range differs from admitted original indices".into(),
                        );
                    }
                    range.cluster_id = cluster_id;
                    payload.page_id = page_id;
                    let payload_index = admitted_payloads.len();
                    admitted_payloads.push(payload);
                    pages.push((run.page_id, page_id));
                    clusters.push((run.cluster_id, cluster_id));
                    for &index in domain
                        .iter()
                        .filter(|&&index| authored.clusters[index].cluster_id == cluster_id)
                    {
                        covered.insert((instance_index, execution_key, index));
                        bindings.push(AuthoredIndexedRowBinding {
                            instance_index,
                            stable_instance_key: execution_key,
                            author_array_index: u32::try_from(index)
                                .map_err(|_| "authored row index overflow")?,
                            local_page_id: run.page_id,
                            local_cluster_id: run.cluster_id,
                            local_range_index: run.range_index,
                            payload_index,
                            payload_range_index: range_index,
                        });
                    }
                }
                remapped.push(
                    crate::graphics::VirtualGeometryIndexedSourceAdmission::new(
                        source.stable_instance_key(),
                        source.primitive().clone(),
                        pages,
                        clusters,
                    )
                    .with_authored_rows(execution_key, bindings),
                );
            }
        }
        for identity in active_rows {
            if !covered.contains(&identity) {
                return Err(
                    "authored raw candidate has no matching immutable primitive admission".into(),
                );
            }
        }
        self.indexed_sources = remapped;
        self.resident_page_payloads = admitted_payloads;
        self.extract = authored.clone();
        // The authored path previously owned neither automatic diagnostic
        // references nor automatic BVH instances. Native geometry is separate.
        self.cpu_reference_instances.clear();
        self.bvh_visualization_instances.clear();
        Ok(self)
    }
}
