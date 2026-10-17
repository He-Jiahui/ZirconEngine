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
        }
    }
    pub fn stable_instance_key(&self) -> u64 {
        self.stable_instance_key
    }
    pub(crate) fn with_authored_execution_binding(
        mut self,
        execution_key: u64,
        resident_payload_indices: Vec<usize>,
    ) -> Self {
        self.authored_execution_key = Some(execution_key);
        self.resident_payload_indices = Some(resident_payload_indices);
        self
    }
    pub(crate) fn has_authored_execution_binding(&self) -> bool {
        self.authored_execution_key.is_some() && self.resident_payload_indices.is_some()
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
        if self.identity.instance_index.is_some()
            || self.identity.submission_index.is_none()
            || draw_ref_rank > u32::from(u16::MAX)
        {
            return Err("authored draw reference is not a checked legacy submission".into());
        }
        self.identity.draw_ref_index = draw_ref_index;
        self.identity.draw_ref_rank = Some(draw_ref_rank);
        Ok(self)
    }
    pub fn source_row(&self) -> u32 {
        self.source_row
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
    let mut rows = Vec::new();
    for segment in data
        .execution_segments
        .iter()
        .filter(|segment| segment.state == RenderVirtualGeometryExecutionState::Resident)
    {
        let instance_index = segment.instance_index.unwrap_or(u32::MAX);
        let (raw_start, raw_end) = if let Some(index) = segment.instance_index {
            let instance = data
                .instances
                .get(usize::try_from(index).map_err(|_| "instance index overflow")?)
                .ok_or("resident source instance index exceeds the actual instance table")?;
            if instance.entity != segment.entity
                || instance.stable_instance_key_or_legacy() != segment.stable_instance_key
            {
                return Err("resident source identity does not match its actual instance".into());
            }
            (
                instance.cluster_offset,
                instance
                    .cluster_offset
                    .checked_add(instance.cluster_count)
                    .ok_or("raw instance span overflow")?,
            )
        } else {
            if !data.instances.is_empty()
                || crate::core::framework::render::render_mesh_stable_instance_key(
                    segment.entity,
                    0,
                ) != segment.stable_instance_key
            {
                return Err("instance-less source is not the original legacy identity".into());
            }
            (
                0,
                u32::try_from(data.author_cluster_catalog.len())
                    .map_err(|_| "raw catalog length overflow")?,
            )
        };
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
            if raw_index < raw_start || raw_index >= raw_end {
                return Err("candidate raw index lies outside its instance".into());
            }
            let raw = data
                .author_cluster_catalog
                .get(usize::try_from(raw_index).map_err(|_| "raw index overflow")?)
                .ok_or("candidate raw cluster row is absent")?;
            if selected.entity != segment.entity {
                continue;
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
            for (page_index, page) in data
                .resident_page_payloads
                .iter()
                .enumerate()
                .filter(|(_, page)| page.page_id == segment.page_id)
            {
                for (range_index, range) in page
                    .cluster_ranges
                    .iter()
                    .enumerate()
                    .filter(|(_, range)| range.cluster_id == selected.cluster_id)
                {
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
                        source_row: u32::try_from(rows.len())
                            .map_err(|_| "source row exceeds u32")?,
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
                    });
                }
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
