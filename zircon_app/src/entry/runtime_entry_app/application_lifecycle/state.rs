//! Winit 表面准入与宿主已发布表面状态的内部标记。
//! 状态迁移由 machine/transitions 管理；window 和 presenter 的真实释放仍由 App 执行。

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum ApplicationLifecycleState {
    #[default]
    Cold,
    AwaitingSurface,
    SurfaceActive,
    Suspended,
    Exiting,
}
