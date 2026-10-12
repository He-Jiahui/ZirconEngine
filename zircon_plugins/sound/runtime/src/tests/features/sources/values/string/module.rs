#[path = "../../../../values/string/module/crate_name.rs"]
mod crate_name;
#[path = "../../../../sources/values/string/module/name.rs"]
mod name;
#[path = "../../../../sources/values/string/module/root.rs"]
mod root;

pub(in super::super::super::super) use crate_name::*;
pub(in super::super::super::super) use name::*;
pub(in super::super::super::super) use root::*;
