#[path = "attributes.rs"]
mod attributes;
#[path = "entries.rs"]
mod entries;
#[path = "entry.rs"]
mod entry;
#[path = "ids.rs"]
mod ids;
#[path = "options.rs"]
mod options;
#[path = "parse.rs"]
mod parse;

pub(in crate::ui::retained_host::ui) use self::options::{
    projected_command_palette_option_rows, projected_command_palette_options,
    projected_command_palette_structured_options,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
