#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WelcomeRecentPointerAction {
    Open,
    Safe,
    Recover,
    Remove,
}
