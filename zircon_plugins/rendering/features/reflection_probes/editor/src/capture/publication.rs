//! 项目资源管理器拥有生成源的提交事务；本入口消费已编码探针源并转交 URI 与字节。
use zircon_plugin_rendering_reflection_probes_runtime::EncodedReflectionProbeCaptureSource;
use zircon_runtime::asset::{ProjectAssetManager, ProjectGeneratedSourceReceipt};
use zircon_runtime::core::CoreError;

/// 将已编码的源资产交给项目资产事务；成功回执由管理器提供，失败时本函数不自行补写文件。
pub fn publish_reflection_probe_capture_source(
    asset_manager: &ProjectAssetManager,
    source: EncodedReflectionProbeCaptureSource,
) -> Result<ProjectGeneratedSourceReceipt, ReflectionProbeCaptureProjectPublicationError> {
    let (output_uri, bytes) = source.into_parts();
    asset_manager
        .publish_generated_project_source(output_uri, bytes)
        .map_err(ReflectionProbeCaptureProjectPublicationError::Project)
}

#[derive(Debug, thiserror::Error)]
pub enum ReflectionProbeCaptureProjectPublicationError {
    #[error("publish captured reflection-probe source through the project asset transaction: {0}")]
    Project(#[source] CoreError),
}

#[cfg(test)]
#[path = "tests/publication.rs"]
mod tests;
