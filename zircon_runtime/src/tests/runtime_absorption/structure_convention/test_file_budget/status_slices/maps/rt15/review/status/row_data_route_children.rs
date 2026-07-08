use super::*;

#[path = "row_routes/budgets.rs"]
mod budgets;
#[path = "row_routes/child_paths.rs"]
mod child_paths;
#[path = "row_routes/literal_ownership.rs"]
mod literal_ownership;
#[path = "row_routes/route_input_ownership.rs"]
mod route_input_ownership;
#[path = "row_routes/route_inputs.rs"]
mod route_inputs;
#[path = "row_routes/route_mounts.rs"]
mod route_mounts;
#[path = "row_routes/source_reads.rs"]
mod source_reads;
#[path = "row_routes/status_mirrors.rs"]
mod status_mirrors;

use child_paths::*;
use route_inputs::*;
use source_reads::*;
