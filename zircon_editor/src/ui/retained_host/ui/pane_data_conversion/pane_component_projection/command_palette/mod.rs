mod attributes;
mod entries;
mod entry;
mod ids;
mod options;
mod parse;

pub(in crate::ui::retained_host::ui) use self::options::{
    projected_command_palette_option_rows, projected_command_palette_options,
    projected_command_palette_structured_options,
};

#[cfg(test)]
mod tests;
