#[path = "mailbox_path.rs"]
mod mailbox_path;
#[path = "read.rs"]
mod read;
#[path = "wait.rs"]
mod wait;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub(crate) use wait::wait_for_editor_handshake;
