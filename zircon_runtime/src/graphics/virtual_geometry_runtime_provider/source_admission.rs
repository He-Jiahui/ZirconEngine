use std::sync::Arc;

use crate::asset::{AssetUri, ModelPrimitiveAsset};
use crate::core::framework::render::{RenderMeshSnapshot, RenderVirtualGeometryDebugState};
use crate::core::resource::{ResourceId, ResourceState};

use super::VirtualGeometryRuntimeExtractOutput;

/// Full dependency snapshot. The numeric geometry fingerprint is never its identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VirtualGeometrySourceDependency {
    pub(crate) locator: AssetUri,
    pub(crate) resource_id: Option<ResourceId>,
    pub(crate) revision: Option<u64>,
    pub(crate) state: Option<ResourceState>,
}

/// Canonical immutable CPU geometry, held by the existing streamer entry and GPU mesh.
/// Only runtime resource admission may construct this type.
#[derive(Clone, Debug, PartialEq)]
pub struct VirtualGeometrySourceAdmission {
    source_id: ResourceId,
    source_revision: u64,
    dependencies: Vec<VirtualGeometrySourceDependency>,
    primitives: Arc<[ModelPrimitiveAsset]>,
    indexed_topology: Vec<Result<Vec<VirtualGeometryIndexedTopologyRun>, String>>,
}

impl VirtualGeometrySourceAdmission {
    pub(crate) fn new(
        source_id: ResourceId,
        source_revision: u64,
        dependencies: Vec<VirtualGeometrySourceDependency>,
        primitives: Vec<ModelPrimitiveAsset>,
    ) -> Self {
        let indexed_topology = primitives.iter().map(admitted_indexed_topology).collect();
        Self {
            source_id,
            source_revision,
            dependencies,
            primitives: primitives.into(),
            indexed_topology,
        }
    }

    pub fn source_id(&self) -> ResourceId {
        self.source_id
    }
    pub fn source_revision(&self) -> u64 {
        self.source_revision
    }
    pub fn dependencies(&self) -> &[VirtualGeometrySourceDependency] {
        &self.dependencies
    }
    pub fn primitives(&self) -> &[ModelPrimitiveAsset] {
        &self.primitives
    }

