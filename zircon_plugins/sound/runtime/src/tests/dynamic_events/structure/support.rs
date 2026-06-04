mod assertions;
mod retired;
mod source;

pub(super) use assertions::{assert_source_contains, assert_structural_module};
pub(super) use retired::retired_flat_module;
pub(super) use source::src_root;
