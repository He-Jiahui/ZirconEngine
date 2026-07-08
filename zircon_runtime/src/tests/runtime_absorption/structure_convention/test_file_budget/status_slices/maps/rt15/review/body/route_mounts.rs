use super::*;

#[path = "mounts/budgets.rs"]
mod budgets;
#[path = "mounts/folder_backed.rs"]
mod folder_backed;
#[path = "mounts/moved_children.rs"]
mod moved_children;
#[path = "mounts/parent_routes.rs"]
mod parent_routes;
#[path = "mounts/paths.rs"]
mod paths;
#[path = "mounts/status_docs.rs"]
mod status_docs;

use paths::*;
