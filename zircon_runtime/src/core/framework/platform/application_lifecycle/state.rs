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
