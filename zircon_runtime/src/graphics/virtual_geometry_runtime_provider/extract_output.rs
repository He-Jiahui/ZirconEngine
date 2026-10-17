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
    ) -> Result<Self, String> {
        use std::collections::{BTreeMap, BTreeSet};
        if !authored.instances.is_empty() {
            return Err(
                "authored instance ranges require an explicit canonical topology mapping".into(),
            );
        }
        let mut page_map = BTreeMap::new();
        let mut cluster_map = BTreeMap::new();
        let mut remapped = Vec::new();
        let mut covered = BTreeSet::new();
        let mut canonical_keys = BTreeSet::new();
        for source in &self.indexed_sources {
            let instances = self
                .extract
                .instances
                .iter()
                .filter(|instance| {
                    instance.stable_instance_key_or_legacy() == source.stable_instance_key()
                })
                .collect::<Vec<_>>();
            if instances.len() != 1 {
                return Err("canonical admission instance is not unique".into());
            }
            let instance = instances[0];
            if !canonical_keys.insert(source.stable_instance_key()) {
                return Err("canonical primitive admission key is not unique".into());
            }
            let actual_clusters = self
                .extract
                .clusters
                .get(
                    usize::try_from(instance.cluster_offset)
                        .map_err(|_| "canonical cluster offset overflow")?
                        ..usize::try_from(
                            instance
                                .cluster_offset
                                .checked_add(instance.cluster_count)
                                .ok_or("canonical cluster end overflow")?,
                        )
                        .map_err(|_| "canonical cluster end overflow")?,
                )
                .ok_or("canonical cluster range exceeds its immutable admission")?;
            let authored_ids = authored
                .clusters
                .iter()
                .filter(|row| row.entity == instance.entity)
                .map(|row| (row.cluster_id, row.page_id))
                .collect::<BTreeSet<_>>();
            if actual_clusters.len() != 1
                || authored_ids.len() != 1
                || source.cluster_ids().len() != 1
                || source.page_ids().len() != 1
            {
                return Err(
                    "authored topology has no proven unique cooked cluster and page mapping".into(),
                );
            }
            let &(cluster, page) = authored_ids
                .iter()
                .next()
                .ok_or("authored cluster identity is absent")?;
            if !authored.pages.iter().any(|row| row.page_id == page) {
                return Err("authored cluster has no original authored page".into());
            }
            let actual = &actual_clusters[0];
            let (local_cluster, global_cluster) = source.cluster_ids()[0];
            let (local_page, global_page) = source.page_ids()[0];
            if global_cluster != actual.cluster_id || global_page != actual.page_id {
                return Err(
                    "canonical primitive maps disagree with its admitted cluster range".into(),
                );
            }
            // Geometry ranges were parsed and checked against the same immutable
            // primitive when its revision was admitted; no whole-mesh fallback.
            if source
                .primitive()
                .indexed_topology()
                .map_err(str::to_owned)?
                .is_empty()
            {
                return Err("cooked authored primitive has no real indexed topology".into());
            }
            if page_map.insert(global_page, page).is_some()
                || cluster_map.insert(global_cluster, cluster).is_some()
            {
                return Err("canonical admission IDs are not globally unique".into());
            }
            // Capture physical payload ownership before authored IDs alias the
            // distinct cooked pages. Never cross-join two primitives' geometry.
            let resident_payload_indices = self
                .resident_page_payloads
                .iter()
                .enumerate()
                .filter(|(_, payload)| payload.page_id == global_page)
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            if resident_payload_indices.is_empty() {
                return Err("canonical primitive has no admitted resident payload".into());
            }
            covered.insert(instance.entity);
            remapped.push(
                crate::graphics::VirtualGeometryIndexedSourceAdmission::new(
                    source.stable_instance_key(),
                    source.primitive().clone(),
                    vec![(local_page, page)],
                    vec![(local_cluster, cluster)],
                )
                .with_authored_execution_binding(
                    crate::core::framework::render::render_mesh_stable_instance_key(
                        instance.entity,
                        0,
                    ),
                    resident_payload_indices,
                ),
            );
        }
        if authored
            .clusters
            .iter()
            .any(|row| !covered.contains(&row.entity))
        {
            return Err(
                "authored raw candidate has no matching immutable primitive admission".into(),
            );
        }
        for payload in &mut self.resident_page_payloads {
            payload.page_id = *page_map
                .get(&payload.page_id)
                .ok_or("cooked payload page is not admitted")?;
            for range in &mut payload.cluster_ranges {
                range.cluster_id = *cluster_map
                    .get(&range.cluster_id)
                    .ok_or("cooked payload cluster is not admitted")?;
            }
        }
        self.indexed_sources = remapped;
        self.extract = authored.clone();
        // The authored path previously owned neither automatic diagnostic
        // references nor automatic BVH instances. Native geometry is separate.
        self.cpu_reference_instances.clear();
        self.bvh_visualization_instances.clear();
        Ok(self)
    }
}
