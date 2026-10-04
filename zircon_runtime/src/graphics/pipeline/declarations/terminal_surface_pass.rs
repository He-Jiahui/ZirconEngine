//! 终端 pass 的名字、executor ID 与外部输出资源名必须跨 authoring、注册表和执行器一致。
//! 这些常量是图形管线与 scene renderer 之间的字符串契约，改动需同步对应消费者。
pub(crate) const SURFACE_PRESENT_PASS_NAME: &str = "surface-present";
pub(crate) const SURFACE_PRESENT_EXECUTOR_ID: &str = "frame.surface-present";
pub(crate) const OUTPUT_TARGET_DIRECT_IMPORT_PASS_NAME: &str = "output-target-direct-import";
pub(crate) const OUTPUT_TARGET_DIRECT_IMPORT_EXECUTOR_ID: &str =
    "frame.output-target-direct-import";
pub(crate) const OUTPUT_TARGET_WRITEBACK_PASS_NAME: &str = "output-target-writeback";
pub(crate) const OUTPUT_TARGET_WRITEBACK_EXECUTOR_ID: &str = "frame.output-target-writeback";
pub(crate) const OUTPUT_TARGET_TEXTURE_RESOURCE_NAME: &str = "camera-output-target";
