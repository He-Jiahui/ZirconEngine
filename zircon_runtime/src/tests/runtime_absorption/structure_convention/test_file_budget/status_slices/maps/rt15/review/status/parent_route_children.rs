use super::*;

#[path = "parent_routes/budgets.rs"]
mod budgets;
#[path = "parent_routes/literal_ownership.rs"]
mod literal_ownership;
#[path = "parent_routes/route_input_ownership.rs"]
mod route_input_ownership;
#[path = "parent_routes/route_inputs.rs"]
mod route_inputs;
#[path = "parent_routes/route_mounts.rs"]
mod route_mounts;
#[path = "parent_routes/source_reads.rs"]
mod source_reads;
#[path = "parent_routes/status_mirrors.rs"]
mod status_mirrors;

use route_inputs::*;
use source_reads::*;
