/// 应用级状态机：恢复与挂起先进入过渡态，宿主回执后才成为稳定状态。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ApplicationLifecycleState {
    #[default]
    Cold,
    Running,
    WillSuspend,
    Suspended,
    WillResume,
    Exiting,
}
