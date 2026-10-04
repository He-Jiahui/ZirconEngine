//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "p0_robustness/gpu_pass_timer.rs"]
mod gpu_pass_timer;
#[path = "p0_robustness/lock_poison.rs"]
mod lock_poison;
#[path = "p0_robustness/native_fixture.rs"]
mod native_fixture;
#[path = "p0_robustness/native_host_callbacks.rs"]
mod native_host_callbacks;
#[path = "p0_robustness/priority_recommendation.rs"]
mod priority_recommendation;
#[path = "p0_robustness/render_submit.rs"]
mod render_submit;
