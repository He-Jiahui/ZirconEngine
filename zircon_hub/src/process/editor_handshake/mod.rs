mod mailbox_path;
mod read;
mod wait;

#[cfg(test)]
mod tests;

pub(crate) use wait::wait_for_editor_handshake;
