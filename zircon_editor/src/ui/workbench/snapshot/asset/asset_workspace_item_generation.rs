use std::collections::HashMap;
use std::ops::Index;
use std::sync::Arc;

use super::AssetItemSnapshot;

const ASSET_WORKSPACE_ITEM_CHUNK_SIZE: usize = 64;

/// Immutable visible-asset rows shared by every surface and selection overlay in one generation.
#[derive(Clone, Debug)]
pub struct AssetWorkspaceItemGeneration {
    chunks: Arc<[Arc<[AssetItemSnapshot]>]>,
    len: usize,
    indices_by_uuid: Arc<HashMap<String, usize>>,
    indices_by_locator: Arc<HashMap<String, usize>>,
    selected_indices: Arc<[usize]>,
}

impl AssetWorkspaceItemGeneration {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get(&self, index: usize) -> Option<&AssetItemSnapshot> {
        let chunk = self.chunks.get(index / ASSET_WORKSPACE_ITEM_CHUNK_SIZE)?;
        chunk.get(index % ASSET_WORKSPACE_ITEM_CHUNK_SIZE)
    }

    pub fn iter(&self) -> impl Iterator<Item = &AssetItemSnapshot> {
        self.chunks.iter().flat_map(|chunk| chunk.iter())
    }

    pub(crate) fn item_chunks(&self) -> impl ExactSizeIterator<Item = &[AssetItemSnapshot]> {
        self.chunks.iter().map(AsRef::as_ref)
    }

    pub(crate) fn selected_index(&self, uuid: &str) -> Option<usize> {
        self.indices_by_uuid.get(uuid).copied()
    }

    pub(crate) fn locator_index(&self, locator: &str) -> Option<usize> {
        self.indices_by_locator.get(locator).copied()
    }

    pub(crate) fn selected_indices(&self) -> &[usize] {
        &self.selected_indices
    }

    pub fn shares_items_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.chunks, &other.chunks)
    }

    pub(crate) fn shares_item_identity_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.indices_by_uuid, &other.indices_by_uuid)
    }

    #[cfg(test)]
    fn shares_selected_indices_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.selected_indices, &other.selected_indices)
    }

    pub(crate) fn replace_existing_items(
        &self,
        replacements: impl IntoIterator<Item = AssetItemSnapshot>,
    ) -> Option<Self> {
        let mut replacements_by_chunk = HashMap::<usize, Vec<(usize, AssetItemSnapshot)>>::new();
        for replacement in replacements {
            let index = self.selected_index(&replacement.uuid)?;
            let current = self.get(index)?;
            if current.locator != replacement.locator {
                return None;
            }
            replacements_by_chunk
                .entry(index / ASSET_WORKSPACE_ITEM_CHUNK_SIZE)
                .or_default()
                .push((index % ASSET_WORKSPACE_ITEM_CHUNK_SIZE, replacement));
        }
        if replacements_by_chunk.is_empty() {
            return Some(self.clone());
        }

        let mut chunks = self.chunks.to_vec();
        let mut selected_indices = self.selected_indices.to_vec();
        for (chunk_index, replacements) in replacements_by_chunk {
            let mut items = chunks.get(chunk_index)?.to_vec();
            for (item_index, replacement) in replacements {
                let selected_index = chunk_index
                    .saturating_mul(ASSET_WORKSPACE_ITEM_CHUNK_SIZE)
                    .saturating_add(item_index);
                update_selected_index(&mut selected_indices, selected_index, replacement.selected);
                *items.get_mut(item_index)? = replacement;
            }
            chunks[chunk_index] = items.into();
        }
        Some(Self {
            chunks: chunks.into(),
            len: self.len,
            indices_by_uuid: Arc::clone(&self.indices_by_uuid),
            indices_by_locator: Arc::clone(&self.indices_by_locator),
            selected_indices: selected_indices.into(),
        })
    }

    pub(crate) fn project_items(&self, mut project: impl FnMut(&mut AssetItemSnapshot)) -> Self {
        let chunks = self
            .chunks
            .iter()
            .map(|chunk| project_chunk(chunk, &mut project))
            .collect::<Vec<_>>();
        let selected_indices = selected_indices_from_chunks(&chunks);
        Self {
            chunks: chunks.into(),
            len: self.len,
            indices_by_uuid: Arc::clone(&self.indices_by_uuid),
            indices_by_locator: Arc::clone(&self.indices_by_locator),
            selected_indices: selected_indices.into(),
        }
    }

    pub(crate) fn project_items_reusing(
        &self,
        previous_source: &Self,
        previous_projected: &Self,
        mut project: impl FnMut(&mut AssetItemSnapshot),
    ) -> Self {
        let aligned = self.len == previous_source.len
            && self.len == previous_projected.len
            && self.chunks.len() == previous_source.chunks.len()
            && self.chunks.len() == previous_projected.chunks.len();
        if !aligned {
            return self.project_items(project);
        }

        if self
            .chunks
            .iter()
            .zip(previous_source.chunks.iter())
            .all(|(chunk, previous)| Arc::ptr_eq(chunk, previous))
        {
            return Self {
                chunks: Arc::clone(&previous_projected.chunks),
                len: self.len,
                indices_by_uuid: Arc::clone(&self.indices_by_uuid),
                indices_by_locator: Arc::clone(&self.indices_by_locator),
                selected_indices: Arc::clone(&previous_projected.selected_indices),
            };
        }

        let mut chunks = Vec::with_capacity(self.chunks.len());
        let mut selected_indices = previous_projected.selected_indices.to_vec();
        for (index, chunk) in self.chunks.iter().enumerate() {
            if Arc::ptr_eq(chunk, &previous_source.chunks[index]) {
                chunks.push(Arc::clone(&previous_projected.chunks[index]));
            } else {
                let chunk = project_chunk(chunk, &mut project);
                replace_chunk_selected_indices(&mut selected_indices, index, &chunk);
                chunks.push(chunk);
            }
        }
        Self {
            chunks: chunks.into(),
            len: self.len,
            indices_by_uuid: Arc::clone(&self.indices_by_uuid),
            indices_by_locator: Arc::clone(&self.indices_by_locator),
            selected_indices: selected_indices.into(),
        }
    }

    pub(crate) fn shares_item_chunk_with(&self, index: usize, other: &Self) -> bool {
        let chunk_index = index / ASSET_WORKSPACE_ITEM_CHUNK_SIZE;
        self.chunks
            .get(chunk_index)
            .zip(other.chunks.get(chunk_index))
            .is_some_and(|(left, right)| Arc::ptr_eq(left, right))
    }

    #[cfg(test)]
    pub(crate) fn push(&mut self, item: AssetItemSnapshot) {
        let mut items = self.iter().cloned().collect::<Vec<_>>();
        items.push(item);
        *self = items.into();
    }
}

