#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 环境图像不适用、沿用现有 bundle、或为本次请求编码新 bundle 的三种结果。
pub enum EnvironmentIblSourceStagingStatus {
    Skipped,
    Reused,
    Written,
}
