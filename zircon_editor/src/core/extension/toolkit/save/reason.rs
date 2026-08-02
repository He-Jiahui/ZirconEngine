#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveReason {
    Explicit,
    SaveAll,
    Close,
}
