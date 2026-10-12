#[path = "default_menu_bar.rs"]
mod default_menu_bar;
#[path = "extension_menu.rs"]
mod extension_menu;

pub(super) use default_menu_bar::default_menu_bar_with_sources;

#[cfg(test)]
#[path = "tests/toolkit_menu_tests.rs"]
mod toolkit_menu_tests;
