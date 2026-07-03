use super::gpu_scene::{create_storage_buffer, GpuScene};
use super::layout::{
    GpuVirtualGeometryClusterWord, GpuVirtualGeometryPage,
    GPU_VIRTUAL_GEOMETRY_CLUSTER_WORD_STRIDE, GPU_VIRTUAL_GEOMETRY_PAGE_STRIDE,
};
use super::upload::write_full_pod_buffer;

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
        let page_capacity_changed = self.virtual_geometry_pages_shadow.len() != pages.len();
        let cluster_capacity_changed =
            self.virtual_geometry_clusters_shadow.len() != cluster_words.len();

        if page_capacity_changed {
            self.virtual_geometry_pages_buffer = create_storage_buffer(
                device,
                "zircon-gpu-scene-virtual-geometry-pages",
                buffer_size_for_len(pages.len(), GPU_VIRTUAL_GEOMETRY_PAGE_STRIDE),
            );
        }
        if cluster_capacity_changed {
            self.virtual_geometry_clusters_buffer = create_storage_buffer(
                device,
                "zircon-gpu-scene-virtual-geometry-clusters",
                buffer_size_for_len(
                    cluster_words.len(),
                    GPU_VIRTUAL_GEOMETRY_CLUSTER_WORD_STRIDE,
                ),
            );
        }

        self.virtual_geometry_pages_shadow.clear();
        self.virtual_geometry_pages_shadow.extend_from_slice(pages);
        self.virtual_geometry_clusters_shadow.clear();
        self.virtual_geometry_clusters_shadow
            .extend_from_slice(cluster_words);

        let uploaded_bytes = write_full_pod_buffer(
            queue,
            &self.virtual_geometry_pages_buffer,
            &self.virtual_geometry_pages_shadow,
            pages.len(),
        ) + write_full_pod_buffer(
            queue,
            &self.virtual_geometry_clusters_buffer,
            &self.virtual_geometry_clusters_shadow,
            cluster_words.len(),
        );
        let rebuilt_bind_group = page_capacity_changed || cluster_capacity_changed;
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
        assert!(second_report.uploaded_bytes > 0);
        assert!(!second_report.rebuilt_bind_group);
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
            usage: wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        }))
    }

    fn test_skinned_joint_palette_min_binding_size() -> wgpu::BufferSize {
        wgpu::BufferSize::new(
            TEST_SKINNED_JOINT_MATRIX_COUNT * TEST_SKINNED_JOINT_MATRIX_BYTES
                + TEST_SKINNED_JOINT_PARAMS_BYTES,
        )
        .expect("test skinned joint palette uniform size is non-zero")
    }
}
