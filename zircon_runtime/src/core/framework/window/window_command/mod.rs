mod command_id;
mod header;
mod receipt;
mod state_generation;
mod terminal;

pub use command_id::WindowCommandId;
pub use header::{WindowCommand, WindowCommandHeader};
pub use receipt::{WindowCommandAccepted, WindowCommandReceipt};
pub use state_generation::WindowObservedGeneration;
pub use terminal::WindowCommandTerminal;

#[cfg(test)]
mod tests;
