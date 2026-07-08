#[path = "ui_architecture/architecture_boundaries.rs"]
mod architecture_boundaries;
#[path = "ui_architecture/legacy_renames.rs"]
mod legacy_renames;
#[path = "ui_architecture/mirror_docs.rs"]
mod mirror_docs;
#[path = "ui_architecture/split_layout.rs"]
mod split_layout;
#[path = "ui_architecture/support.rs"]
mod support;

// Route-level helper mirrors; implementations stay in support.rs.
// fn repo_root()
// fn top_level_entry_names(
// fn rust_files_under(
// fn production_ui_file(
