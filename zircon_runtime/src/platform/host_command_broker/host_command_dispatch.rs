use crate::core::framework::window::{WindowCommandReceipt, WindowEffectiveSnapshot};

use super::{HostCommandExecution, WindowCommandFailure};

/// The platform thread either executes the next admitted command or observes
/// that its deadline already produced a terminal receipt.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum HostCommandDispatch {
    Execute(HostCommandExecution),
    Terminal(WindowCommandReceipt<WindowEffectiveSnapshot, WindowCommandFailure>),
}
