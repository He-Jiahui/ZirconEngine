#[path = "builtin_modules/core_spine.rs"]
#[cfg(all(feature = "graphics", feature = "script", feature = "ui"))]
mod core_spine;
#[path = "builtin_modules/plugin_selection.rs"]
mod plugin_selection;
#[path = "builtin_modules/split_layout.rs"]
mod split_layout;
