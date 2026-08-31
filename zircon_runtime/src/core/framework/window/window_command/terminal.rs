/// The one terminal disposition for an accepted window command. Rejected and
/// failed outcomes retain their platform reason while the enclosing receipt
/// always records the exact effective state observed at completion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowCommandTerminal<Failure> {
    Applied,
    Rejected { reason: Failure },
    Canceled,
    Failed { reason: Failure },
}

impl<Failure> WindowCommandTerminal<Failure> {
    pub const fn is_applied(&self) -> bool {
        matches!(self, Self::Applied)
    }

    pub const fn is_terminal(&self) -> bool {
        true
    }
}