    pub(crate) fn primitive(
        self: &Arc<Self>,
        index: usize,
    ) -> Option<VirtualGeometryPrimitiveAdmission> {
        self.primitives.get(index)?;
        Some(VirtualGeometryPrimitiveAdmission {
            source: Arc::clone(self),
            index,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VirtualGeometryPrimitiveAdmission {
    source: Arc<VirtualGeometrySourceAdmission>,
    index: usize,
}

impl VirtualGeometryPrimitiveAdmission {
    pub fn source(&self) -> &Arc<VirtualGeometrySourceAdmission> {
        &self.source
    }
    pub fn primitive_index(&self) -> usize {
        self.index
    }
    pub fn primitive(&self) -> &ModelPrimitiveAsset {
        &self.source.primitives[self.index]
    }
}

/// Unsupported preserves the existing owned public callback for older providers.
/// Ready(None) means no automatic content and never invokes that fallback.
pub enum VirtualGeometryAdmittedExtract {
    Unsupported,
    Ready(Option<VirtualGeometryRuntimeExtractOutput>),
    Failed(String),
}

pub type VirtualGeometrySourceLoader<'a> =
    dyn FnMut(&RenderMeshSnapshot) -> Result<Vec<VirtualGeometryPrimitiveAdmission>, String> + 'a;

pub(crate) fn build_admitted_or_owned_extract(
    provider: &dyn super::VirtualGeometryRuntimeProvider,
    meshes: &[RenderMeshSnapshot],
    debug: Option<RenderVirtualGeometryDebugState>,
    admit: &mut VirtualGeometrySourceLoader<'_>,
    owned: impl FnOnce() -> Option<VirtualGeometryRuntimeExtractOutput>,
) -> Result<Option<VirtualGeometryRuntimeExtractOutput>, String> {
    match provider.build_extract_from_admitted_sources(meshes, debug, admit) {
        VirtualGeometryAdmittedExtract::Unsupported => Ok(owned()),
        VirtualGeometryAdmittedExtract::Ready(output) => Ok(output),
        VirtualGeometryAdmittedExtract::Failed(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::{VirtualGeometryRuntimeProvider, VirtualGeometryRuntimeState};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug)]
    struct OwnedProvider {
        calls: AtomicUsize,
    }
    impl VirtualGeometryRuntimeProvider for OwnedProvider {
        fn create_state(&self) -> Box<dyn VirtualGeometryRuntimeState> {
            panic!("these extraction contract tests do not create camera state")
        }
        fn build_extract_from_meshes(
            &self,
            _meshes: &[RenderMeshSnapshot],
            _debug: Option<RenderVirtualGeometryDebugState>,
            _load: &mut dyn FnMut(ResourceId) -> Option<crate::asset::ModelAsset>,
        ) -> Option<VirtualGeometryRuntimeExtractOutput> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            None
        }
    }

    #[derive(Debug)]
    struct SharedProvider {
        fail: bool,
    }
    impl VirtualGeometryRuntimeProvider for SharedProvider {
        fn create_state(&self) -> Box<dyn VirtualGeometryRuntimeState> {
            panic!("these extraction contract tests do not create camera state")
        }
        fn build_extract_from_admitted_sources(
            &self,
            _meshes: &[RenderMeshSnapshot],
            _debug: Option<RenderVirtualGeometryDebugState>,
            _admit: &mut VirtualGeometrySourceLoader<'_>,
        ) -> VirtualGeometryAdmittedExtract {
            if self.fail {
                VirtualGeometryAdmittedExtract::Failed("source reload".into())
            } else {
                VirtualGeometryAdmittedExtract::Ready(None)
            }
        }
    }

    #[test]
    fn unsupported_shared_admission_preserves_original_owned_provider() {
        let provider = OwnedProvider {
            calls: AtomicUsize::new(0),
        };
        let mut admit =
            |_mesh: &RenderMeshSnapshot| -> Result<Vec<VirtualGeometryPrimitiveAdmission>, String> {
                panic!("old provider must not require a new admission implementation")
            };
        let output = build_admitted_or_owned_extract(&provider, &[], None, &mut admit, || {
            provider.build_extract_from_meshes(&[], None, &mut |_| None)
        })
        .expect("old provider path remains valid");
        assert!(output.is_none());
        assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn admitted_empty_output_does_not_fall_back_to_owned_model_loading() {
        let mut admit = |_mesh: &RenderMeshSnapshot| Ok(Vec::new());
        let output = build_admitted_or_owned_extract(
            &SharedProvider { fail: false },
            &[],
            None,
            &mut admit,
            || panic!("Some(shared empty result) must not become unsupported"),
        )
        .expect("empty shared result is valid");
        assert!(output.is_none());
    }

    #[test]
    fn admitted_source_failure_is_not_silently_replaced_by_owned_geometry() {
        let mut admit = |_mesh: &RenderMeshSnapshot| Ok(Vec::new());
        let output = build_admitted_or_owned_extract(
            &SharedProvider { fail: true },
            &[],
            None,
            &mut admit,
            || panic!("source failure must not reload a different geometry revision"),
        );
        assert_eq!(output.err().as_deref(), Some("source reload"));
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VirtualGeometryIndexedSourceAdmission {
    stable_instance_key: u64,
    primitive: VirtualGeometryPrimitiveAdmission,
    page_ids: Vec<(u32, u32)>,
    cluster_ids: Vec<(u32, u32)>,
    authored_execution_key: Option<u64>,
    resident_payload_indices: Option<Vec<usize>>,
    authored_rows: Option<Vec<AuthoredIndexedRowBinding>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AuthoredIndexedRowBinding {
    pub(crate) instance_index: Option<u32>,
    pub(crate) stable_instance_key: u64,
    pub(crate) author_array_index: u32,
    pub(crate) local_page_id: u32,
    pub(crate) local_cluster_id: u32,
    pub(crate) local_range_index: u32,
    pub(crate) payload_index: usize,
    pub(crate) payload_range_index: usize,
}
impl VirtualGeometryIndexedSourceAdmission {
    pub fn new(
        stable_instance_key: u64,
        primitive: VirtualGeometryPrimitiveAdmission,
        page_ids: Vec<(u32, u32)>,
        cluster_ids: Vec<(u32, u32)>,
    ) -> Self {
        Self {
            stable_instance_key,
            primitive,
            page_ids,
            cluster_ids,
            authored_execution_key: None,
            resident_payload_indices: None,
            authored_rows: None,
        }
    }
    pub fn stable_instance_key(&self) -> u64 {
        self.stable_instance_key
    }
    pub(crate) fn has_authored_execution_binding(&self) -> bool {
        self.authored_execution_key.is_some() && self.authored_rows.is_some()
    }
    pub(crate) fn with_authored_rows(
        mut self,
        execution_key: u64,
        rows: Vec<AuthoredIndexedRowBinding>,
    ) -> Self {
        self.authored_execution_key = Some(execution_key);
        self.resident_payload_indices = Some(rows.iter().map(|row| row.payload_index).collect());
        self.authored_rows = Some(rows);
        self
    }
    pub(crate) fn authored_rows(&self) -> Option<&[AuthoredIndexedRowBinding]> {
        self.authored_rows.as_deref()
    }
    pub(crate) fn authored_run_for_row(
        &self,
        row: &VirtualGeometryIndexedSourceRow,
    ) -> Result<&VirtualGeometryIndexedTopologyRun, String> {
        let matches = self
            .authored_rows
            .as_deref()
            .ok_or("source has no authored binding")?
            .iter()
            .filter(|binding| {
                binding.instance_index == row.identity.instance_index
                    && binding.stable_instance_key == row.identity.stable_instance_key
                    && binding.author_array_index == row.author_array_index
                    && binding.payload_index == row.page_index
                    && binding.payload_range_index == row.range_index
            })
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err("source row has no unique actual authored binding".into());
        }
        let binding = matches[0];
        let pages = self
            .page_ids
            .iter()
            .filter(|(local, _)| *local == binding.local_page_id)
            .collect::<Vec<_>>();
        let clusters = self
            .cluster_ids
            .iter()
            .filter(|(local, _)| *local == binding.local_cluster_id)
            .collect::<Vec<_>>();
        if pages.len() != 1
            || clusters.len() != 1
            || pages[0].1 != row.identity.page_id
            || clusters[0].1 != row.cluster_id
        {
            return Err(
                "authored binding disagrees with its original local-to-authored topology map"
                    .into(),
            );
        }
        let runs = self.primitive.indexed_topology().map_err(str::to_owned)?;
        let runs = runs
            .iter()
            .filter(|run| {
                run.page_id == binding.local_page_id
                    && run.cluster_id == binding.local_cluster_id
                    && run.range_index == binding.local_range_index
            })
            .collect::<Vec<_>>();
        if runs.len() != 1 {
            return Err("authored binding lost its admitted original range".into());
        }
        Ok(runs[0])
    }
    pub(crate) fn execution_stable_instance_key(&self) -> u64 {
        self.authored_execution_key
            .unwrap_or(self.stable_instance_key)
    }
    pub(crate) fn owns_resident_payload(&self, page_index: usize) -> bool {
        // Ordinary automatic sources retain their original matching semantics.
        self.resident_payload_indices
            .as_ref()
            .map_or(true, |indices| indices.contains(&page_index))
    }
    pub fn primitive(&self) -> &VirtualGeometryPrimitiveAdmission {
        &self.primitive
    }
    pub(crate) fn page_ids(&self) -> &[(u32, u32)] {
        &self.page_ids
    }
    pub(crate) fn cluster_ids(&self) -> &[(u32, u32)] {
        &self.cluster_ids
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VirtualGeometryIndexedTopologyRun {
    pub(crate) page_id: u32,
    pub(crate) cluster_id: u32,
    pub(crate) range_index: u32,
    pub(crate) indices: std::ops::Range<u32>,
}
fn admitted_indexed_topology(
    primitive: &ModelPrimitiveAsset,
) -> Result<Vec<VirtualGeometryIndexedTopologyRun>, String> {
    let Some(asset) = primitive.virtual_geometry.as_ref() else {
        return Ok(Vec::new());
    };
    if asset.cluster_page_headers.len() != asset.cluster_page_data.len() {
        return Err("virtual geometry page headers and payloads differ in length".into());
    }
    let mut runs = Vec::new();
    for (header, payload) in asset
        .cluster_page_headers
        .iter()
        .zip(&asset.cluster_page_data)
    {
        let word = |index: usize| -> Result<u32, String> {
            let start = index.checked_mul(4).ok_or("payload word offset overflow")?;
            let end = start.checked_add(4).ok_or("payload word end overflow")?;
            let raw = payload
                .get(start..end)
                .ok_or("truncated virtual geometry payload")?;
            Ok(u32::from_le_bytes(
                raw.try_into().map_err(|_| "unaligned payload word")?,
            ))
        };
        if payload.len() % 4 != 0 || word(0)? != u32::from_le_bytes(*b"ZVG0") || word(1)? != 1 {
            return Err("unsupported virtual geometry indexed payload".into());
        }
        let count = usize::try_from(word(6)?).map_err(|_| "payload count exceeds usize")?;
        let expected = count
            .checked_mul(4)
            .and_then(|n| n.checked_add(7))
            .and_then(|n| n.checked_mul(4))
            .ok_or("payload length overflow")?;
        if expected != payload.len() {
            return Err("payload count does not describe its actual bytes".into());
        }
        let mut range_index = 0_u32;
        for item in 0..count {
            let base = 7 + item * 4;
            let cluster_id = word(base + 1)?;
            if !asset
                .cluster_headers
                .iter()
                .any(|cluster| cluster.cluster_id == cluster_id)
            {
                return Err("indexed payload references an unknown local cluster".into());
            }
            let start = word(base + 2)?
                .checked_mul(3)
                .ok_or("triangle index start overflow")?;
            let len = word(base + 3)?
                .checked_mul(3)
                .ok_or("triangle index count overflow")?;
            let end = start
                .checked_add(len)
                .ok_or("triangle index end overflow")?;
            let slice = primitive
                .indices
                .get(start as usize..end as usize)
                .ok_or("indexed payload exceeds its admitted primitive index buffer")?;
            if slice
                .iter()
                .any(|index| *index as usize >= primitive.vertices.len())
            {
                return Err("indexed payload references an unavailable admitted vertex".into());
            }
            if len != 0 {
                runs.push(VirtualGeometryIndexedTopologyRun {
                    page_id: header.page_id,
                    cluster_id,
                    range_index,
                    indices: start..end,
                });
                range_index = range_index
                    .checked_add(1)
                    .ok_or("page indexed run count overflow")?;
            }
        }
    }
    Ok(runs)
}
impl VirtualGeometryPrimitiveAdmission {
    pub(crate) fn indexed_topology(&self) -> Result<&[VirtualGeometryIndexedTopologyRun], &str> {
        match &self.source.indexed_topology[self.index] {
            Ok(runs) => Ok(runs),
            Err(error) => Err(error.as_str()),
        }
    }
    pub(crate) fn same_primitive(&self, other: &Self) -> bool {
        self.index == other.index && Arc::ptr_eq(&self.source, &other.source)
    }
}

#[derive(Clone, Debug)]
pub struct VirtualGeometryIndexedSourceRow {
    identity: crate::core::framework::render::RenderVirtualGeometryExecutionIdentity,
    source_row: u32,
    candidate_index: u32,
    author_array_index: u32,
    normal_cluster_total: u32,
    cluster_id: u32,
    cluster_ordinal: u32,
    page_index: usize,
    range_index: usize,
    lineage_depth: u32,
    frontier_rank: u32,
    authored_topology_binding: bool,
}
impl VirtualGeometryIndexedSourceRow {
    pub fn identity(
        &self,
    ) -> crate::core::framework::render::RenderVirtualGeometryExecutionIdentity {
        self.identity
    }
    pub(crate) fn with_draw_reference_identity(
        mut self,
        draw_ref_index: u32,
        draw_ref_rank: u32,
    ) -> Result<Self, String> {
        if !self.authored_topology_binding
            || self.identity.submission_index.is_none()
            || draw_ref_rank > u32::from(u16::MAX)
        {
            return Err("authored draw reference has no checked topology binding".into());
        }
        self.identity.draw_ref_index = draw_ref_index;
        self.identity.draw_ref_rank = Some(draw_ref_rank);
        Ok(self)
    }
    pub fn source_row(&self) -> u32 {
        self.source_row
    }
    pub fn has_authored_topology_binding(&self) -> bool {
        self.authored_topology_binding
    }
    pub fn candidate_index(&self) -> u32 {
        self.candidate_index
    }
    pub fn author_array_index(&self) -> u32 {
        self.author_array_index
    }
    pub fn normal_cluster_total(&self) -> u32 {
        self.normal_cluster_total
    }
    pub fn cluster_id(&self) -> u32 {
        self.cluster_id
    }
    pub fn cluster_ordinal(&self) -> u32 {
        self.cluster_ordinal
    }
    pub fn page_index(&self) -> usize {
        self.page_index
    }
    pub fn range_index(&self) -> usize {
        self.range_index
    }
    pub fn lineage_depth(&self) -> u32 {
        self.lineage_depth
    }
    pub fn frontier_rank(&self) -> u32 {
        self.frontier_rank
    }
}
pub fn virtual_geometry_indexed_source_rows(
    data: &crate::core::framework::render::RenderVirtualGeometryExecutionData,
) -> Result<Vec<VirtualGeometryIndexedSourceRow>, String> {
    use crate::core::framework::render::{
        RenderVirtualGeometryExecutionIdentity, RenderVirtualGeometryExecutionState,
    };
    if data.selected_clusters.len() != data.candidate_author_indices.len() {
        return Err("candidate raw provenance is incomplete".into());
    }
    if let Some(bindings) = &data.indexed_candidate_payload_ranges {
        let mut seen = std::collections::BTreeSet::new();
        for &(candidate, page, range) in bindings {
            let selected = data
                .selected_clusters
                .get(candidate as usize)
                .ok_or("bound candidate is absent")?;
            let payload = data
                .resident_page_payloads
                .get(page)
                .ok_or("bound payload is absent")?;
            let physical = payload
                .cluster_ranges
                .get(range)
                .ok_or("bound payload range is absent")?;
            if !seen.insert((candidate, page, range))
                || payload.page_id != selected.page_id
                || physical.cluster_id != selected.cluster_id
                || physical.vertex_count == 0
            {
                return Err("bound physical range differs from its original candidate".into());
            }
            let end = physical
                .vertex_start
                .checked_add(physical.vertex_count)
                .ok_or("bound vertex end overflow")?;
            payload
                .vertices
                .get(physical.vertex_start as usize..end as usize)
                .ok_or("bound vertex range exceeds physical payload")?;
        }
    }
    let mut rows = Vec::new();
    for segment in data
        .execution_segments
        .iter()
        .filter(|segment| segment.state == RenderVirtualGeometryExecutionState::Resident)
    {
        let instance_index = segment.instance_index.unwrap_or(u32::MAX);
        let raw_domain = crate::core::framework::render::RenderVirtualGeometryExtract::checked_cluster_row_indices(
            &data.instances, &data.author_cluster_catalog, segment.entity,
            segment.stable_instance_key, segment.instance_index).map_err(str::to_owned)?;
        let end = segment
            .cluster_start_ordinal
            .checked_add(segment.cluster_span_count)
            .ok_or("resident source cluster span overflow")?;
        for (candidate_index, (selected, &(author_instance, raw_index, total))) in data
            .selected_clusters
            .iter()
            .zip(&data.candidate_author_indices)
            .enumerate()
        {
            if author_instance != instance_index {
                continue;
            }
            let raw = data
                .author_cluster_catalog
                .get(usize::try_from(raw_index).map_err(|_| "raw index overflow")?)
                .ok_or("candidate raw cluster row is absent")?;
            if selected.entity != segment.entity {
                continue;
            }
            if !raw_domain.contains(&(raw_index as usize)) {
                return Err("candidate raw index lies outside its instance".into());
            }
            if selected.instance_index != segment.instance_index
                || raw.entity != selected.entity
                || raw.cluster_id != selected.cluster_id
                || raw.page_id != selected.page_id
                || raw.lod_level != selected.lod_level
            {
                return Err(
                    "GPU upload candidate disagrees with its original raw author row".into(),
                );
            }
            let ordinal = selected.cluster_ordinal;
            if ordinal >= total || end > total {
                return Err("normal ordinal exceeds existing policy total".into());
            }
            if selected.state != RenderVirtualGeometryExecutionState::Resident
                || selected.page_id != segment.page_id
                || ordinal < segment.cluster_start_ordinal
                || ordinal >= end
            {
                continue;
            }
            let physical_ranges = match &data.indexed_candidate_payload_ranges {
                Some(bindings) => bindings
                    .iter()
                    .filter(|(candidate, _, _)| *candidate as usize == candidate_index)
                    .map(|&(_, page, range)| (page, range))
                    .collect::<Vec<_>>(),
                None => data
                    .resident_page_payloads
                    .iter()
                    .enumerate()
                    .filter(|(_, page)| page.page_id == segment.page_id)
                    .flat_map(|(page_index, page)| {
                        page.cluster_ranges
                            .iter()
                            .enumerate()
                            .filter(move |(_, range)| range.cluster_id == selected.cluster_id)
                            .map(move |(range_index, _)| (page_index, range_index))
                    })
                    .collect(),
            };
            for (page_index, range_index) in physical_ranges {
                let page = data
                    .resident_page_payloads
                    .get(page_index)
                    .ok_or("physical page is absent")?;
                let range = page
                    .cluster_ranges
                    .get(range_index)
                    .ok_or("physical range is absent")?;
                let start = range.vertex_start as usize;
                let end = start
                    .checked_add(range.vertex_count as usize)
                    .ok_or("resident source vertex range overflow")?;
                if range.vertex_count == 0 {
                    continue;
                }
                page.vertices
                    .get(start..end)
                    .ok_or("resident source range exceeds actual page vertices")?;
                if rows.len() == 4096 {
                    return Err("resident source exceeds the declared 4096-row ABI".into());
                }
                rows.push(VirtualGeometryIndexedSourceRow {
                    identity: RenderVirtualGeometryExecutionIdentity::from_segment(segment),
                    source_row: u32::try_from(rows.len()).map_err(|_| "source row exceeds u32")?,
                    candidate_index: u32::try_from(candidate_index)
                        .map_err(|_| "candidate index exceeds u32")?,
                    cluster_id: selected.cluster_id,
                    cluster_ordinal: ordinal,
                    author_array_index: raw_index,
                    normal_cluster_total: total,
                    page_index,
                    range_index,
                    lineage_depth: segment.lineage_depth,
                    frontier_rank: segment.frontier_rank,
                    authored_topology_binding: data.indexed_candidate_payload_ranges.is_some(),
                });
            }
        }
    }
    Ok(rows)
}

#[derive(Clone, Debug)]
pub struct VirtualGeometryIndexedAuthorRun {
    source: VirtualGeometryIndexedSourceRow,
    index_range: std::ops::Range<u32>,
    source_index_count: u32,
    gpu_scene_instance: u32,
}
impl VirtualGeometryIndexedAuthorRun {
    pub(crate) fn new(
        source: VirtualGeometryIndexedSourceRow,
        index_range: std::ops::Range<u32>,
        source_index_count: u32,
        gpu_scene_instance: u32,
    ) -> Self {
        Self {
            source,
            index_range,
            source_index_count,
            gpu_scene_instance,
        }
    }
    pub fn source(&self) -> &VirtualGeometryIndexedSourceRow {
        &self.source
    }
    pub fn index_range(&self) -> std::ops::Range<u32> {
        self.index_range.clone()
    }
    pub fn source_index_count(&self) -> u32 {
        self.source_index_count
    }
    pub fn gpu_scene_instance(&self) -> u32 {
        self.gpu_scene_instance
    }
}

#[cfg(test)]
mod authored_physical_binding_tests {
    use super::virtual_geometry_indexed_source_rows;
    use crate::core::framework::render::RenderVirtualGeometryExecutionData;

    #[test]
    fn explicit_empty_physical_bindings_remain_empty_and_invalid_bindings_fail() {
        let mut data = RenderVirtualGeometryExecutionData::default();
        assert!(data.indexed_candidate_payload_ranges.is_none());
        data.indexed_candidate_payload_ranges = Some(Vec::new());
        assert!(virtual_geometry_indexed_source_rows(&data)
            .unwrap()
            .is_empty());
        // Malformed physical ownership must fail even when no resident segment
        // would consume it; it cannot silently select the automatic path.
        data.indexed_candidate_payload_ranges = Some(vec![(0, 0, 0)]);
        assert!(virtual_geometry_indexed_source_rows(&data).is_err());
        assert_eq!(data.indexed_candidate_payload_ranges, Some(vec![(0, 0, 0)]));
    }
}
