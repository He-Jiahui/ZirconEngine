mod mailbox_path;
mod read;
mod wait;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub(crate) use wait::wait_for_editor_handshake;
