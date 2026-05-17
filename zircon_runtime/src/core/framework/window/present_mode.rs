use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowPresentMode {
    AutoVsync,
    AutoNoVsync,
    #[default]
    Fifo,
    FifoRelaxed,
    Immediate,
    Mailbox,
}
