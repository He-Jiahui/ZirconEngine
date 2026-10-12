//! Host 窗口副作用的请求与回执协议：入队确认只代表接纳，终态回执才给出生效状态。
//! 目标窗口和请求分别带身份，避免延迟完成覆盖复用槽位或后续请求。

#[path = "command_id.rs"]
mod command_id;
#[path = "header.rs"]
mod header;
#[path = "receipt.rs"]
mod receipt;
#[path = "state_generation.rs"]
mod state_generation;
#[path = "terminal.rs"]
mod terminal;

pub use command_id::WindowCommandId;
pub use header::{WindowCommand, WindowCommandHeader};
pub use receipt::{WindowCommandAccepted, WindowCommandReceipt};
pub use state_generation::WindowObservedGeneration;
pub use terminal::WindowCommandTerminal;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
