use super::*;

#[path = "parent_routes/guard_body.rs"]
mod guard_body;
#[path = "parent_routes/paths.rs"]
mod paths;
#[path = "parent_routes/route_metadata.rs"]
mod route_metadata;
#[path = "parent_routes/sources.rs"]
mod sources;

use paths::*;
use sources::*;
