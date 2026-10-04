use super::gpu_scene::{
    create_storage_buffer, grow_capacity, GpuScene, GPU_SCENE_INITIAL_VIRTUAL_GEOMETRY_CAPACITY,
};
use super::layout::{
    GpuVirtualGeometryClusterWord, GpuVirtualGeometryPage,
    GPU_VIRTUAL_GEOMETRY_CLUSTER_WORD_STRIDE, GPU_VIRTUAL_GEOMETRY_PAGE_STRIDE,
};
use super::upload::GpuSceneBufferUploadBatchBuilder;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use zr_rhi_wgpu::WgpuBufferUploadBatch;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuSceneVirtualGeometryUploadReport {
    pub(crate) page_count: u32,
    pub(crate) cluster_word_count: u32,
    pub(crate) uploaded_bytes: u64,
    pub(crate) rebuilt_bind_group: bool,
}

pub(crate) struct GpuScenePreparedVirtualGeometryUpload {
    pub(super) owner: Arc<()>,
    pub(super) batch: WgpuBufferUploadBatch,
    pub(super) report: GpuSceneVirtualGeometryUploadReport,
    pub(super) commit: GpuSceneVirtualGeometryUploadCommit,
}

pub(super) struct GpuSceneVirtualGeometryUploadCommit {
    pages: Vec<GpuVirtualGeometryPage>,
    cluster_words: Vec<GpuVirtualGeometryClusterWord>,
    reservation: GpuSceneVirtualGeometryPreparationReservation,
}

struct GpuSceneVirtualGeometryPreparationReservation {
    state: Arc<AtomicBool>,
}

impl GpuScenePreparedVirtualGeometryUpload {
    pub(crate) const fn report(&self) -> GpuSceneVirtualGeometryUploadReport {
        self.report
    }

    pub(crate) const fn scene_data_counts(&self) -> [u32; 2] {
        [self.report.page_count, self.report.cluster_word_count]
    }

    pub(super) fn is_owned_by(&self, owner: &Arc<()>) -> bool {
        Arc::ptr_eq(&self.owner, owner)
    }
}

impl GpuSceneVirtualGeometryUploadCommit {
    pub(super) fn commit(self, gpu_scene: &mut GpuScene) {
        let Self {
            pages,
            cluster_words,
            reservation: _reservation,
        } = self;
        gpu_scene.virtual_geometry_pages_shadow = pages;
        gpu_scene.virtual_geometry_clusters_shadow = cluster_words;
        gpu_scene.virtual_geometry_pages_require_full_upload = false;
        gpu_scene.virtual_geometry_clusters_require_full_upload = false;
    }
}

impl Drop for GpuSceneVirtualGeometryPreparationReservation {
    fn drop(&mut self) {
        self.state.store(false, Ordering::Release);
    }
}

