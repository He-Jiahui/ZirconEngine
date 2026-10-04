use std::fmt::Debug;

use super::{
    VirtualGeometryRuntimeFeedback, VirtualGeometryRuntimePrepareInput,
    VirtualGeometryRuntimePrepareOutput, VirtualGeometryRuntimeUpdate,
};

/// 单个相机历史的虚拟几何驻留状态；先依据可见页计划准备帧，再在成功提交后吸收 GPU 与可见性反馈。
/// 实现不得把另一相机或旧代际的页请求当作当前状态。
pub trait VirtualGeometryRuntimeState: Debug + Send {
    fn prepare_frame(
        &mut self,
        input: VirtualGeometryRuntimePrepareInput<'_>,
    ) -> VirtualGeometryRuntimePrepareOutput;

    fn update_after_render(
        &mut self,
        feedback: VirtualGeometryRuntimeFeedback,
    ) -> VirtualGeometryRuntimeUpdate;
}
