mod commands;
mod geometry;
mod identity;
mod style;
mod surface;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_chip_commands;

#[cfg(test)]
use super::super::paint_theme::PALETTE;
#[cfg(test)]
use identity::is_workbench_chip;
#[cfg(test)]
use style::{CHIP_BORDER, CHIP_SURFACE};

#[cfg(test)]
#[path = "template_chips_tests.rs"]
mod tests;
