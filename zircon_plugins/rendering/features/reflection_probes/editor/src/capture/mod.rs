//! 编辑器探针命令只持有序列化请求与框架句柄；项目发布与捕获执行分别拥有各自的资源边界。
mod publication;
mod trigger;

pub use publication::{
    ReflectionProbeCaptureProjectPublicationError, publish_reflection_probe_capture_source,
};
pub use trigger::{
    ReflectionProbeCaptureEditorCommand, ReflectionProbeCaptureEditorCommandError,
    ReflectionProbeCaptureEditorExecutionError, ReflectionProbeCaptureEditorResult,
    ReflectionProbeCaptureEditorTrigger,
};
