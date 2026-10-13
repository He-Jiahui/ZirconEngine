//! 项目清单与所选引擎之间的语义版本准入判定及其可传递快照。
//! `assessment` 解析并评估版本要求，`directional_range` 仅在区间边界足以证明时给出升级或降级方向；其余子模块定义跨 crate 使用的值和错误。
//! Editor 的 `ProjectPreflightReceipt::evaluate_engine_compatibility` 与模板描述符的 `ProjectTemplateDescriptor::assess_engine_compatibility` 都调用公开评估入口；宿主仍须把此结论与迁移、provider 及项目准入策略合并后才能激活。
//! `crate::project::tests::engine_compatibility` 锁定公开结果，`zircon_runtime_interface/src/project/tests/directional_range.rs` 锁定比较符边界和方向归约。
#[path = "assessment.rs"]
mod assessment;
#[path = "compatibility.rs"]
mod compatibility;
#[path = "directional_range.rs"]
mod directional_range;
#[path = "disposition.rs"]
mod disposition;
#[path = "error.rs"]
mod error;
#[path = "version.rs"]
mod version;
#[path = "version_parse_error.rs"]
mod version_parse_error;

pub use assessment::assess_project_engine_compatibility;
pub use compatibility::ProjectEngineCompatibility;
pub use disposition::ProjectEngineCompatibilityDisposition;
pub use error::ProjectEngineCompatibilityError;
pub use version::ProjectEngineVersion;
pub use version_parse_error::ProjectEngineVersionParseError;
