#[path = "aggregation.rs"]
mod aggregation;
#[path = "axis_factory.rs"]
mod axis_factory;
#[path = "defaults.rs"]
mod defaults;
#[path = "merge.rs"]
mod merge;

pub(crate) use aggregation::aggregate_row_constraints;
pub(crate) use axis_factory::{fixed_axis, fixed_zero_constraints};
pub use defaults::{default_constraints_for_content, default_region_constraints};
pub(crate) use merge::{merge_constraints, set_primary_preferred};
