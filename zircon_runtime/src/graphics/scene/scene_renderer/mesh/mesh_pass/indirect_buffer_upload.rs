use core::ops::Range;
use std::sync::Arc;

use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct IndirectBufferUploadStats {
    pub(super) byte_count: u64,
    pub(super) range_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PodRangeUploadCommit {
    staged_revision: u64,
    buffer_revision: u64,
}

/// Keeps accepted CPU state separate from the reusable preparation scratch.
pub(super) struct PodRangeUploadShadow<T> {
    committed: Vec<T>,
    staged: Vec<T>,
    next_staged_revision: u64,
    staged_revision: u64,
    committed_buffer_revision: Option<u64>,
}

impl<T> Default for PodRangeUploadShadow<T> {
    fn default() -> Self {
        Self {
            committed: Vec::new(),
            staged: Vec::new(),
            next_staged_revision: 1,
            staged_revision: 0,
            committed_buffer_revision: None,
        }
    }
}

impl<T> PodRangeUploadShadow<T>
where
    T: bytemuck::Pod + Copy + PartialEq,
{
    pub(super) fn prepare(
        &mut self,
        buffer: &wgpu::Buffer,
        buffer_revision: u64,
        current: &[T],
        uploads: &mut WgpuBufferUploadBatch,
    ) -> (IndirectBufferUploadStats, Option<PodRangeUploadCommit>) {
        let force_full_upload = self.committed_buffer_revision != Some(buffer_revision);
        if !force_full_upload && self.committed == current {
            return (IndirectBufferUploadStats::default(), None);
        }

        let dirty_ranges = changed_element_ranges(&self.committed, current, force_full_upload);
        let stats = append_pod_range_uploads(buffer, current, &dirty_ranges, uploads);

        self.staged.clear();
        self.staged.extend_from_slice(current);
        self.staged_revision = self.next_staged_revision;
        self.next_staged_revision = self.next_staged_revision.wrapping_add(1).max(1);
        (
            stats,
            Some(PodRangeUploadCommit {
                staged_revision: self.staged_revision,
                buffer_revision,
            }),
        )
    }

    pub(super) fn accepts(&self, commit: PodRangeUploadCommit) -> bool {
        self.staged_revision == commit.staged_revision
    }

    pub(super) fn commit(&mut self, commit: PodRangeUploadCommit) -> bool {
        if !self.accepts(commit) {
            return false;
        }
        std::mem::swap(&mut self.committed, &mut self.staged);
        self.staged.clear();
        self.committed_buffer_revision = Some(commit.buffer_revision);
        true
    }
}

pub(super) fn changed_element_ranges<T: PartialEq>(
    committed: &[T],
    current: &[T],
    force_full_upload: bool,
) -> Vec<Range<usize>> {
    if force_full_upload {
        return (!current.is_empty())
            .then_some(0..current.len())
            .into_iter()
            .collect();
    }

    let shared_len = committed.len().min(current.len());
    let mut dirty_ranges = Vec::new();
    let mut cursor = 0usize;
    while cursor < shared_len {
        while cursor < shared_len && committed[cursor] == current[cursor] {
            cursor += 1;
        }
        let dirty_start = cursor;
        while cursor < shared_len && committed[cursor] != current[cursor] {
            cursor += 1;
        }
        if dirty_start != cursor {
            dirty_ranges.push(dirty_start..cursor);
        }
    }
    if current.len() > shared_len {
        if dirty_ranges
            .last()
            .is_some_and(|range| range.end == shared_len)
        {
            dirty_ranges
                .last_mut()
                .expect("the preceding range check established a last range")
                .end = current.len();
        } else {
            dirty_ranges.push(shared_len..current.len());
        }
    }
    dirty_ranges
}

fn append_pod_range_uploads<T: bytemuck::Pod>(
    buffer: &wgpu::Buffer,
    current: &[T],
    dirty_ranges: &[Range<usize>],
    uploads: &mut WgpuBufferUploadBatch,
) -> IndirectBufferUploadStats {
    if dirty_ranges.is_empty() {
        return IndirectBufferUploadStats::default();
    }

    let element_size = std::mem::size_of::<T>();
    assert!(
        element_size > 0,
        "indirect buffer elements must not be zero-sized"
    );
    let payload: Arc<[u8]> = bytemuck::cast_slice(current).into();
    let mut byte_count = 0_u64;
    for element_range in dirty_ranges {
        let byte_start = element_range
            .start
            .checked_mul(element_size)
            .expect("indirect upload byte start overflowed usize");
        let byte_end = element_range
            .end
            .checked_mul(element_size)
            .expect("indirect upload byte end overflowed usize");
        let destination_offset =
            u64::try_from(byte_start).expect("indirect upload offset did not fit u64");
        uploads.push(
            WgpuBufferUpload::new(
                buffer.clone(),
                destination_offset,
                Arc::clone(&payload),
                byte_start..byte_end,
            )
            .expect("indirect upload source range must fit the packed payload"),
        );
        byte_count = byte_count.saturating_add(
            u64::try_from(byte_end - byte_start)
                .expect("indirect upload byte count did not fit u64"),
        );
    }
    IndirectBufferUploadStats {
        byte_count,
        range_count: u32::try_from(dirty_ranges.len()).unwrap_or(u32::MAX),
    }
}

#[cfg(test)]
mod tests {
    use super::changed_element_ranges;

    #[test]
    fn stable_values_produce_no_dirty_ranges() {
        assert!(changed_element_ranges(&[1_u32, 2, 3], &[1, 2, 3], false).is_empty());
    }

    #[test]
    fn changed_ranges_cover_growth_and_contiguous_changes() {
        assert_eq!(
            changed_element_ranges(&[1_u32, 2, 3], &[1, 7, 8, 9, 10], false),
            vec![1..5]
        );
    }

    #[test]
    fn shrink_only_updates_the_committed_shadow_without_gpu_writes() {
        assert!(changed_element_ranges(&[1_u32, 2, 3], &[1, 2], false).is_empty());
    }

    #[test]
    fn interleaved_changes_remain_exact_until_profile_justifies_coalescing() {
        assert_eq!(
            changed_element_ranges(&[1_u32, 2, 3, 4, 5], &[9, 2, 8, 4, 7], false),
            vec![0..1, 2..3, 4..5]
        );
    }

    #[test]
    fn forced_upload_covers_the_complete_non_empty_payload() {
        assert_eq!(
            changed_element_ranges(&[1_u32, 2], &[1, 2, 3], true),
            vec![0..3]
        );
        assert!(changed_element_ranges::<u32>(&[1, 2], &[], true).is_empty());
    }
}
