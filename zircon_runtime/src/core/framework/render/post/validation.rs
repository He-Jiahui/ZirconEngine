use thiserror::Error;

use super::super::RenderPipelinePhase;
use super::PostProcessEffectKind;

/// 图提交前的结构错误：资源供给、效果依赖和 ViewFamily 阶段均须先通过检查。
/// 帧提交应将此错误传播给上层；执行器不能靠缺失纹理时的临时回退代替验证。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum PostProcessGraphValidationError {
    #[error("post-process node `{node}` requires missing resource `{resource}`")]
    MissingRequiredInput { node: String, resource: String },
    #[error("post-process node `{node}` produces duplicate resource `{resource}`")]
    DuplicateOutputResource { node: String, resource: String },
    #[error("post-process node `{node}` depends on disabled or missing effect `{dependency}`")]
    MissingDependency {
        node: String,
        dependency: PostProcessEffectKind,
    },
    #[error("post-process node `{node}` requires unavailable view-family phase `{phase:?}`")]
    UnavailableViewFamilyPhase {
        node: String,
        phase: RenderPipelinePhase,
    },
    #[error("resolved view family requires post-process phase `{phase:?}`, but the stack has no node for it")]
    MissingRequiredViewFamilyPhase { phase: RenderPipelinePhase },
    #[error("post-process pass graph contains a dependency cycle")]
    CycleDetected,
}
