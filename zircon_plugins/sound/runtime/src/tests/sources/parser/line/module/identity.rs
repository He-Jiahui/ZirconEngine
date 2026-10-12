#[path = "../../../../parser/line/module/identity/crate_name.rs"]
mod crate_name;
#[path = "../../../../parser/line/module/identity/name.rs"]
mod name;
#[path = "../../../../parser/line/module/identity/root.rs"]
mod root;

pub(in super::super::super::super::super) use crate_name::*;
pub(in super::super::super::super::super) use name::*;
pub(in super::super::super::super::super) use root::*;