impl GpuScene {
    pub(crate) fn prepare_virtual_geometry_resident_buffers(
        &mut self,
        device: &wgpu::Device,
        pages: Vec<GpuVirtualGeometryPage>,
        cluster_words: Vec<GpuVirtualGeometryClusterWord>,
    ) -> GpuScenePreparedVirtualGeometryUpload {
        let reservation = self.reserve_virtual_geometry_upload_preparation();
        let pages_changed = self.virtual_geometry_pages_require_full_upload
            || self.virtual_geometry_pages_shadow != pages;
        let clusters_changed = self.virtual_geometry_clusters_require_full_upload
            || self.virtual_geometry_clusters_shadow != cluster_words;
        let required_page_capacity =
            u32::try_from(pages.len()).expect("virtual geometry page buffer capacity exceeded u32");
        let required_cluster_capacity = u32::try_from(cluster_words.len())
            .expect("virtual geometry cluster buffer capacity exceeded u32");
        let page_buffer_replaced = required_page_capacity > self.virtual_geometry_pages_capacity;
        let cluster_buffer_replaced =
            required_cluster_capacity > self.virtual_geometry_clusters_capacity;

        if page_buffer_replaced {
            self.virtual_geometry_pages_require_full_upload = true;
            self.virtual_geometry_pages_capacity = grow_capacity(
                required_page_capacity,
                GPU_SCENE_INITIAL_VIRTUAL_GEOMETRY_CAPACITY,
            );
            self.virtual_geometry_pages_buffer = create_storage_buffer(
                device,
                "zircon-gpu-scene-virtual-geometry-pages",
                buffer_size_for_len(
                    self.virtual_geometry_pages_capacity as usize,
                    GPU_VIRTUAL_GEOMETRY_PAGE_STRIDE,
                ),
            );
        }
        if cluster_buffer_replaced {
            self.virtual_geometry_clusters_require_full_upload = true;
            self.virtual_geometry_clusters_capacity = grow_capacity(
                required_cluster_capacity,
                GPU_SCENE_INITIAL_VIRTUAL_GEOMETRY_CAPACITY,
            );
            self.virtual_geometry_clusters_buffer = create_storage_buffer(
                device,
                "zircon-gpu-scene-virtual-geometry-clusters",
                buffer_size_for_len(
                    self.virtual_geometry_clusters_capacity as usize,
                    GPU_VIRTUAL_GEOMETRY_CLUSTER_WORD_STRIDE,
                ),
            );
        }

        let mut uploads = GpuSceneBufferUploadBatchBuilder::new();
        let uploaded_bytes = if pages_changed {
            if self.virtual_geometry_pages_require_full_upload {
                uploads.push_pod_slice(&self.virtual_geometry_pages_buffer, 0, &pages)
            } else {
                uploads.push_changed_pod_slice(
                    &self.virtual_geometry_pages_buffer,
                    &self.virtual_geometry_pages_shadow,
                    &pages,
                )
            }
        } else {
            0
        } + if clusters_changed {
            if self.virtual_geometry_clusters_require_full_upload {
                uploads.push_pod_slice(&self.virtual_geometry_clusters_buffer, 0, &cluster_words)
            } else {
                uploads.push_changed_pod_slice(
                    &self.virtual_geometry_clusters_buffer,
                    &self.virtual_geometry_clusters_shadow,
                    &cluster_words,
                )
            }
        } else {
            0
        };

        let rebuilt_bind_group = page_buffer_replaced || cluster_buffer_replaced;
        if rebuilt_bind_group {
            self.rebuild_scene_bind_group(device);
        }

        GpuScenePreparedVirtualGeometryUpload {
            owner: Arc::clone(&self.upload_transaction_owner),
            batch: uploads.into_batch(),
            report: GpuSceneVirtualGeometryUploadReport {
                page_count: u32::try_from(pages.len()).unwrap_or(u32::MAX),
                cluster_word_count: u32::try_from(cluster_words.len()).unwrap_or(u32::MAX),
                uploaded_bytes,
                rebuilt_bind_group,
            },
            commit: GpuSceneVirtualGeometryUploadCommit {
                pages,
                cluster_words,
                reservation,
            },
        }
    }

    fn reserve_virtual_geometry_upload_preparation(
        &self,
    ) -> GpuSceneVirtualGeometryPreparationReservation {
        let reserved = self
            .virtual_geometry_preparation_reservation
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok();
        assert!(
            reserved,
            "GPU Scene permits one outstanding virtual-geometry preparation"
        );
        GpuSceneVirtualGeometryPreparationReservation {
            state: Arc::clone(&self.virtual_geometry_preparation_reservation),
        }
    }

    #[cfg(test)]
    pub(crate) fn debug_virtual_geometry_page_shadow(&self) -> &[GpuVirtualGeometryPage] {
        &self.virtual_geometry_pages_shadow
    }

    #[cfg(test)]
    pub(crate) fn debug_virtual_geometry_cluster_shadow(&self) -> &[GpuVirtualGeometryClusterWord] {
        &self.virtual_geometry_clusters_shadow
    }
}

fn buffer_size_for_len(len: usize, stride: usize) -> u64 {
    let byte_len = len
        .checked_mul(stride)
        .and_then(|bytes| u64::try_from(bytes).ok())
        .unwrap_or(u64::MAX);
    byte_len.max(16)
}

#[cfg(test)]
#[path = "tests/virtual_geometry.rs"]
mod tests;
