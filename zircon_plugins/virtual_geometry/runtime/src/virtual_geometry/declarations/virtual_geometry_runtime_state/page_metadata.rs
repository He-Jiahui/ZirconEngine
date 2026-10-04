use std::collections::{BTreeMap, BTreeSet};

use super::runtime_state::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::virtual_geometry) fn has_page_size(&self, page_id: u32) -> bool {
        self.page_sizes.contains_key(&page_id)
    }

    pub(in crate::virtual_geometry) fn page_size_bytes(&self, page_id: u32) -> u64 {
        self.page_sizes.get(&page_id).copied().unwrap_or_default()
    }

    pub(in crate::virtual_geometry) fn insert_page_size(
        &mut self,
        page_id: u32,
        size_bytes: u64,
    ) -> Option<u64> {
        self.page_sizes.insert(page_id, size_bytes)
    }

    pub(in crate::virtual_geometry) fn retain_page_sizes(
        &mut self,
        mut retain: impl FnMut(&u32) -> bool,
    ) {
        self.page_sizes.retain(|page_id, _| retain(page_id));
    }

    pub(in crate::virtual_geometry) fn page_parent_pages(&self) -> &BTreeMap<u32, u32> {
        &self.page_parent_pages
    }

    pub(in crate::virtual_geometry) fn page_child_pages(&self) -> &BTreeMap<u32, Vec<u32>> {
        &self.page_child_pages
    }

    pub(in crate::virtual_geometry) fn page_dependency_count(&self) -> usize {
        self.page_parent_pages.len()
    }

    pub(in crate::virtual_geometry) fn page_descendant_ids(&self, page_id: u32) -> Vec<u32> {
        let mut stack = self
            .page_child_pages()
            .get(&page_id)
            .cloned()
            .unwrap_or_default();
        let mut descendants = BTreeSet::new();

        while let Some(candidate_page_id) = stack.pop() {
            if !descendants.insert(candidate_page_id) {
                continue;
            }
            if let Some(child_page_ids) = self.page_child_pages().get(&candidate_page_id) {
                stack.extend(child_page_ids.iter().copied());
            }
        }

        descendants.into_iter().collect()
    }

    /// 接收 extract 的页血缘并重建子页索引，供请求排序和淘汰策略共同读取。
    pub(in crate::virtual_geometry) fn replace_page_parent_pages(
        &mut self,
        page_parent_pages: BTreeMap<u32, u32>,
    ) {
        self.page_child_pages = page_child_pages_from_parent_pages(&page_parent_pages);
        self.page_parent_pages = page_parent_pages;
    }

    pub(in crate::virtual_geometry) fn retain_page_parent_pages(
        &mut self,
        mut retain: impl FnMut(&u32, &u32) -> bool,
    ) {
        let mut removed_page_ids = BTreeSet::new();
        self.page_parent_pages.retain(|page_id, parent_page_id| {
            let keep = retain(page_id, parent_page_id);
            if !keep {
                removed_page_ids.insert(*page_id);
            }
            keep
        });
        // Dropping a parent page also invalidates descendants whose parent link would dangle.
        while !removed_page_ids.is_empty() {
            let mut removed_child_page_ids = BTreeSet::new();
            self.page_parent_pages.retain(|page_id, parent_page_id| {
                let keep = !removed_page_ids.contains(parent_page_id);
                if !keep {
                    removed_child_page_ids.insert(*page_id);
                }
                keep
            });
            removed_page_ids = removed_child_page_ids;
        }
        self.page_child_pages = page_child_pages_from_parent_pages(&self.page_parent_pages);
    }
}

fn page_child_pages_from_parent_pages(
    page_parent_pages: &BTreeMap<u32, u32>,
) -> BTreeMap<u32, Vec<u32>> {
    let mut page_child_pages = BTreeMap::<u32, Vec<u32>>::new();
    for (&page_id, &parent_page_id) in page_parent_pages {
        page_child_pages
            .entry(parent_page_id)
            .or_default()
            .push(page_id);
    }
    for child_page_ids in page_child_pages.values_mut() {
        child_page_ids.sort_unstable();
        child_page_ids.dedup();
    }
    page_child_pages
}

#[cfg(test)]
#[path = "page_metadata/tests/descendant_allocation_tests.rs"]
mod descendant_allocation_tests;

#[cfg(test)]
#[path = "tests/page_metadata.rs"]
mod tests;
