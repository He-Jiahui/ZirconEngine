#[path = "../extraction/command.rs"]
mod command;
#[path = "extraction/entry.rs"]
mod entry;
#[path = "extraction/image.rs"]
mod image;
#[path = "extraction/model.rs"]
mod model;
#[path = "../extraction/visibility.rs"]
mod visibility;

pub(in crate::ui::retained_host::host_contract) use entry::extract_chrome_commands;
pub(in crate::ui::retained_host::host_contract) use model::ChromeCommandExtraction;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use command::chrome_command_from_recorded_for_test;
