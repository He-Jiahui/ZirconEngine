use super::gpu_scene::{
    GPU_SCENE_INITIAL_VIRTUAL_GEOMETRY_CAPACITY, GpuScene, create_storage_buffer, grow_capacity,
};
use super::layout::{
    GPU_VIRTUAL_GEOMETRY_CLUSTER_WORD_STRIDE, GPU_VIRTUAL_GEOMETRY_PAGE_STRIDE,
    GpuVirtualGeometryClusterWord, GpuVirtualGeometryPage,
};
use super::upload::GpuSceneBufferUploadBatchBuilder;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
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
mod tests {
    use std::sync::Arc;

    use super::super::layout::GPU_VIRTUAL_GEOMETRY_CLUSTER_WORD_STRIDE;
    use crate::graphics::scene::gpu_scene::{
        GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT, GpuScene, GpuSceneVirtualGeometryUploadReport,
        GpuVirtualGeometryClusterWord, GpuVirtualGeometryPage,
    };

    const TEST_SKINNED_JOINT_MATRIX_COUNT: u64 = 256;
    const TEST_SKINNED_JOINT_MATRIX_BYTES: u64 = 64;
    const TEST_SKINNED_JOINT_PARAMS_BYTES: u64 = 16;

    #[test]
    fn render_gpu_scene_uploads_virtual_geometry_resident_buffers() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);
        let pages = [
            GpuVirtualGeometryPage::new(0, 4, 20, GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT),
            GpuVirtualGeometryPage::new(4, 8, 30, GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT),
        ];
        let cluster_words = [
            GpuVirtualGeometryClusterWord {
                values: [1.0, 2.0, 3.0, 1.0],
            },
            GpuVirtualGeometryClusterWord {
                values: [0.0, 1.0, 0.0, 0.0],
            },
        ];

        let first_report =
            submit_virtual_geometry_upload(&mut scene, &backend, &pages, &cluster_words);

        assert_eq!(first_report.page_count, 2);
        assert_eq!(first_report.cluster_word_count, 2);
        assert!(first_report.uploaded_bytes > 0);
        assert!(first_report.rebuilt_bind_group);
        assert_eq!(scene.debug_virtual_geometry_page_shadow(), &pages);
        assert_eq!(
            scene.debug_virtual_geometry_cluster_shadow(),
            &cluster_words
        );

        let second_report =
            submit_virtual_geometry_upload(&mut scene, &backend, &pages, &cluster_words);

        assert_eq!(second_report.page_count, 2);
        assert_eq!(second_report.cluster_word_count, 2);
        assert_eq!(second_report.uploaded_bytes, 0);
        assert!(!second_report.rebuilt_bind_group);
    }

    #[test]
    fn render_gpu_scene_reuses_virtual_geometry_buffers_when_active_rows_shrink() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);
        let pages = [
            GpuVirtualGeometryPage::new(0, 4, 20, GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT),
            GpuVirtualGeometryPage::new(4, 8, 30, GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT),
        ];
        let cluster_words = [
            GpuVirtualGeometryClusterWord {
                values: [1.0, 2.0, 3.0, 1.0],
            },
            GpuVirtualGeometryClusterWord {
                values: [4.0, 5.0, 6.0, 1.0],
            },
        ];

        let first_report =
            submit_virtual_geometry_upload(&mut scene, &backend, &pages, &cluster_words);
        assert!(first_report.rebuilt_bind_group);

        let shrunk_pages = [GpuVirtualGeometryPage::new(
            0,
            4,
            20,
            GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT,
        )];
        let shrunk_cluster_words = [GpuVirtualGeometryClusterWord {
            values: [7.0, 8.0, 9.0, 1.0],
        }];
        let shrunk_report = submit_virtual_geometry_upload(
            &mut scene,
            &backend,
            &shrunk_pages,
            &shrunk_cluster_words,
        );

        assert!(shrunk_report.uploaded_bytes > 0);
        assert!(
            !shrunk_report.rebuilt_bind_group,
            "a smaller active resident set must reuse the existing VG buffers and scene bind group"
        );
        assert_eq!(scene.debug_virtual_geometry_page_shadow(), &shrunk_pages);
        assert_eq!(
            scene.debug_virtual_geometry_cluster_shadow(),
            &shrunk_cluster_words
        );
    }

    #[test]
    fn render_gpu_scene_uploads_only_changed_virtual_geometry_cluster_rows() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);
        let pages = [GpuVirtualGeometryPage::new(
            0,
            1,
            20,
            GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT,
        )];
        let initial_cluster_words = [
            GpuVirtualGeometryClusterWord {
                values: [1.0, 2.0, 3.0, 1.0],
            },
            GpuVirtualGeometryClusterWord {
                values: [4.0, 5.0, 6.0, 1.0],
            },
        ];
        let _ =
            submit_virtual_geometry_upload(&mut scene, &backend, &pages, &initial_cluster_words);

        let changed_cluster_words = [
            initial_cluster_words[0],
            GpuVirtualGeometryClusterWord {
                values: [7.0, 8.0, 9.0, 1.0],
            },
        ];
        let report =
            submit_virtual_geometry_upload(&mut scene, &backend, &pages, &changed_cluster_words);

        assert_eq!(
            report.uploaded_bytes, GPU_VIRTUAL_GEOMETRY_CLUSTER_WORD_STRIDE as u64,
            "one changed VG cluster word must upload exactly one storage row"
        );
        assert!(!report.rebuilt_bind_group);
    }

    #[test]
    fn render_gpu_scene_dropped_virtual_geometry_preparation_keeps_shadow_for_retry() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);
        let pages = [GpuVirtualGeometryPage::new(
            0,
            1,
            20,
            GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT,
        )];
        let cluster_words = [GpuVirtualGeometryClusterWord {
            values: [1.0, 2.0, 3.0, 1.0],
        }];
        submit_virtual_geometry_upload(&mut scene, &backend, &pages, &cluster_words);
        let grown_pages = vec![pages[0]; scene.virtual_geometry_pages_capacity as usize + 1];
        let dropped = scene.prepare_virtual_geometry_resident_buffers(
            &backend.device,
            grown_pages,
            cluster_words.to_vec(),
        );
        assert!(dropped.report().uploaded_bytes > 0);
        assert!(dropped.report().rebuilt_bind_group);
        let prospective_counts = dropped.scene_data_counts();
        let mut scene_data_counts = scene.current_scene_data_counts();
        scene_data_counts[1] = prospective_counts[0];
        scene_data_counts[2] = prospective_counts[1];
        let mut dropped_frame =
            scene.prepare_direct_updates_for_scene_data_counts(scene_data_counts);
        dropped_frame.append_virtual_geometry_upload(dropped);
        drop(dropped_frame);
        assert_eq!(scene.debug_virtual_geometry_page_shadow(), &pages);

        let retry = scene.prepare_virtual_geometry_resident_buffers(
            &backend.device,
            pages.to_vec(),
            cluster_words.to_vec(),
        );
        assert!(retry.report().uploaded_bytes > 0);
    }

    #[test]
    fn render_gpu_scene_rejects_overlapping_virtual_geometry_preparations() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);
        let first = scene.prepare_virtual_geometry_resident_buffers(
            &backend.device,
            vec![GpuVirtualGeometryPage::new(
                0,
                1,
                20,
                GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT,
            )],
            vec![GpuVirtualGeometryClusterWord {
                values: [1.0, 2.0, 3.0, 1.0],
            }],
        );

        let overlapping = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            scene.prepare_virtual_geometry_resident_buffers(&backend.device, Vec::new(), Vec::new())
        }));
        assert!(overlapping.is_err());

        drop(first);
        let retry = scene.prepare_virtual_geometry_resident_buffers(
            &backend.device,
            Vec::new(),
            Vec::new(),
        );
        drop(retry);
    }

    #[test]
    fn render_gpu_scene_rejects_foreign_virtual_geometry_preparation_attachment() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut source_scene = test_gpu_scene(&backend.device);
        let mut target_scene = test_gpu_scene(&backend.device);
        let prepared = source_scene.prepare_virtual_geometry_resident_buffers(
            &backend.device,
            vec![GpuVirtualGeometryPage::new(
                0,
                1,
                20,
                GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT,
            )],
            vec![GpuVirtualGeometryClusterWord {
                values: [1.0, 2.0, 3.0, 1.0],
            }],
        );
        let mut target_frame = target_scene.prepare_direct_updates();

        let attachment = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            target_frame.append_virtual_geometry_upload(prepared);
        }));

        assert!(attachment.is_err());
        drop(target_frame);
        let retry = source_scene.prepare_virtual_geometry_resident_buffers(
            &backend.device,
            Vec::new(),
            Vec::new(),
        );
        drop(retry);
    }

    fn submit_virtual_geometry_upload(
        scene: &mut GpuScene,
        backend: &crate::graphics::backend::RenderBackend,
        pages: &[GpuVirtualGeometryPage],
        cluster_words: &[GpuVirtualGeometryClusterWord],
    ) -> GpuSceneVirtualGeometryUploadReport {
        let prepared = scene.prepare_virtual_geometry_resident_buffers(
            &backend.device,
            pages.to_vec(),
            cluster_words.to_vec(),
        );
        let report = prepared.report();
        let prospective_counts = prepared.scene_data_counts();
        let mut scene_data_counts = scene.current_scene_data_counts();
        scene_data_counts[1] = prospective_counts[0];
        scene_data_counts[2] = prospective_counts[1];
        let mut frame = scene.prepare_direct_updates_for_scene_data_counts(scene_data_counts);
        frame.append_virtual_geometry_upload(prepared);
        scene
            .submit_prepared_upload(backend, frame)
            .expect("virtual-geometry upload batch must be accepted by the test backend");
        report
    }

    fn test_backend() -> Option<crate::graphics::backend::RenderBackend> {
        crate::graphics::backend::RenderBackend::new_offscreen()
            .inspect_err(|error| eprintln!("skipping gpu scene VG upload test: {error:?}"))
            .ok()
    }

    fn test_gpu_scene(device: &wgpu::Device) -> GpuScene {
        GpuScene::new(
            device,
            test_skinned_joint_palette_buffer(device),
            test_skinned_joint_palette_min_binding_size(),
        )
    }

    fn test_skinned_joint_palette_buffer(device: &wgpu::Device) -> Arc<wgpu::Buffer> {
        Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-test-empty-skinned-joint-palette-buffer"),
            size: test_skinned_joint_palette_min_binding_size().get(),
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        }))
    }

    fn test_skinned_joint_palette_min_binding_size() -> wgpu::BufferSize {
        wgpu::BufferSize::new(
            TEST_SKINNED_JOINT_MATRIX_COUNT * TEST_SKINNED_JOINT_MATRIX_BYTES
                + TEST_SKINNED_JOINT_PARAMS_BYTES,
        )
        .expect("test skinned joint palette storage size is non-zero")
    }
}
