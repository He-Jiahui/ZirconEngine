//! 宿主生命周期把模板事件副作用、失效事务、重算、原生窗口同步和启动装配接到同一帧循环。
#[path = "host_lifecycle/dispatch_effects.rs"]
mod dispatch_effects;
#[path = "host_lifecycle/invalidation_bridge.rs"]
mod invalidation_bridge;
#[path = "host_lifecycle/native_window_presenters.rs"]
mod native_window_presenters;
#[path = "host_lifecycle/pane_payloads.rs"]
mod pane_payloads;
#[path = "host_lifecycle/recompute.rs"]
mod recompute;
#[path = "host_lifecycle/recompute_viewport.rs"]
mod recompute_viewport;
#[path = "host_lifecycle/render_submission.rs"]
mod render_submission;
#[path = "host_lifecycle/scene_hierarchy_refresh.rs"]
mod scene_hierarchy_refresh;
#[path = "host_lifecycle/shell_metrics.rs"]
mod shell_metrics;
#[path = "host_lifecycle/startup.rs"]
mod startup;
#[path = "host_lifecycle/tick.rs"]
mod tick;