fn project_chunk(
    chunk: &Arc<[AssetItemSnapshot]>,
    project: &mut impl FnMut(&mut AssetItemSnapshot),
) -> Arc<[AssetItemSnapshot]> {
    let mut items = chunk.to_vec();
    for item in &mut items {
        project(item);
    }
    items.into()
}

fn selected_indices_from_chunks(chunks: &[Arc<[AssetItemSnapshot]>]) -> Vec<usize> {
    chunks
        .iter()
        .enumerate()
        .flat_map(|(chunk_index, chunk)| {
            chunk
                .iter()
                .enumerate()
                .filter_map(move |(item_index, item)| {
                    item.selected
                        .then_some(chunk_index * ASSET_WORKSPACE_ITEM_CHUNK_SIZE + item_index)
                })
        })
        .collect()
}

fn replace_chunk_selected_indices(
    selected_indices: &mut Vec<usize>,
    chunk_index: usize,
    chunk: &[AssetItemSnapshot],
) {
    selected_indices.retain(|index| index / ASSET_WORKSPACE_ITEM_CHUNK_SIZE != chunk_index);
    for (item_index, item) in chunk.iter().enumerate() {
        if item.selected {
            update_selected_index(
                selected_indices,
                chunk_index * ASSET_WORKSPACE_ITEM_CHUNK_SIZE + item_index,
                true,
            );
        }
    }
}

fn update_selected_index(selected_indices: &mut Vec<usize>, index: usize, selected: bool) {
    match (selected_indices.binary_search(&index), selected) {
        (Err(position), true) => selected_indices.insert(position, index),
        (Ok(position), false) => {
            selected_indices.remove(position);
        }
        _ => {}
    }
}

impl Default for AssetWorkspaceItemGeneration {
    fn default() -> Self {
        Self {
            chunks: Arc::from([]),
            len: 0,
            indices_by_uuid: Arc::new(HashMap::new()),
            indices_by_locator: Arc::new(HashMap::new()),
            selected_indices: Arc::from([]),
        }
    }
}

impl From<Vec<AssetItemSnapshot>> for AssetWorkspaceItemGeneration {
    fn from(items: Vec<AssetItemSnapshot>) -> Self {
        let len = items.len();
        let mut indices_by_uuid = HashMap::with_capacity(len);
        let mut indices_by_locator = HashMap::with_capacity(len);
        let mut selected_indices = Vec::new();
        for (index, item) in items.iter().enumerate() {
            let replaced = indices_by_uuid.insert(item.uuid.clone(), index);
            debug_assert!(replaced.is_none(), "visible asset UUIDs must be unique");
            let replaced = indices_by_locator.insert(item.locator.clone(), index);
            debug_assert!(replaced.is_none(), "visible asset locators must be unique");
            if item.selected {
                selected_indices.push(index);
            }
        }

        let mut chunks = Vec::with_capacity(len.div_ceil(ASSET_WORKSPACE_ITEM_CHUNK_SIZE));
        let mut chunk = Vec::with_capacity(ASSET_WORKSPACE_ITEM_CHUNK_SIZE);
        for item in items {
            chunk.push(item);
            if chunk.len() == ASSET_WORKSPACE_ITEM_CHUNK_SIZE {
                chunks.push(std::mem::take(&mut chunk).into());
                chunk = Vec::with_capacity(ASSET_WORKSPACE_ITEM_CHUNK_SIZE);
            }
        }
        if !chunk.is_empty() {
            chunks.push(chunk.into());
        }

        Self {
            chunks: chunks.into(),
            len,
            indices_by_uuid: Arc::new(indices_by_uuid),
            indices_by_locator: Arc::new(indices_by_locator),
            selected_indices: selected_indices.into(),
        }
    }
}

