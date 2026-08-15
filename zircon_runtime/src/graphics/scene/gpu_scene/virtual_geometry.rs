use super::gpu_scene::{
    create_storage_buffer, grow_capacity, GpuScene, GPU_SCENE_INITIAL_VIRTUAL_GEOMETRY_CAPACITY,
};
use super::layout::{
    GpuVirtualGeometryClusterWord, GpuVirtualGeometryPage,
    GPU_VIRTUAL_GEOMETRY_CLUSTER_WORD_STRIDE, GPU_VIRTUAL_GEOMETRY_PAGE_STRIDE,
};
use super::upload::{write_changed_pod_buffer, write_full_pod_buffer};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuSceneVirtualGeometryUploadReport {
    pub(crate) page_count: u32,
    pub(crate) cluster_word_count: u32,
    pub(crate) uploaded_bytes: u64,
    pub(crate) rebuilt_bind_group: bool,
}

impl GpuScene {
    pub(crate) fn upload_virtual_geometry_resident_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pages: &[GpuVirtualGeometryPage],
        cluster_words: &[GpuVirtualGeometryClusterWord],
    ) -> GpuSceneVirtualGeometryUploadReport {
        let pages_changed = self.virtual_geometry_pages_shadow != pages;
        let clusters_changed = self.virtual_geometry_clusters_shadow != cluster_words;
        let required_page_capacity =
            u32::try_from(pages.len()).expect("virtual geometry page buffer capacity exceeded u32");
        let required_cluster_capacity = u32::try_from(cluster_words.len())
            .expect("virtual geometry cluster buffer capacity exceeded u32");
        let page_buffer_replaced = required_page_capacity > self.virtual_geometry_pages_capacity;
        let cluster_buffer_replaced =
            required_cluster_capacity > self.virtual_geometry_clusters_capacity;

        if page_buffer_replaced {
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

        let uploaded_bytes = if pages_changed {
            if page_buffer_replaced {
                write_full_pod_buffer(
                    queue,
                    &self.virtual_geometry_pages_buffer,
                    pages,
                    pages.len(),
                )
            } else {
                write_changed_pod_buffer(
                    queue,
                    &self.virtual_geometry_pages_buffer,
                    &self.virtual_geometry_pages_shadow,
                    pages,
                )
            }
        } else {
            0
        } + if clusters_changed {
            if cluster_buffer_replaced {
                write_full_pod_buffer(
                    queue,
                    &self.virtual_geometry_clusters_buffer,
                    cluster_words,
                    cluster_words.len(),
                )
            } else {
                write_changed_pod_buffer(
                    queue,
                    &self.virtual_geometry_clusters_buffer,
                    &self.virtual_geometry_clusters_shadow,
                    cluster_words,
                )
            }
        } else {
            0
        };

        if pages_changed {
            self.virtual_geometry_pages_shadow.clear();
            self.virtual_geometry_pages_shadow.extend_from_slice(pages);
        }
        if clusters_changed {
            self.virtual_geometry_clusters_shadow.clear();
            self.virtual_geometry_clusters_shadow
                .extend_from_slice(cluster_words);
        }

        let rebuilt_bind_group = page_buffer_replaced || cluster_buffer_replaced;
        if rebuilt_bind_group {
            self.rebuild_scene_bind_group(device);
        }

        GpuSceneVirtualGeometryUploadReport {
            page_count: u32::try_from(pages.len()).unwrap_or(u32::MAX),
            cluster_word_count: u32::try_from(cluster_words.len()).unwrap_or(u32::MAX),
            uploaded_bytes,
            rebuilt_bind_group,
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
        GpuScene, GpuVirtualGeometryClusterWord, GpuVirtualGeometryPage,
        GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT,
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

        let first_report = scene.upload_virtual_geometry_resident_buffers(
            &backend.device,
            &backend.queue,
            &pages,
            &cluster_words,
        );

        assert_eq!(first_report.page_count, 2);
        assert_eq!(first_report.cluster_word_count, 2);
        assert!(first_report.uploaded_bytes > 0);
        assert!(first_report.rebuilt_bind_group);
        assert_eq!(scene.debug_virtual_geometry_page_shadow(), &pages);
        assert_eq!(
            scene.debug_virtual_geometry_cluster_shadow(),
            &cluster_words
        );

        let second_report = scene.upload_virtual_geometry_resident_buffers(
            &backend.device,
            &backend.queue,
            &pages,
            &cluster_words,
        );

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

        let first_report = scene.upload_virtual_geometry_resident_buffers(
            &backend.device,
            &backend.queue,
            &pages,
            &cluster_words,
        );
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
        let shrunk_report = scene.upload_virtual_geometry_resident_buffers(
            &backend.device,
            &backend.queue,
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
        let _ = scene.upload_virtual_geometry_resident_buffers(
            &backend.device,
            &backend.queue,
            &pages,
            &initial_cluster_words,
        );

        let changed_cluster_words = [
            initial_cluster_words[0],
            GpuVirtualGeometryClusterWord {
                values: [7.0, 8.0, 9.0, 1.0],
            },
        ];
        let report = scene.upload_virtual_geometry_resident_buffers(
            &backend.device,
            &backend.queue,
            &pages,
            &changed_cluster_words,
        );

        assert_eq!(
            report.uploaded_bytes, GPU_VIRTUAL_GEOMETRY_CLUSTER_WORD_STRIDE as u64,
            "one changed VG cluster word must upload exactly one storage row"
        );
        assert!(!report.rebuilt_bind_group);
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
