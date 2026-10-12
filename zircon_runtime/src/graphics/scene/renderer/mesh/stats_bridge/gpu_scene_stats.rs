use super::super::PreparedMeshQueueStats;
use crate::core::framework::render::RenderGpuSceneUploadPath;
use crate::graphics::scene::gpu_scene::{GpuSceneStats, GpuSceneUploadPath, GpuSceneUploadReport};

impl PreparedMeshQueueStats {
    /// 在帧成功提交后并入 GPUScene 实际上传结果。
    /// 调用方须传入对应本帧的 upload_report，避免把预估值当提交事实。
    pub(crate) fn with_gpu_scene_stats(
        mut self,
        stats: GpuSceneStats,
        upload_report: GpuSceneUploadReport,
    ) -> Self {
        self.gpu_scene_primitive_count = stats.primitive_count;
        self.gpu_scene_instance_count = stats.instance_count;
        self.gpu_scene_dirty_entry_count = stats.dirty_entry_count;
        self.gpu_scene_uploaded_bytes = upload_report.uploaded_bytes;
        self.gpu_scene_upload_path = render_gpu_scene_upload_path(upload_report.upload_path);
        self.gpu_scene_free_span_count = stats.free_span_count;
        self.gpu_scene_primitive_upload_range_count = upload_report.primitive_upload_range_count;
        self.gpu_scene_instance_upload_range_count = upload_report.instance_upload_range_count;
        self
    }
}

fn render_gpu_scene_upload_path(path: GpuSceneUploadPath) -> RenderGpuSceneUploadPath {
    match path {
        GpuSceneUploadPath::DirectQueueWrite => RenderGpuSceneUploadPath::DirectQueueWrite,
        GpuSceneUploadPath::StagingCopy => RenderGpuSceneUploadPath::StagingCopy,
    }
}