impl FromIterator<AssetItemSnapshot> for AssetWorkspaceItemGeneration {
    fn from_iter<T: IntoIterator<Item = AssetItemSnapshot>>(iter: T) -> Self {
        iter.into_iter().collect::<Vec<_>>().into()
    }
}

impl Index<usize> for AssetWorkspaceItemGeneration {
    type Output = AssetItemSnapshot;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
            .unwrap_or_else(|| panic!("asset workspace item index {index} is out of bounds"))
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime_interface::resource::ResourceKind;

    use super::super::AssetTypeProjectionSnapshot;
    use super::*;

    fn item(index: usize) -> AssetItemSnapshot {
        AssetItemSnapshot {
            uuid: format!("asset-{index}"),
            locator: format!("res://asset-{index}.zdata"),
            display_name: format!("Asset {index}"),
            file_name: format!("asset-{index}.zdata"),
            extension: "zdata".to_string(),
            kind: ResourceKind::Data,
            asset_type: AssetTypeProjectionSnapshot::default(),
            preview_artifact_path: String::new(),
            dirty: false,
            diagnostics: Vec::new(),
            selected: index % 3 == 0,
            resource_state: None,
            resource_revision: None,
        }
    }

    fn legacy_project_items_reusing(
        source: &AssetWorkspaceItemGeneration,
        previous_source: &AssetWorkspaceItemGeneration,
        previous_projected: &AssetWorkspaceItemGeneration,
    ) -> AssetWorkspaceItemGeneration {
        let mut chunks = Vec::with_capacity(source.chunks.len());
        let mut selected_indices = previous_projected.selected_indices.to_vec();
        for (index, chunk) in source.chunks.iter().enumerate() {
            if Arc::ptr_eq(chunk, &previous_source.chunks[index]) {
                chunks.push(Arc::clone(&previous_projected.chunks[index]));
            } else {
                let chunk = project_chunk(chunk, &mut |_| {});
                replace_chunk_selected_indices(&mut selected_indices, index, &chunk);
                chunks.push(chunk);
            }
        }
        AssetWorkspaceItemGeneration {
            chunks: chunks.into(),
            len: source.len,
            indices_by_uuid: Arc::clone(&source.indices_by_uuid),
            indices_by_locator: Arc::clone(&source.indices_by_locator),
            selected_indices: selected_indices.into(),
        }
    }

    #[test]
    fn optimization_batch_gz_editor581_full_reuse_shares_projected_arrays() {
        let source = (0..256).map(item).collect::<AssetWorkspaceItemGeneration>();
        let projected = source.project_items(|item| item.display_name.push_str(" projected"));
        let next = source.project_items_reusing(&source, &projected, |item| {
            item.display_name.push_str(" should-not-run")
        });

        assert!(next.shares_items_with(&projected));
        assert!(next.shares_item_identity_with(&projected));
        assert!(next.shares_selected_indices_with(&projected));
        assert_eq!(next[42].display_name, "Asset 42 projected");
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn optimization_batch_gz_editor581_project_reuse_performance_evidence() {
        let source = (0..8_192)
            .map(item)
            .collect::<AssetWorkspaceItemGeneration>();
        let projected = source.project_items(|item| item.display_name.push_str(" projected"));
        let mut legacy_samples = Vec::with_capacity(17);
        let mut optimized_samples = Vec::with_capacity(17);
        for _ in 0..17 {
            let started = Instant::now();
            black_box(legacy_project_items_reusing(
                black_box(&source),
                black_box(&source),
                black_box(&projected),
            ));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(source.project_items_reusing(
                black_box(&source),
                black_box(&projected),
                |_| {},
            ));
            optimized_samples.push(started.elapsed().as_nanos());
        }

        legacy_samples.sort_unstable();
        optimized_samples.sort_unstable();
        let legacy_p95 = legacy_samples[16];
        let optimized_p95 = optimized_samples[16];
        println!(
            "EDITOR581_PROJECT_REUSE_BENCH_V1 item_count={} chunks={} legacy_p95_ns={} optimized_p95_ns={} target_ratio_bp=7000",
            source.len,
            source.chunks.len(),
            legacy_p95,
            optimized_p95,
        );
        assert!(
            optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(7_000),
            "projected asset reuse P95 {optimized_p95} ns exceeded 70% of legacy {legacy_p95} ns"
        );
    }
}
