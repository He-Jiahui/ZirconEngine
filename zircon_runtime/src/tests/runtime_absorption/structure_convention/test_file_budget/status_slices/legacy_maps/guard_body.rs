use super::*;

#[path = "body/budgets.rs"]
mod budgets;
#[path = "body/folder_backed.rs"]
mod folder_backed;
#[path = "body/legacy_routes.rs"]
mod legacy_routes;
#[path = "body/paths.rs"]
mod paths;
#[path = "body/status_mirrors.rs"]
mod status_mirrors;

use paths::*;
